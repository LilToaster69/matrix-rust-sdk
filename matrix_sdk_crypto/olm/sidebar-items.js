initSidebarItems({"enum":[["PicklingMode","Used for setting the encryption parameter for pickling (serialisation) functions. `Unencrypted` is functionally equivalent to `Encrypted{key: [].to_vec() }`, but is much more clear. Pickling modes have to be equivalent for pickling and unpickling operations to succeed. `Encrypted` takes ownership of `key`, in order to properly destroy it after use."]],"struct":[["AccountPickle","A typed representation of a base64 encoded string containing the account pickle."],["CrossSigningStatus","Struct representing the state of our private cross signing keys, it shows which private cross signing keys we have locally stored."],["EncryptionSettings","Settings for an encrypted room."],["ExportedRoomKey","An exported version of an `InboundGroupSession`"],["IdentityKeys","Struct representing the parsed result of [`OlmAccount::identity_keys()`]."],["InboundGroupSession","Inbound group session."],["InboundGroupSessionPickle","The typed representation of a base64 encoded string of the GroupSession pickle."],["OlmMessageHash","A hash of a successfully decrypted Olm message."],["OutboundGroupSession","Outbound group session."],["PickledAccount","A pickled version of an `Account`."],["PickledCrossSigningIdentity","The pickled version of a `PrivateCrossSigningIdentity`."],["PickledInboundGroupSession","A pickled version of an `InboundGroupSession`."],["PickledOutboundGroupSession","A pickled version of an `InboundGroupSession`."],["PickledSession","A pickled version of a `Session`."],["PrivateCrossSigningIdentity","Private cross signing identity."],["ReadOnlyAccount","Account holding identity keys for which sessions can be created."],["Session","Cryptographic session that enables secure communication between two `Account`s"],["SessionPickle","The typed representation of a base64 encoded string of the Olm Session pickle."],["ShareInfo","Struct holding info about the share state of a outbound group session."]]});