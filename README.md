# Rusty

## Android
### Requirements
```bash
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi \
  x86_64-linux-android
```

### Generate .so
```bash
make so
```

### Generate kotlin
```bash
make kotlin
```

## Implementation
### Authentication
```rust
// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
let cipher_token = v1_token_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
let token_hash = v1_token_decrypt(...)
```

### OAuth URL
```rust
// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
let ciphertext_url = v1_oauth_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
let url = v1_oauth_decrypt(...)
```

### Platform publisher
```rust
// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
let ciphertext = v1_platform_publisher_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
let payload = v1_platform_publisher_decrypt(...)
```

### Bridge publisher (online first)
```rust
// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
let ciphertext = v1_bridge_online_first_publisher_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
let request = v1_bridge_online_first_publisher_decrypt(...)
```

### Bridge publisher (offline first)
```rust
// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
// Returns OfflineFirstEncryptionResponse {
//   tx_payload: bytes, // encrypted payload
//   sc_pk_enc: bytes, // encrypted long term identity key
//   h: bytes //MixHash value
// }
let ciphertext = v1_bridge_offline_first_publisher_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
// Returns OfflineFirstDecryptionResponse {
//   payload: bytes, // decrypted payload
//   h: bytes //MixHash value
// }
let response = v1_bridge_offline_first_publisher_decrypt(...)
```

### Publishing (with Attachments)
```rust

// example message
 let contents = V1ContentsContainer(
      V1ContentCategories::Message,
      body,
      to,
      subject,
      attachment
  );

  let payload_att = V1Payloads(
      contents.content_from(),
      k_id,
      len_att,
      t_id,
      sess_id
  );

// For sending
let split = payload_att.split(Arc::new(SMS)).unwrap();

// Receiving
let joined = V1Payloads::join(split, V1ContentCategories::Message).unwrap();
```
