use std::time::{SystemTime, UNIX_EPOCH};
use aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngExt;
use sha2::Sha256;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};
use crate::v1::cryptography::V1CryptographicError;

fn requests_key_derivation(
    dh: SharedSecret,
    dh1: SharedSecret
) -> (Vec<u8>, Vec<u8>) {
    const TOKEN_PROTOCOL: &[u8] = b"Noise_NK_0RTT_25519_AESGCM";
    const TOKEN_SALT: &[u8] = b"RelaySMS v1";
    const TOKEN_DS: &[u8] = b"RelaySMS Publisher O-Auth2.0 v1";
    let mut iter = dh.to_bytes().into_iter().chain(dh1.to_bytes());
    let con_dh_dh1: [u8; 64] = std::array::from_fn(|i| { iter.next().unwrap() });

    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(TOKEN_SALT), con_dh_dh1.as_slice());

    let info = [TOKEN_PROTOCOL, [0x00u8].as_slice(), TOKEN_DS].concat();
    let mut key = [0u8; 32];
    hk.expand(info.as_slice(), &mut key).expect("expansion should be ok");

    let info = [TOKEN_PROTOCOL, [0x01u8].as_slice(), TOKEN_DS].concat();
    let mut nonce = [0u8; 12];
    hk.expand(info.as_slice(), &mut nonce).expect("expansion should be ok");

    (key.to_vec(), nonce.to_vec())
}

#[derive(PartialEq, Debug, uniffi::Record)]
struct RequestPayload {
    pub ciphertext: Vec<u8>,
    pub timestamp: u64
}


#[uniffi::export]
fn requests_encrypt(
    ec_pk: &[u8],
    ss_kid: &[u8],
    es: &[u8],
    method_name: &[u8],
) -> Result<RequestPayload, V1CryptographicError> {

    let ec_pk: [u8; 32] = ec_pk.try_into().expect("ec_pk should be 32 bytes");
    let ec_pk = PublicKey::from(ec_pk);

    let ss_kid: [u8; 32] = ss_kid.try_into().expect("ss_kid should be 32 bytes");
    let ss_kid = StaticSecret::from(ss_kid);

    let es: [u8; 32] = es.try_into().expect("es_kid should be 32 bytes");
    let es = StaticSecret::from(es);
    let es_pk = PublicKey::from(&es);

    let associated_data = [
        ec_pk.to_bytes().to_vec(),
        es_pk.to_bytes().to_vec(),
    ].concat();

    let start = SystemTime::now();
    let timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward")
        .as_secs();

    let request_string = [method_name, timestamp.to_le_bytes().as_slice()].concat();
    let payload = Payload {
        msg: request_string.as_slice(),
        aad: associated_data.as_slice(),
    };

    let dh = ss_kid.diffie_hellman(&ec_pk);
    let dh1 = es.diffie_hellman(&ec_pk);

    let (aes_key, nonce) = requests_key_derivation(dh, dh1);
    let nonce = Nonce::try_from(nonce.as_slice()).unwrap();
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");
    match cipher.encrypt(&nonce, payload) {
        Ok(ciphertext) => Ok( RequestPayload {
            ciphertext,
            timestamp
        }),
        Err(e) => Err(V1CryptographicError::FailedToEncrypt {
            err: e.to_string(),
        })
    }
}


#[uniffi::export]
fn requests_decrypt(
    ec_kid: &[u8],
    ss_kid_pk: &[u8],
    es_kid_pk: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, V1CryptographicError> {
    let ss_kid_pk: [u8; 32] = ss_kid_pk.try_into().expect("ss_kid_pk should be 32 bytes");
    let ss_kid_pk = PublicKey::from(ss_kid_pk);

    let es_kid_pk: [u8; 32] = es_kid_pk.try_into().expect("es_kid_pk should be 32 bytes");
    let es_kid_pk = PublicKey::from(es_kid_pk);

    let ec_kid: [u8; 32] = ec_kid.try_into().expect("ec_kid should be 32 bytes");
    let ec_kid = StaticSecret::from(ec_kid);
    let ec_kid_pk = PublicKey::from(&ec_kid);

    let associated_data = [
        ec_kid_pk.to_bytes().to_vec(),
        es_kid_pk.to_bytes().to_vec(),
    ].concat();

    let payload = Payload {
        msg: ciphertext,
        aad: associated_data.as_slice(),
    };

    let dh = ec_kid.diffie_hellman(&ss_kid_pk);
    let dh1 = ec_kid.diffie_hellman(&es_kid_pk);

    let (aes_key, nonce) = requests_key_derivation(dh, dh1);
    let nonce = Nonce::try_from(nonce.as_slice()).unwrap();

    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");
    match cipher.decrypt(&nonce, payload) {
        Ok(ciphertext) => Ok(ciphertext.to_vec()),
        Err(e) => Err(V1CryptographicError::FailedToEncrypt {
            err: e.to_string(),
        })
    }
}

#[test]
fn test_request_encryption_decryption() {
    let rng: [u8; 32] = rand::rng().random();
    let ec_kid = StaticSecret::from(rng);
    let ec_kid_pk = PublicKey::from(&ec_kid);

    let rng: [u8; 32] = rand::rng().random();
    let ss_kid = StaticSecret::from(rng);

    let rng: [u8; 32] = rand::rng().random();
    let es_kid = StaticSecret::from(rng);

    let method_name= b"/send";
    let ciphertext = requests_encrypt(
        ec_kid_pk.to_bytes().as_slice(),
        ss_kid.to_bytes().as_slice(),
        es_kid.to_bytes().as_slice(),
        method_name.as_slice(),
    ).unwrap();
    let request_string = [method_name,
        ciphertext.timestamp.to_le_bytes().as_slice()].concat();

    let es_kid_pk = PublicKey::from(&es_kid);
    let ss_kid_pk = PublicKey::from(&ss_kid);
    let decrypted = requests_decrypt(
        ec_kid.to_bytes().as_slice(),
        ss_kid_pk.as_bytes(),
        es_kid_pk.as_bytes(),
        ciphertext.ciphertext.as_slice(),
    ).unwrap();

    assert_eq!(request_string.to_vec(), decrypted);
}
