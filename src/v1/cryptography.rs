mod platforms;
mod requests;
mod tokens;
mod bridges;
mod backups;

use aead::{Aead, Payload};
use aes_gcm::Nonce;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum V1CryptographicError {
    #[error("Failed to decrypt: {err}")]
    FailedToDecrypt {
        err: String,
    },

    #[error("Failed to encrypt: {err}")]
    FailedToEncrypt {
        err: String,
    },

    #[error("Ciphertext is empty")]
    CiphertextEmpty,

    #[error("No recovery key found")]
    NoRecoveryKeyFound,
}

pub fn triple_dh_decryption(
    key_id: u8,
    ec_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    ss_kid: Vec<u8>,
    es_kid: Vec<u8>,
) -> Result<(SharedSecret, SharedSecret, Vec<u8>), V1CryptographicError>{
    let associated_data = [
        key_id.to_le_bytes().to_vec(),
        ec_kid_pk.clone(),
        es_kid_pk.clone()
    ].concat();

    let ec_kid_pk: [u8; 32] = ec_kid_pk.try_into().expect("ec_kid_pk should be 32 bytes");
    let ec_kid_pk= PublicKey::from(ec_kid_pk);

    let ss_kid: [u8; 32] = ss_kid.try_into().expect("ss_kid should be 32 bytes");
    let ss_kid_secret = StaticSecret::from(ss_kid);

    let es_kid: [u8; 32] = es_kid.try_into().expect("es_kid should be 32 bytes");
    let es_kid_secret = StaticSecret::from(es_kid);

    let dh = ss_kid_secret.diffie_hellman(&ec_kid_pk);
    let dh1 = es_kid_secret.diffie_hellman(&ec_kid_pk);
    Ok((dh, dh1, associated_data.to_vec()))
}

fn v1_publishing_encryption(
    protocol: &[u8],
    salt: &[u8],
    ds: &[u8],
    ec_kid: Vec<u8>,
    ss_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    key_id: u8,
    plaintext: Vec<u8>,
) -> Result<Vec<u8>, V1CryptographicError>{
    let ec_kid: [u8; 32] = ec_kid.try_into().expect("ec_kid should be 32 bytes");
    let ec_kid = StaticSecret::from(ec_kid);
    let ec_kid_pk = PublicKey::from(&ec_kid);

    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();
    let associated_data = [
        key_id.to_le_bytes().as_slice().to_vec(),
        ec_kid_pk.to_bytes().to_vec(),
        es_kid_pk.clone()
    ].concat();

    let payload = Payload {
        msg: plaintext.as_slice(),
        aad: associated_data.as_slice(),
    };
    let ss_kid_pk: [u8; 32] = ss_kid_pk.try_into().expect("ss_kid_pk should be 32 bytes");
    let ss_kid_pk= PublicKey::from(ss_kid_pk);

    let es_kid_pk: [u8; 32] = es_kid_pk.try_into().expect("es_kid_pk should be 32 bytes");
    let es_kid_pk= PublicKey::from(es_kid_pk);

    let dh = ec_kid.diffie_hellman(&ss_kid_pk);
    let dh1 = ec_kid.diffie_hellman(&es_kid_pk);

    let mut iter = dh.to_bytes().into_iter().chain(dh1.to_bytes());
    let con_dh_dh1: [u8; 64] = std::array::from_fn(|i| { iter.next().unwrap() });
    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(salt), con_dh_dh1.as_slice());

    let info = [protocol, [0x00u8].as_slice(), ds].concat();

    let mut chacha_key = [0u8; 32];
    hk.expand(info.as_slice(), &mut chacha_key).expect("expansion should be ok");

    let cipher = ChaCha20Poly1305::new_from_slice(&chacha_key)
        .expect("Aes256Gcm::new_from_slice failed");

    match cipher.encrypt(&nonce, payload) {
        Ok(ciphertext) => Ok(ciphertext.to_vec()),
        Err(e) => Err(V1CryptographicError::FailedToEncrypt {
            err: e.to_string(),
        })
    }
}

fn v1_publishing_decrypt(
    protocol: &[u8],
    salt: &[u8],
    ds: &[u8],
    ec_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    ss_kid: Vec<u8>,
    es_kid: Vec<u8>,
    key_id: u8,
    received_payload: Vec<u8>,
) -> Result<Vec<u8>, V1CryptographicError> {
    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();
    let (dh, dh1, aad) = match triple_dh_decryption(
        key_id,
        ec_kid_pk,
        es_kid_pk,
        ss_kid,
        es_kid
    ) {
        Ok((dh1, dh2, aad)) => (dh1, dh2, aad),
        Err(e) => return Err(V1CryptographicError::FailedToDecrypt {
            err: e.to_string(),
        })
    };
    let payload = Payload {
        msg: received_payload.as_slice(),
        aad: aad.as_slice(),
    };

    let mut iter = dh.to_bytes().into_iter().chain(dh1.to_bytes());
    let con_dh_dh1: [u8; 64] = std::array::from_fn(|i| { iter.next().unwrap() });
    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(salt), con_dh_dh1.as_slice());
    let info = [protocol, [0x00u8].as_slice(), ds].concat();
    let mut chacha_key = [0u8; 32];
    hk.expand(info.as_slice(), &mut chacha_key).expect("expansion should be ok");
    chacha_key.to_vec();

    let cipher = ChaCha20Poly1305::new_from_slice(&chacha_key)
        .expect("ChaCha20Poly1305::new_from_slice failed");

    match cipher.decrypt(&nonce, payload) {
        Ok(decrypted_payload) => Ok(decrypted_payload),
        Err(e) => Err(V1CryptographicError::FailedToDecrypt {
            err: e.to_string(),
        })
    }
}
