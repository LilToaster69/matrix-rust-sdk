//! Responses to client API calls.

use once_cell::sync::Lazy;
use serde_json::{json, Value as JsonValue};

/// `GET /_matrix/client/v3/devices`
pub static DEVICES: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "devices": [
            {
                "device_id": "BNYQQWUMXO",
                "display_name": "Client 1",
                "last_seen_ip": "-",
                "last_seen_ts": 1596117733037u64,
                "user_id": "@example:localhost"
            },
            {
                "device_id": "LEBKSEUSNR",
                "display_name": "Client 2",
                "last_seen_ip": "-",
                "last_seen_ts": 1599057006985u64,
                "user_id": "@example:localhost"
            }
        ]
    })
});

/// `GET /_matrix/client/v3/directory/room/{roomAlias}`
pub static GET_ALIAS: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "room_id": "!lUbmUPdxdXxEQurqOs:example.net",
        "servers": [
          "example.org",
          "example.net",
          "matrix.org",
        ]
    })
});

/// `POST /_matrix/client/v3/keys/query`
pub static KEYS_QUERY: Lazy<JsonValue> = Lazy::new(|| {
    json!({
      "device_keys": {
        "@alice:example.org": {
          "JLAFKJWSCS": {
              "algorithms": [
                  "m.olm.v1.curve25519-aes-sha2",
                  "m.megolm.v1.aes-sha2"
              ],
              "device_id": "JLAFKJWSCS",
              "user_id": "@alice:example.org",
              "keys": {
                  "curve25519:JLAFKJWSCS": "wjLpTLRqbqBzLs63aYaEv2Boi6cFEbbM/sSRQ2oAKk4",
                  "ed25519:JLAFKJWSCS": "nE6W2fCblxDcOFmeEtCHNl8/l8bXcu7GKyAswA4r3mM"
              },
              "signatures": {
                  "@alice:example.org": {
                      "ed25519:JLAFKJWSCS": "m53Wkbh2HXkc3vFApZvCrfXcX3AI51GsDHustMhKwlv3TuOJMj4wistcOTM8q2+e/Ro7rWFUb9ZfnNbwptSUBA"
                  }
              },
              "unsigned": {
                  "device_display_name": "Alice's mobile phone"
              }
          }
        }
      },
      "failures": {}
    })
});

/// ``
pub static KEYS_UPLOAD: Lazy<JsonValue> = Lazy::new(|| {
    json!({
      "one_time_key_counts": {
        "curve25519": 10,
        "signed_curve25519": 20
      }
    })
});

/// Successful call to `POST /_matrix/client/v3/login` without auto-discovery.
pub static LOGIN: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "access_token": "abc123",
        "device_id": "GHTYAJCE",
        "home_server": "matrix.org",
        "user_id": "@cheeky_monkey:matrix.org"
    })
});

/// Successful call to `POST /_matrix/client/v3/login` with auto-discovery.
pub static LOGIN_WITH_DISCOVERY: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "access_token": "abc123",
        "device_id": "GHTYAJCE",
        "home_server": "matrix.org",
        "user_id": "@cheeky_monkey:matrix.org",
        "well_known": {
            "m.homeserver": {
                "base_url": "https://example.org"
            },
            "m.identity_server": {
                "base_url": "https://id.example.org"
            }
        }
    })
});

/// Failed call to `POST /_matrix/client/v3/login`
pub static LOGIN_RESPONSE_ERR: Lazy<JsonValue> = Lazy::new(|| {
    json!({
      "errcode": "M_FORBIDDEN",
      "error": "Invalid password"
    })
});

/// `GET /_matrix/client/v3/login`
pub static LOGIN_TYPES: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "flows": [
            {
                "type": "m.login.password"
            },
            {
                "type": "m.login.sso"
            },
            {
                "type": "m.login.token"
            }
        ]
    })
});

/// `GET /_matrix/client/v3/publicRooms`
pub static PUBLIC_ROOMS: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "chunk": [
            {
                "aliases": [
                    "#murrays:cheese.bar"
                ],
                "avatar_url": "mxc://bleeker.street/CHEDDARandBRIE",
                "guest_can_join": false,
                "name": "CHEESE",
                "num_joined_members": 37,
                "room_id": "!ol19s:bleecker.street",
                "topic": "Tasty tasty cheese",
                "world_readable": true
            }
        ],
        "next_batch": "p190q",
        "prev_batch": "p1902",
        "total_room_count_estimate": 115
    })
});

/// Failed call to `POST /_matrix/client/v3/register`
pub static REGISTRATION_RESPONSE_ERR: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "errcode": "M_FORBIDDEN",
        "error": "Invalid password",
        "completed": ["example.type.foo"],
        "flows": [
            {
                "stages": ["example.type.foo", "example.type.bar"]
            },
            {
                "stages": ["example.type.foo", "example.type.baz"]
            }
        ],
        "params": {
            "example.type.baz": {
                "example_key": "foobar"
            }
        },
        "session": "xxxxxx"
    })
});

/// `GET /_matrix/client/versions`
pub static VERSIONS: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "versions": [
            "r0.0.1",
            "r0.1.0",
            "r0.2.0",
            "r0.3.0",
            "r0.4.0",
            "r0.5.0",
            "r0.6.0"
        ],
        "unstable_features": {
            "org.matrix.label_based_filtering":true,
            "org.matrix.e2e_cross_signing":true
        }
    })
});

/// `GET /.well-known/matrix/client`
pub static WELL_KNOWN: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "m.homeserver": {
            "base_url": "HOMESERVER_URL"
        }
    })
});

/// `GET /_matrix/client/v3/account/whoami`
pub static WHOAMI: Lazy<JsonValue> = Lazy::new(|| {
    json!({
        "user_id": "@joe:example.org"
    })
});
