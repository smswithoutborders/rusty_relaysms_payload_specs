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
cipher_token = v1_token_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
token_hash = v1_token_decrypt(...)
```

### OAuth URL
```rust
// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
ciphertext_url = v1_oauth_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
url = v1_oauth_decrypt(...)
```

### Publisher
```rust
// Get token
// Returns `FailedToEncrypt` in cases cannot decrypt
ciphertext = v1_platform_publisher_encrypt(...)

// Verify token
// Returns `FailedToDecrypt` in cases cannot decrypt
payload = v1_platform_publisher_decrypt(...)
```
