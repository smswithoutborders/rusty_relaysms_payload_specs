use aead::{Aead, Payload};
use aes_gcm::Nonce;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use hkdf::Hkdf;
use rand::RngExt;
use sha2::Sha256;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};
use crate::v1::cryptography::{v1_publishing_decrypt, v1_publishing_encryption, triple_dh_decryption, V1CryptographicError};

const PROTOCOL: &[u8] = b"3DH_25519_ChaCha20-Poly1305";
const SALT: &[u8] = b"RelaySMS v1";
const DS: &[u8] = b"RelaySMS Publishing Key v1";

#[uniffi::export]
fn v1_platform_publisher_encrypt(
    ec_kid: Vec<u8>,
    ss_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    key_id: u8,
    plaintext: Vec<u8>,
) -> Result<Vec<u8>, V1CryptographicError> {
    v1_publishing_encryption(
        PROTOCOL,
        SALT,
        DS,
        ec_kid,
        ss_kid_pk,
        es_kid_pk,
        key_id,
        plaintext,
    )
}

#[uniffi::export]
fn v1_platform_publisher_decrypt(
    ec_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    ss_kid: Vec<u8>,
    es_kid: Vec<u8>,
    key_id: u8,
    received_payload: Vec<u8>,
) -> Result<Vec<u8>, V1CryptographicError> {
    v1_publishing_decrypt(
        PROTOCOL,
        SALT,
        DS,
        ec_kid_pk,
        es_kid_pk,
        ss_kid,
        es_kid,
        key_id,
        received_payload,
    )
}

#[test]
fn test_publisher_encryption_decryption() {
    let rng: [u8; 32] = rand::rng().random();
    let ec_kid = StaticSecret::from(rng);

    let rng: [u8; 32] = rand::rng().random();
    let ss_kid = StaticSecret::from(rng);
    let ss_kid_pk = PublicKey::from(&ss_kid).as_bytes().to_vec();

    let rng: [u8; 32] = rand::rng().random();
    let es_kid = StaticSecret::from(rng);
    let es_kid_pk = PublicKey::from(&es_kid).as_bytes().to_vec();

    let plaintext= b"129701738923479234023840384h3hsfhasdf7123947293472394732497";
    let key_id= 255u8;
    let ciphertext = v1_platform_publisher_encrypt(
        ec_kid.as_bytes().to_vec(),
        ss_kid_pk,
        es_kid_pk,
        key_id,
        plaintext.to_vec(),
    ).unwrap();

    let ec_kid_pk = PublicKey::from(&ec_kid).as_bytes().to_vec();
    let es_kid_pk = PublicKey::from(&es_kid).as_bytes().to_vec();
    let ss_kid = ss_kid.as_bytes().to_vec();
    let es_kid = es_kid.as_bytes().to_vec();
    let decrypted = v1_platform_publisher_decrypt(
        ec_kid_pk,
        es_kid_pk,
        ss_kid,
        es_kid,
        key_id,
        ciphertext,
    ).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted);
}
