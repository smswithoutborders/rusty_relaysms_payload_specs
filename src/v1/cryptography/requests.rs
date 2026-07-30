use std::os::unix::raw::time_t;
use std::time::{SystemTime, UNIX_EPOCH};
use aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngExt;
use sha2::Sha256;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};
use crate::v1::cryptography::V1CryptographicError;

fn requests_key_derivation( dh: SharedSecret, ) -> Vec<u8> {
    const TOKEN_PROTOCOL: &[u8] = b"Noise_NK_0RTT_25519_AESGCM";
    const TOKEN_SALT: &[u8] = b"RelaySMS v1";
    const TOKEN_DS: &[u8] = b"RelaySMS Publisher O-Auth2.0 v1";

    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(TOKEN_SALT), dh.as_bytes().as_slice());

    let info = [TOKEN_PROTOCOL, [0x00u8].as_slice(), TOKEN_DS].concat();
    let mut key = [0u8; 32];
    hk.expand(info.as_slice(), &mut key).expect("expansion should be ok");

    key.to_vec()
}

#[derive(PartialEq, Debug, uniffi::Record)]
struct RequestPayload {
    pub ciphertext: Vec<u8>,
    pub timestamp: u64,
    pub nonce: Vec<u8>,
}


#[uniffi::export]
fn v1_requests_encrypt(
    ec: &[u8],
    ss_kid_pk: &[u8],
    method_name: &[u8],
    payload: Option<Vec<u8>>,
    timestamp: Option<u64>,
) -> Result<RequestPayload, V1CryptographicError> {
    let ec: [u8; 32] = ec.try_into().expect("es_kid should be 32 bytes");
    let ec = StaticSecret::from(ec);
    let ec_pk = PublicKey::from(&ec);

    let ss_kid_pk: [u8; 32] = ss_kid_pk.try_into().expect("ss_kid_pk should be 32 bytes");
    let ss_kid_pk = PublicKey::from(ss_kid_pk);

    let associated_data = [
        ec_pk.to_bytes().to_vec(),
        ss_kid_pk.to_bytes().to_vec(),
    ].concat();

    let start = SystemTime::now();
    let timestamp = if timestamp.is_some() {timestamp.unwrap()} else {
        start.duration_since(UNIX_EPOCH)
            .expect("time should go forward")
            .as_secs()
    };

    let method_len = method_name.len() as u8;
    let mut request_string: Vec<u8> = Vec::new();
    request_string.push(method_len);
    request_string.extend(method_name);
    request_string.extend(timestamp.to_le_bytes());
    if payload.is_some() {
        request_string.extend(payload.unwrap())
    }

    let payload = Payload {
        msg: request_string.as_slice(),
        aad: associated_data.as_slice(),
    };

    let dh = ec.diffie_hellman(&ss_kid_pk);
    let aes_key = requests_key_derivation(dh);

    let rng: [u8; 12] = rand::rng().random();
    let nonce = rng.as_slice();
    let nonce1 = Nonce::try_from(nonce).unwrap();
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");
    match cipher.encrypt(&nonce1, payload) {
        Ok(ciphertext) => Ok( RequestPayload {
            ciphertext,
            timestamp,
            nonce: nonce.to_vec(),
        }),
        Err(e) => Err(V1CryptographicError::FailedToEncrypt {
            err: e.to_string(),
        })
    }
}

#[derive(PartialEq, Debug, uniffi::Record)]
struct ResponsePayload {
    pub method_name: Vec<u8>,
    pub payload: Vec<u8>,
}


#[uniffi::export]
fn v1_requests_decrypt(
    ss_kid: &[u8],
    ec_pk: &[u8],
    nonce: Vec<u8>,
    ciphertext: &[u8],
) -> Result<ResponsePayload, V1CryptographicError> {
    let ss_kid: [u8; 32] = ss_kid.try_into().expect("ss_kid should be 32 bytes");
    let ss_kid = StaticSecret::from(ss_kid);
    let ss_kid_pk = PublicKey::from(&ss_kid);

    let ec_pk: [u8; 32] = ec_pk.try_into().expect("ec_pk should be 32 bytes");
    let ec_pk = PublicKey::from(ec_pk);

    let associated_data = [
        ec_pk.to_bytes().to_vec(),
        ss_kid_pk.to_bytes().to_vec(),
    ].concat();

    let payload = Payload {
        msg: ciphertext,
        aad: associated_data.as_slice(),
    };

    let dh = ss_kid.diffie_hellman(&ec_pk);

    let aes_key = requests_key_derivation(dh);
    let nonce = Nonce::try_from(nonce.as_slice()).unwrap();

    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");

    match cipher.decrypt(&nonce, payload) {
        Ok(ciphertext) => {
            let length = ciphertext[0];
            let timestamp_len = 8;
            let total_len = 1 + length + timestamp_len;
            let method_name = &ciphertext[1..(1+ length) as usize];
            let payload = &ciphertext[total_len as usize..];
            Ok(ResponsePayload {
                method_name: method_name.to_vec(),
                payload: payload.to_vec(),
            })
        },
        Err(e) => Err(V1CryptographicError::FailedToDecrypt {
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

    let method_name= b"/send";
    let payload: [u8; 64] = rand::rng().random();

    let ss_kid_pk = PublicKey::from(&ss_kid);
    let ciphertext = v1_requests_encrypt(
        ec_kid.to_bytes().as_slice(),
        ss_kid_pk.to_bytes().as_slice(),
        method_name.as_slice(),
        Some(payload.to_vec()),
        None,
    ).unwrap();

    let decrypted = v1_requests_decrypt(
        ss_kid.as_bytes(),
        ec_kid_pk.to_bytes().as_slice(),
        ciphertext.nonce,
        ciphertext.ciphertext.as_slice(),
    ).unwrap();

    assert_eq!(payload.to_vec(), decrypted.payload);
    assert_eq!(method_name.to_vec(), decrypted.method_name);

    let start = SystemTime::now();
    let timestamp = start.duration_since(UNIX_EPOCH)
        .expect("time should go forward")
        .as_secs();

    let ciphertext = v1_requests_encrypt(
        ec_kid.to_bytes().as_slice(),
        ss_kid_pk.to_bytes().as_slice(),
        method_name.as_slice(),
        None,
        Some(timestamp),
    ).unwrap();

    let decrypted = v1_requests_decrypt(
        ss_kid.as_bytes(),
        ec_kid_pk.to_bytes().as_slice(),
        ciphertext.nonce,
        ciphertext.ciphertext.as_slice(),
    ).unwrap();

    assert!(decrypted.payload.is_empty());
    assert_eq!(method_name.to_vec(), decrypted.method_name);
}
