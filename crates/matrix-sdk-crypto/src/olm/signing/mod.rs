// Copyright 2020 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod pk_signing;

use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use matrix_sdk_common::locks::Mutex;
use pk_signing::{MasterSigning, PickledSignings, SelfSigning, Signing, SigningError, UserSigning};
use ruma::{
    api::client::r0::keys::upload_signatures::Request as SignatureUploadRequest,
    encryption::{DeviceKeys, KeyUsage},
    events::secret::request::SecretName,
    UserId,
};
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;

use crate::{
    error::SignatureError,
    identities::{MasterPubkey, SelfSigningPubkey, UserSigningPubkey},
    requests::UploadSigningKeysRequest,
    store::SecretImportError,
    utilities::decode,
    OwnUserIdentity, ReadOnlyAccount, ReadOnlyDevice, ReadOnlyOwnUserIdentity,
    ReadOnlyUserIdentity,
};

/// Private cross signing identity.
///
/// This object holds the private and public ed25519 key triplet that is used
/// for cross signing.
///
/// The object might be completely empty or have only some of the key pairs
/// available.
///
/// It can be used to sign devices or other identities.
#[derive(Clone, Debug)]
pub struct PrivateCrossSigningIdentity {
    user_id: Arc<UserId>,
    shared: Arc<AtomicBool>,
    pub(crate) master_key: Arc<Mutex<Option<MasterSigning>>>,
    pub(crate) user_signing_key: Arc<Mutex<Option<UserSigning>>>,
    pub(crate) self_signing_key: Arc<Mutex<Option<SelfSigning>>>,
}

/// A struct containing information if any of our cross signing keys were
/// cleared because the public keys differ from the keys that are uploaded to
/// the server.
#[derive(Debug, Clone)]
pub struct ClearResult {
    /// Was the master key cleared.
    master_cleared: bool,
    /// Was the self-signing key cleared.
    self_signing_cleared: bool,
    /// Was the user-signing key cleared.
    user_signing_cleared: bool,
}

impl ClearResult {
    /// Did we clear any of the private cross signing keys.
    pub fn any_cleared(&self) -> bool {
        self.master_cleared || self.self_signing_cleared || self.user_signing_cleared
    }
}

/// The pickled version of a `PrivateCrossSigningIdentity`.
///
/// Can be used to store the identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickledCrossSigningIdentity {
    /// The user id of the identity owner.
    pub user_id: Box<UserId>,
    /// Have the public keys of the identity been shared.
    pub shared: bool,
    /// The encrypted pickle of the identity.
    pub pickle: String,
}

/// Struct representing the state of our private cross signing keys, it shows
/// which private cross signing keys we have locally stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSigningStatus {
    /// Do we have the master key.
    pub has_master: bool,
    /// Do we have the self signing key, this one is necessary to sign our own
    /// devices.
    pub has_self_signing: bool,
    /// Do we have the user signing key, this one is necessary to sign other
    /// users.
    pub has_user_signing: bool,
}

impl PrivateCrossSigningIdentity {
    /// Get the user id that this identity belongs to.
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Is the identity empty.
    ///
    /// An empty identity doesn't contain any private keys.
    ///
    /// It is usual for the identity not to contain the master key since the
    /// master key is only needed to sign the subkeys.
    ///
    /// An empty identity indicates that either no identity was created for this
    /// use or that another device created it and hasn't shared it yet with us.
    pub async fn is_empty(&self) -> bool {
        let has_master = self.master_key.lock().await.is_some();
        let has_user = self.user_signing_key.lock().await.is_some();
        let has_self = self.self_signing_key.lock().await.is_some();

        !(has_master && has_user && has_self)
    }

    /// Can we sign our own devices, i.e. do we have a self signing key.
    pub async fn can_sign_devices(&self) -> bool {
        self.self_signing_key.lock().await.is_some()
    }

    /// Can we sign other users, i.e. do we have a user signing key.
    pub async fn can_sign_users(&self) -> bool {
        self.user_signing_key.lock().await.is_some()
    }

    /// Do we have the master key.
    pub async fn has_master_key(&self) -> bool {
        self.master_key.lock().await.is_some()
    }

    /// Get the status of our private cross signing keys, i.e. if we have the
    /// master key and the subkeys.
    pub async fn status(&self) -> CrossSigningStatus {
        CrossSigningStatus {
            has_master: self.has_master_key().await,
            has_self_signing: self.can_sign_devices().await,
            has_user_signing: self.can_sign_users().await,
        }
    }

    /// Get the public part of the master key, if we have one.
    pub async fn master_public_key(&self) -> Option<MasterPubkey> {
        self.master_key.lock().await.as_ref().map(|m| m.public_key.to_owned())
    }

    /// Get the public part of the self-signing key, if we have one.
    pub async fn self_signing_public_key(&self) -> Option<SelfSigningPubkey> {
        self.self_signing_key.lock().await.as_ref().map(|k| k.public_key.to_owned())
    }

    /// Get the public part of the user-signing key, if we have one.
    pub async fn user_signing_public_key(&self) -> Option<UserSigningPubkey> {
        self.user_signing_key.lock().await.as_ref().map(|k| k.public_key.to_owned())
    }

    /// Export the seed of the private cross signing key
    ///
    /// The exported seed will be encoded as unpadded base64.
    ///
    /// # Arguments
    ///
    /// * `secret_name` - The type of the cross signing key that should be
    /// exported.
    pub async fn export_secret(&self, secret_name: &SecretName) -> Option<String> {
        match secret_name {
            SecretName::CrossSigningMasterKey => {
                self.master_key.lock().await.as_ref().map(|m| m.export_seed())
            }
            SecretName::CrossSigningUserSigningKey => {
                self.user_signing_key.lock().await.as_ref().map(|m| m.export_seed())
            }
            SecretName::CrossSigningSelfSigningKey => {
                self.self_signing_key.lock().await.as_ref().map(|m| m.export_seed())
            }
            _ => None,
        }
    }

    pub(crate) async fn import_secret(
        &self,
        public_identity: OwnUserIdentity,
        secret_name: &SecretName,
        seed: &str,
    ) -> Result<(), SecretImportError> {
        let (master, self_signing, user_signing) = match secret_name {
            SecretName::CrossSigningMasterKey => (Some(seed), None, None),
            SecretName::CrossSigningSelfSigningKey => (None, Some(seed), None),
            SecretName::CrossSigningUserSigningKey => (None, None, Some(seed)),
            _ => return Ok(()),
        };

        self.import_secrets(public_identity, master, self_signing, user_signing).await
    }

    pub(crate) async fn import_secrets(
        &self,
        public_identity: OwnUserIdentity,
        master_key: Option<&str>,
        self_signing_key: Option<&str>,
        user_signing_key: Option<&str>,
    ) -> Result<(), SecretImportError> {
        let master = if let Some(master_key) = master_key {
            let seed = decode(master_key)?;

            let master = MasterSigning::from_seed(self.user_id().to_owned(), seed);

            if public_identity.master_key() == &master.public_key {
                Ok(Some(master))
            } else {
                Err(SecretImportError::MismatchedPublicKeys)
            }
        } else {
            Ok(None)
        }?;

        let user_signing = if let Some(user_signing_key) = user_signing_key {
            let seed = decode(user_signing_key)?;
            let subkey = UserSigning::from_seed(self.user_id().to_owned(), seed);

            if public_identity.user_signing_key() == &subkey.public_key {
                Ok(Some(subkey))
            } else {
                Err(SecretImportError::MismatchedPublicKeys)
            }
        } else {
            Ok(None)
        }?;

        let self_signing = if let Some(self_signing_key) = self_signing_key {
            let seed = decode(self_signing_key)?;
            let subkey = SelfSigning::from_seed(self.user_id().to_owned(), seed);

            if public_identity.self_signing_key() == &subkey.public_key {
                Ok(Some(subkey))
            } else {
                Err(SecretImportError::MismatchedPublicKeys)
            }
        } else {
            Ok(None)
        }?;

        if let Some(master) = master {
            *self.master_key.lock().await = Some(master);
        }

        if let Some(self_signing) = self_signing {
            *self.self_signing_key.lock().await = Some(self_signing);
        }

        if let Some(user_signing) = user_signing {
            *self.user_signing_key.lock().await = Some(user_signing);
        }

        Ok(())
    }

    /// Remove our private cross signing key if the public keys differ from
    /// what's found in the `ReadOnlyOwnUserIdentity`.
    pub(crate) async fn clear_if_differs(
        &self,
        public_identity: &ReadOnlyOwnUserIdentity,
    ) -> ClearResult {
        let result = self.get_public_identity_diff(public_identity).await;

        if result.master_cleared {
            *self.master_key.lock().await = None;
        }

        if result.user_signing_cleared {
            *self.user_signing_key.lock().await = None;
        }

        if result.self_signing_cleared {
            *self.self_signing_key.lock().await = None;
        }

        result
    }

    async fn get_public_identity_diff(
        &self,
        public_identity: &ReadOnlyOwnUserIdentity,
    ) -> ClearResult {
        let master_differs = self
            .master_public_key()
            .await
            .map(|master| &master != public_identity.master_key())
            .unwrap_or(false);

        let user_signing_differs = self
            .user_signing_public_key()
            .await
            .map(|subkey| &subkey != public_identity.user_signing_key())
            .unwrap_or(false);

        let self_signing_differs = self
            .self_signing_public_key()
            .await
            .map(|subkey| &subkey != public_identity.self_signing_key())
            .unwrap_or(false);

        ClearResult {
            master_cleared: master_differs,
            user_signing_cleared: user_signing_differs,
            self_signing_cleared: self_signing_differs,
        }
    }

    /// Get the names of the secrets we are missing.
    pub(crate) async fn get_missing_secrets(&self) -> Vec<SecretName> {
        let mut missing = Vec::new();

        if !self.has_master_key().await {
            missing.push(SecretName::CrossSigningMasterKey);
        }

        if !self.can_sign_devices().await {
            missing.push(SecretName::CrossSigningSelfSigningKey);
        }

        if !self.can_sign_users().await {
            missing.push(SecretName::CrossSigningUserSigningKey);
        }

        missing
    }

    /// Create a new empty identity.
    pub(crate) fn empty(user_id: Box<UserId>) -> Self {
        Self {
            user_id: user_id.into(),
            shared: Arc::new(AtomicBool::new(false)),
            master_key: Arc::new(Mutex::new(None)),
            self_signing_key: Arc::new(Mutex::new(None)),
            user_signing_key: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) async fn to_public_identity(
        &self,
    ) -> Result<ReadOnlyOwnUserIdentity, SignatureError> {
        let master = self
            .master_key
            .lock()
            .await
            .as_ref()
            .ok_or(SignatureError::MissingSigningKey)?
            .public_key
            .clone();

        let self_signing = self
            .self_signing_key
            .lock()
            .await
            .as_ref()
            .ok_or(SignatureError::MissingSigningKey)?
            .public_key
            .clone();

        let user_signing = self
            .user_signing_key
            .lock()
            .await
            .as_ref()
            .ok_or(SignatureError::MissingSigningKey)?
            .public_key
            .clone();

        let identity = ReadOnlyOwnUserIdentity::new(master, self_signing, user_signing)?;
        identity.mark_as_verified();

        Ok(identity)
    }

    /// Sign the given public user identity with this private identity.
    pub(crate) async fn sign_user(
        &self,
        user_identity: &ReadOnlyUserIdentity,
    ) -> Result<SignatureUploadRequest, SignatureError> {
        let master_key = self
            .user_signing_key
            .lock()
            .await
            .as_ref()
            .ok_or(SignatureError::MissingSigningKey)?
            .sign_user(user_identity)
            .await?;

        let mut signed_keys = BTreeMap::new();

        signed_keys.entry(user_identity.user_id().to_owned()).or_insert_with(BTreeMap::new).insert(
            user_identity
                .master_key()
                .get_first_key()
                .ok_or(SignatureError::MissingSigningKey)?
                .to_owned(),
            serde_json::to_value(master_key)?,
        );

        Ok(SignatureUploadRequest::new(signed_keys))
    }

    /// Sign the given device keys with this identity.
    pub(crate) async fn sign_device(
        &self,
        device: &ReadOnlyDevice,
    ) -> Result<SignatureUploadRequest, SignatureError> {
        let mut device_keys = device.as_device_keys().to_owned();
        device_keys.signatures.clear();
        self.sign_device_keys(&mut device_keys).await
    }

    /// Sign an Olm account with this private identity.
    pub(crate) async fn sign_account(
        &self,
        account: &ReadOnlyAccount,
    ) -> Result<SignatureUploadRequest, SignatureError> {
        let mut device_keys = account.unsigned_device_keys();
        self.sign_device_keys(&mut device_keys).await
    }

    pub(crate) async fn sign_device_keys(
        &self,
        device_keys: &mut DeviceKeys,
    ) -> Result<SignatureUploadRequest, SignatureError> {
        self.self_signing_key
            .lock()
            .await
            .as_ref()
            .ok_or(SignatureError::MissingSigningKey)?
            .sign_device(device_keys)
            .await?;

        let mut signed_keys = BTreeMap::new();
        signed_keys
            .entry((&*self.user_id).to_owned())
            .or_insert_with(BTreeMap::new)
            .insert(device_keys.device_id.to_string(), serde_json::to_value(device_keys)?);

        Ok(SignatureUploadRequest::new(signed_keys))
    }

    pub(crate) async fn sign(&self, message: &str) -> Result<String, SignatureError> {
        Ok(self
            .master_key
            .lock()
            .await
            .as_ref()
            .ok_or(SignatureError::MissingSigningKey)?
            .sign(message)
            .await)
    }

    /// Create a new identity for the given Olm Account.
    ///
    /// Returns the new identity, the upload signing keys request and a
    /// signature upload request that contains the signature of the account
    /// signed by the self signing key.
    ///
    /// # Arguments
    ///
    /// * `account` - The Olm account that is creating the new identity. The
    /// account will sign the master key and the self signing key will sign the
    /// account.
    pub(crate) async fn new_with_account(
        account: &ReadOnlyAccount,
    ) -> (Self, UploadSigningKeysRequest, SignatureUploadRequest) {
        let master = Signing::new();

        let mut public_key =
            master.cross_signing_key(account.user_id().to_owned(), KeyUsage::Master);

        account
            .sign_cross_signing_key(&mut public_key)
            .await
            .expect("Can't sign our freshly created master key with our account");

        let master = MasterSigning { inner: master, public_key: public_key.into() };

        let identity = Self::new_helper(account.user_id(), master).await;
        let signature_request = identity
            .sign_account(account)
            .await
            .expect("Can't sign own device with new cross signign keys");

        let request = identity.as_upload_request().await;

        (identity, request, signature_request)
    }

    async fn new_helper(user_id: &UserId, master: MasterSigning) -> Self {
        let user = Signing::new();
        let mut public_key = user.cross_signing_key(user_id.to_owned(), KeyUsage::UserSigning);
        master.sign_subkey(&mut public_key).await;

        let user = UserSigning { inner: user, public_key: public_key.into() };

        let self_signing = Signing::new();
        let mut public_key =
            self_signing.cross_signing_key(user_id.to_owned(), KeyUsage::SelfSigning);
        master.sign_subkey(&mut public_key).await;

        let self_signing = SelfSigning { inner: self_signing, public_key: public_key.into() };

        Self {
            user_id: user_id.into(),
            shared: Arc::new(AtomicBool::new(false)),
            master_key: Arc::new(Mutex::new(Some(master))),
            self_signing_key: Arc::new(Mutex::new(Some(self_signing))),
            user_signing_key: Arc::new(Mutex::new(Some(user))),
        }
    }

    /// Create a new cross signing identity without signing the device that
    /// created it.
    #[cfg(test)]
    pub(crate) async fn new(user_id: Box<UserId>) -> Self {
        let master = Signing::new();

        let public_key = master.cross_signing_key(user_id.clone(), KeyUsage::Master);
        let master = MasterSigning { inner: master, public_key: public_key.into() };

        Self::new_helper(&user_id, master).await
    }

    #[cfg(test)]
    pub(crate) async fn reset(&mut self) {
        let new = Self::new(self.user_id().to_owned()).await;
        *self = new
    }

    /// Mark the identity as shared.
    pub fn mark_as_shared(&self) {
        self.shared.store(true, Ordering::SeqCst)
    }

    /// Has the identity been shared.
    ///
    /// A shared identity here means that the public keys of the identity have
    /// been uploaded to the server.
    pub fn shared(&self) -> bool {
        self.shared.load(Ordering::SeqCst)
    }

    /// Store the cross signing identity as a pickle.
    ///
    /// # Arguments
    ///
    /// * `pickle_key` - The key that should be used to encrypt the signing
    /// object, must be 32 bytes long.
    ///
    /// # Panics
    ///
    /// This will panic if the provided pickle key isn't 32 bytes long.
    pub async fn pickle(
        &self,
        pickle_key: &[u8],
    ) -> Result<PickledCrossSigningIdentity, JsonError> {
        let master_key = if let Some(m) = self.master_key.lock().await.as_ref() {
            Some(m.pickle(pickle_key).await)
        } else {
            None
        };

        let self_signing_key = if let Some(m) = self.self_signing_key.lock().await.as_ref() {
            Some(m.pickle(pickle_key).await)
        } else {
            None
        };

        let user_signing_key = if let Some(m) = self.user_signing_key.lock().await.as_ref() {
            Some(m.pickle(pickle_key).await)
        } else {
            None
        };

        let pickle = PickledSignings { master_key, user_signing_key, self_signing_key };

        let pickle = serde_json::to_string(&pickle)?;

        Ok(PickledCrossSigningIdentity {
            user_id: self.user_id.as_ref().to_owned(),
            shared: self.shared(),
            pickle,
        })
    }

    /// Restore the private cross signing identity from a pickle.
    ///
    /// # Panic
    ///
    /// Panics if the pickle_key isn't 32 bytes long.
    pub async fn from_pickle(
        pickle: PickledCrossSigningIdentity,
        pickle_key: &[u8],
    ) -> Result<Self, SigningError> {
        let signings: PickledSignings = serde_json::from_str(&pickle.pickle)?;

        let master =
            signings.master_key.map(|m| MasterSigning::from_pickle(m, pickle_key)).transpose()?;

        let self_signing = signings
            .self_signing_key
            .map(|s| SelfSigning::from_pickle(s, pickle_key))
            .transpose()?;

        let user_signing = signings
            .user_signing_key
            .map(|s| UserSigning::from_pickle(s, pickle_key))
            .transpose()?;

        Ok(Self {
            user_id: pickle.user_id.into(),
            shared: Arc::new(AtomicBool::from(pickle.shared)),
            master_key: Arc::new(Mutex::new(master)),
            self_signing_key: Arc::new(Mutex::new(self_signing)),
            user_signing_key: Arc::new(Mutex::new(user_signing)),
        })
    }

    /// Get the upload request that is needed to share the public keys of this
    /// identity.
    pub(crate) async fn as_upload_request(&self) -> UploadSigningKeysRequest {
        let master_key =
            self.master_key.lock().await.as_ref().map(|k| k.public_key.to_owned().into());

        let user_signing_key =
            self.user_signing_key.lock().await.as_ref().map(|k| k.public_key.to_owned().into());

        let self_signing_key =
            self.self_signing_key.lock().await.as_ref().map(|k| k.public_key.to_owned().into());

        UploadSigningKeysRequest { master_key, self_signing_key, user_signing_key }
    }
}

#[cfg(test)]
mod test {
    use matrix_sdk_test::async_test;
    use ruma::{device_id, user_id, UserId};

    use super::{PrivateCrossSigningIdentity, Signing};
    use crate::{
        identities::{ReadOnlyDevice, ReadOnlyUserIdentity},
        olm::ReadOnlyAccount,
    };

    fn user_id() -> &'static UserId {
        user_id!("@example:localhost")
    }

    fn pickle_key() -> &'static [u8] {
        &[0u8; 32]
    }

    #[test]
    fn signing_creation() {
        let signing = Signing::new();
        assert!(!signing.public_key().as_str().is_empty());
    }

    #[async_test]
    async fn signature_verification() {
        let signing = Signing::new();

        let message = "Hello world";

        let signature = signing.sign(message).await;
        assert!(signing.verify(message, &signature).await.is_ok());
    }

    #[async_test]
    async fn pickling_signing() {
        let signing = Signing::new();
        let pickled = signing.pickle(pickle_key()).await;

        let unpickled = Signing::from_pickle(pickled, pickle_key()).unwrap();

        assert_eq!(signing.public_key(), unpickled.public_key());
    }

    #[async_test]
    async fn private_identity_creation() {
        let identity = PrivateCrossSigningIdentity::new(user_id().to_owned()).await;

        let master_key = identity.master_key.lock().await;
        let master_key = master_key.as_ref().unwrap();

        assert!(master_key
            .public_key
            .verify_subkey(&identity.self_signing_key.lock().await.as_ref().unwrap().public_key,)
            .is_ok());

        assert!(master_key
            .public_key
            .verify_subkey(&identity.user_signing_key.lock().await.as_ref().unwrap().public_key,)
            .is_ok());
    }

    #[async_test]
    async fn identity_pickling() {
        let identity = PrivateCrossSigningIdentity::new(user_id().to_owned()).await;

        let pickled = identity.pickle(pickle_key()).await.unwrap();

        let unpickled =
            PrivateCrossSigningIdentity::from_pickle(pickled, pickle_key()).await.unwrap();

        assert_eq!(identity.user_id, unpickled.user_id);
        assert_eq!(&*identity.master_key.lock().await, &*unpickled.master_key.lock().await);
        assert_eq!(
            &*identity.user_signing_key.lock().await,
            &*unpickled.user_signing_key.lock().await
        );
        assert_eq!(
            &*identity.self_signing_key.lock().await,
            &*unpickled.self_signing_key.lock().await
        );
    }

    #[async_test]
    async fn private_identity_signed_by_account() {
        let account = ReadOnlyAccount::new(user_id(), device_id!("DEVICEID"));
        let (identity, _, _) = PrivateCrossSigningIdentity::new_with_account(&account).await;
        let master = identity.master_key.lock().await;
        let master = master.as_ref().unwrap();

        assert!(!master.public_key.signatures().is_empty());
    }

    #[async_test]
    async fn sign_device() {
        let account = ReadOnlyAccount::new(user_id(), device_id!("DEVICEID"));
        let (identity, _, _) = PrivateCrossSigningIdentity::new_with_account(&account).await;

        let mut device = ReadOnlyDevice::from_account(&account).await;
        let self_signing = identity.self_signing_key.lock().await;
        let self_signing = self_signing.as_ref().unwrap();

        let mut device_keys = device.as_device_keys().to_owned();
        self_signing.sign_device(&mut device_keys).await.unwrap();
        device.update_device(&device_keys).unwrap();

        let public_key = &self_signing.public_key;
        public_key.verify_device(&device).unwrap()
    }

    #[async_test]
    async fn sign_user_identity() {
        let account = ReadOnlyAccount::new(user_id(), device_id!("DEVICEID"));
        let (identity, _, _) = PrivateCrossSigningIdentity::new_with_account(&account).await;

        let bob_account = ReadOnlyAccount::new(user_id!("@bob:localhost"), device_id!("DEVICEID"));
        let (bob_private, _, _) = PrivateCrossSigningIdentity::new_with_account(&bob_account).await;
        let mut bob_public = ReadOnlyUserIdentity::from_private(&bob_private).await;

        let user_signing = identity.user_signing_key.lock().await;
        let user_signing = user_signing.as_ref().unwrap();

        let master = user_signing.sign_user(&bob_public).await.unwrap();

        let num_signatures: usize = master.signatures.iter().map(|(_, u)| u.len()).sum();
        assert_eq!(num_signatures, 1, "We're only uploading our own signature");

        bob_public.master_key = master.into();

        user_signing.public_key.verify_master_key(bob_public.master_key()).unwrap();
    }
}