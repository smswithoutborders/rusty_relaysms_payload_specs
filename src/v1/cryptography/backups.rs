use std::sync::Arc;
use aead::Aead;
use argon2::Argon2;
use argon2::password_hash::SaltString;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use rand::{rng, RngExt};
use rand::distr::Alphanumeric;
use serde::{Deserialize, Serialize};
use crate::v1::cryptography::V1CryptographicError;

const BACKUP_SALT: &[u8] = b"RelaySMS Export backup";

#[derive(PartialEq, Debug, uniffi::Object, Serialize, Deserialize)]
pub struct BackupRestore {
    nonce: Vec<u8>,
    recovery_key: Option<Vec<u8>>,
    ciphertext: Option<Vec<u8>>,
}


#[uniffi::export]
impl BackupRestore {
    fn get_nonce(&self) -> Vec<u8> { self.nonce.clone() }
    fn get_recovery_key(&self) -> Option<Vec<u8>> { self.recovery_key.clone() }
    fn get_ciphertext(&self) -> Option<Vec<u8>> { self.ciphertext.clone() }

    #[uniffi::constructor]
    fn v1_backup_encrypt(data: &[u8]) -> Result<Arc<BackupRestore>, V1CryptographicError>{
        let recovery_key: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            // 1. Filter the INFINITE stream first for UpperCase OR Digits
            .filter(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            // 2. Now take exactly 64 characters from that filtered stream
            .take(64)
            .collect();

        let recovery_key = recovery_key.as_bytes();

        let rng: [u8; 12] = rand::rng().random();
        let nonce = Nonce::try_from(rng).unwrap();

        // TODO: make sure it's hard to crack
        let mut chacha_key = [0u8; 32];
        Argon2::default().hash_password_into(recovery_key, BACKUP_SALT, &mut chacha_key)
            .expect("Should be able to hash the digest list");

        let cipher = ChaCha20Poly1305::new_from_slice(&chacha_key)
            .expect("Aes256Gcm::new_from_slice failed");

        match cipher.encrypt(&nonce, data) {
            Ok(ciphertext) => Ok(Arc::new(BackupRestore {
                recovery_key: Some(recovery_key.to_vec()),
                nonce: nonce.to_vec(),
                ciphertext: Some(ciphertext),
            })),
            Err(e) => Err(V1CryptographicError::FailedToEncrypt { err: e.to_string() })
        }
    }

    fn v1_restore_decrypt(&self) -> Result<Vec<u8>, V1CryptographicError> {
        if self.recovery_key.is_none() {
            return Err(V1CryptographicError::NoRecoveryKeyFound)
        }

        if self.ciphertext.is_none() {
            return Err(V1CryptographicError::CiphertextEmpty)
        }

        let mut chacha_key = [0u8; 32];
        Argon2::default()
            .hash_password_into(
                self.recovery_key.clone().unwrap().as_ref(),
                BACKUP_SALT,
                &mut chacha_key
            ).expect("Should be able to hash the digest list");

        let cipher = ChaCha20Poly1305::new_from_slice(&chacha_key)
            .expect("Aes256Gcm::new_from_slice failed");

        let nonce = Nonce::try_from(self.nonce.as_slice()).unwrap();
        match cipher.decrypt(&nonce, self.ciphertext.as_ref().unwrap().as_slice()) {
            Ok(ciphertext) => Ok(ciphertext),
            Err(e) => Err(V1CryptographicError::FailedToEncrypt { err: e.to_string() })
        }
    }

    fn serialize(&self) -> Result<Vec<u8>, V1CryptographicError> {
        if self.ciphertext.is_none() {
            return Err(V1CryptographicError::CiphertextEmpty)
        }
        Ok([self.nonce.clone(), self.ciphertext.clone().unwrap().to_vec()].concat())
    }

    // I Hate this
    #[uniffi::constructor]
    fn deserialize(data: &[u8], recovery_key: Option<Vec<u8>>) -> Result<BackupRestore, V1CryptographicError> {
        let nonce = data[..12].to_vec();
        let ciphertext = Some(data[12..].to_vec());
        Ok(BackupRestore {
            recovery_key,
            nonce,
            ciphertext
        })
    }
}


#[test]
fn v1_backup_test() {
    let rng: [u8; 32] = rand::rng().random();
    let backup_restore = BackupRestore::v1_backup_encrypt(rng.as_slice()).unwrap();
    let br = BackupRestore {
        recovery_key: backup_restore.recovery_key.clone(),
        nonce: backup_restore.nonce.clone(),
        ciphertext: backup_restore.ciphertext.clone(),
    };
    let plaintext = br.v1_restore_decrypt().unwrap();

    assert_eq!(rng.as_slice(), plaintext);
}