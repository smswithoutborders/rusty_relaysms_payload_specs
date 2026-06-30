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
### GRPC Requests
```rust
struct RequestPayload {
    pub ciphertext: Vec<u8>,
    pub timestamp: u64
}

// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
let payload: RequesetPayload = v1_requests_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
/* struct ResponsePayload {
    pub method_name: Vec<u8>,
    pub payload: Vec<u8>,
} */
let _ = v1_requests_decrypt(payload.ciphertext)
```

### Tokens
```rust
// Server
// Returns `FailedToEncrypt` in cases cannot decrypt
let ciphertext = v1_token_encrypt_server(...)

// Returns `FailedToDecrypt` in cases cannot decrypt
let token = v1_token_decrypt_server(...)
```

```rust
// Client
// Returns `FailedToEncrypt` in cases cannot decrypt
let ciphertext = v1_token_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
let token = v1_token_decrypt(...)
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

### Publishing (without Attachments)
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
      contents.serialize(),
      k_id,
      len_att,
      t_id,
      sess_id
  );

// For sending
let payload = payload_att.serialize_without_attachment().unwrap();

// For receiving
let t = V1Payloads::get_types(payload)
if t == V1PayloadsTypes::WithoutAttachment {
  let v1_payload = V1Payloads::deserialize_without_attachment();
  let content = v1_platform_publisher_decrypt(v1_payload.get_content())
  let v1_content_container = V1ContentContainer::deserialize(content);
}
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
      contents.serialize(),
      k_id,
      len_att,
      t_id,
      sess_id
  );

// For sending
let payloads: [] = payload_att.split(Transports::Sms).unwrap();

// Receiving
for payload in payloads {

  if t == V1PayloadsTypes::WithAttachmentHeader {
    // first segment with attachment
    // session id
    let session_id = V1PayloadsTypes::get_session_id(payload)
    ...
  }
  
  else if t == V1PayloadsTypes::WithAttachmentNoHeader {
    // nth segment with attachment
    // session id
    let session_id = V1PayloadsTypes::get_session_id(payload)
    ...
  }
}

...
// for all session payload
let v1_payload = V1PayloadsTypes::join([payloads])
let content = v1_platform_publisher_decrypt(v1_payload.get_content())
let v1_content_container: V1ContentContainer = V1ContentContainer::deserialize(content);
```

