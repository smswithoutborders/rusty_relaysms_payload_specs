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
_ = v1_token_decrypt(...)
```
