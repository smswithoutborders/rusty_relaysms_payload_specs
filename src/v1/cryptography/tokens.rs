use aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngExt;
use sha2::Sha256;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};
use crate::v1::cryptography::{triple_dh_decryption, V1CryptographicError};

fn token_key_derivation(
    dh: SharedSecret,
    dh1: SharedSecret
) -> Vec<u8> {
    const TOKEN_PROTOCOL: &[u8] = b"3DH_25519_AESGCM";
    const TOKEN_SALT: &[u8] = b"RelaySMS v1";
    const TOKEN_DS: &[u8] = b"RelaySMS Encryption Key v1";
    let mut iter = dh.to_bytes().into_iter().chain(dh1.to_bytes());
    let con_dh_dh1: [u8; 64] = std::array::from_fn(|i| { iter.next().unwrap() });
    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(TOKEN_SALT), con_dh_dh1.as_slice());
    let info = [TOKEN_PROTOCOL, [0x00u8].as_slice(), TOKEN_DS].concat();
    let mut key = [0u8; 32];
    hk.expand(info.as_slice(), &mut key).expect("expansion should be ok");
    key.to_vec()
}

/*
Returns TokenHash
 */
#[uniffi::export]
fn v1_token_decrypt(
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

    let aes_key = token_key_derivation(dh, dh1);
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");

    match cipher.decrypt(&nonce, payload) {
        Ok(decrypted_payload) => Ok(decrypted_payload),
        Err(e) => Err(V1CryptographicError::FailedToDecrypt {
            err: e.to_string(),
        })
    }
}

/*
protocol = “3DH_25519_AESGCM”
salt = “RelaySMS v1”
ds = “RelaySMS Encryption Key v1”
dh = DH(sC_id, sS_kid_pk)
dh1 = DH(eC_kid, eS_kid_pk)
prk = HKDF-Extract(salt=salt, ikm=dh || dh1)
aes_key = HKDF-Expand(prk=prk, info = protocol || 0x00 || ds)
nonce = 0x00 * 12 // safe because aes_key is one-time
associated_data = Key-id || eC_kid_pk || eS_kid_pk
encrypted_url = AEAD_ENCRYPT(input = Token, key = aes_key, ad = associated_data, nonce=nonce)
 */
#[uniffi::export]
fn v1_token_encrypt(
    ec_kid: Vec<u8>,
    ss_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    key_id: u8,
    token: &[u8],
) -> Result<Vec<u8>, V1CryptographicError> {
    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();

    let ec_kid: [u8; 32] = ec_kid.try_into().expect("ec_kid should be 32 bytes");
    let ec_kid = StaticSecret::from(ec_kid);
    let ec_kid_pk = PublicKey::from(&ec_kid);
    let associated_data = [
        key_id.to_le_bytes().to_vec(),
        ec_kid_pk.to_bytes().to_vec(),
        es_kid_pk.clone()
    ].concat();

    let payload = Payload {
        msg: token,
        aad: associated_data.as_slice(),
    };

    let ss_kid_pk: [u8; 32] = ss_kid_pk.try_into().expect("ss_kid_pk should be 32 bytes");
    let ss_kid_pk= PublicKey::from(ss_kid_pk);

    let es_kid_pk: [u8; 32] = es_kid_pk.try_into().expect("es_kid_pk should be 32 bytes");
    let es_kid_pk= PublicKey::from(es_kid_pk);

    let dh = ec_kid.diffie_hellman(&ss_kid_pk);
    let dh1 = ec_kid.diffie_hellman(&es_kid_pk);

    let aes_key = token_key_derivation(dh, dh1);
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");
    match cipher.encrypt(&nonce, payload) {
        Ok(ciphertext) => Ok(ciphertext.to_vec()),
        Err(e) => Err(V1CryptographicError::FailedToEncrypt {
            err: e.to_string(),
        })
    }
}

#[test]
fn test_token_encryption_decryption() {
    let rng: [u8; 32] = rand::rng().random();
    let ec_kid = StaticSecret::from(rng);

    let rng: [u8; 32] = rand::rng().random();
    let ss_kid = StaticSecret::from(rng);
    let ss_kid_pk = PublicKey::from(&ss_kid).as_bytes().to_vec();

    let rng: [u8; 32] = rand::rng().random();
    let es_kid = StaticSecret::from(rng);
    let es_kid_pk = PublicKey::from(&es_kid).as_bytes().to_vec();

    let token_hash= b"Sample token hash";
    let key_id= 255u8;
    let ciphertext = v1_token_encrypt(
        ec_kid.as_bytes().to_vec(),
        ss_kid_pk,
        es_kid_pk,
        key_id,
        token_hash,
    ).unwrap();

    let ec_kid_pk = PublicKey::from(&ec_kid).as_bytes().to_vec();
    let es_kid_pk = PublicKey::from(&es_kid).as_bytes().to_vec();
    let ss_kid = ss_kid.as_bytes().to_vec();
    let es_kid = es_kid.as_bytes().to_vec();
    let decrypted = v1_token_decrypt(
        ec_kid_pk,
        es_kid_pk,
        ss_kid,
        es_kid,
        key_id,
        ciphertext,
    ).unwrap();

    assert_eq!(token_hash.to_vec(), decrypted);
}
