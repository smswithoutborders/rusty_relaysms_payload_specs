use std::sync::Arc;
use aead::Aead;
use argon2::Argon2;
use argon2::password_hash::SaltString;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use rand::{rng, RngExt};
use serde::{Deserialize, Serialize};
use crate::v1::cryptography::V1CryptographicError;

const BACKUP_SALT: &[u8] = b"RelaySMS Export backup";

#[derive(PartialEq, Debug, uniffi::Object, Serialize, Deserialize)]
pub struct BackupRestore {
    digit_list: Vec<u8>,
    nonce: Vec<u8>,
    ciphertext: Option<Vec<u8>>,
}


#[uniffi::export]
impl BackupRestore {
    #[uniffi::constructor]
    fn v1_backup_encrypt(data: &[u8]) -> Result<Arc<BackupRestore>, V1CryptographicError>{
        let mut rng = rand::rng();

        let digit_list: Vec<u8> = (0..30)
            .map(|_| rng.random_range(0..10))
            .collect();

        let rng: [u8; 12] = rand::rng().random();
        let nonce = Nonce::try_from(rng).unwrap();

        // TODO: make sure it's hard to crack
        let mut chacha_key = [0u8; 32];
        Argon2::default().hash_password_into(digit_list.as_slice(), BACKUP_SALT, &mut chacha_key)
            .expect("Should be able to hash the digest list");

        let cipher = ChaCha20Poly1305::new_from_slice(&chacha_key)
            .expect("Aes256Gcm::new_from_slice failed");

        match cipher.encrypt(&nonce, data) {
            Ok(ciphertext) => Ok(Arc::new(BackupRestore {
                digit_list,
                nonce: nonce.to_vec(),
                ciphertext: Some(ciphertext),
            })),
            Err(e) => Err(V1CryptographicError::FailedToEncrypt { err: e.to_string() })
        }
    }

    fn v1_restore_decrypt(&self) -> Result<Vec<u8>, V1CryptographicError> {
        if self.ciphertext.is_none() {
            return Err(V1CryptographicError::CiphertextEmpty)
        }

        let mut chacha_key = [0u8; 32];
        Argon2::default()
            .hash_password_into(self.digit_list.as_slice(), BACKUP_SALT, &mut chacha_key)
            .expect("Should be able to hash the digest list");

        let cipher = ChaCha20Poly1305::new_from_slice(&chacha_key)
            .expect("Aes256Gcm::new_from_slice failed");

        let nonce = Nonce::try_from(self.nonce.as_slice()).unwrap();
        match cipher.decrypt(&nonce, self.ciphertext.as_ref().unwrap().as_slice()) {
            Ok(ciphertext) => Ok(ciphertext),
            Err(e) => Err(V1CryptographicError::FailedToEncrypt { err: e.to_string() })
        }
    }

}


#[test]
fn v1_backup_test() {
    let rng: [u8; 32] = rand::rng().random();
    let backup_restore = BackupRestore::v1_backup_encrypt(rng.as_slice()).unwrap();
    let br = BackupRestore {
        digit_list: backup_restore.digit_list.clone(),
        nonce: backup_restore.nonce.clone(),
        ciphertext: backup_restore.ciphertext.clone(),
    };
    let plaintext = br.v1_restore_decrypt().unwrap();

    assert_eq!(rng.as_slice(), plaintext);
}