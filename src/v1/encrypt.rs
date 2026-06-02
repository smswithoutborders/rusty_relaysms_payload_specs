use std::io::Read;
use aead::{Payload};
use sha2::Sha256;
use hkdf::Hkdf;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use chacha20poly1305::ChaCha20Poly1305;
use rand::RngExt;

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
}

fn token_key_derivation(
    dh: SharedSecret,
    dh1: SharedSecret
) -> Vec<u8> {
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
protocol = “3DH_25519_AESGCM”
salt = “RelaySMS v1”
ds = “RelaySMS Encryption Key v1”

# Method name begins with a trailing /
# LEN(MethodName) is fixed to 1 byte
request_string = LEN(MethodName) || MethodName

dh = DH(eC_kid, sS_kid_pk)
dh1 = DH(eC_kid, eS_kid_pk)
prk = HKDF-Extract(salt=salt, ikm=dh || dh1)
aes_key = HKDF-Expand(prk=prk, info = protocol || 0x00 || ds)

nonce = 0x00 * 12 // safe because aes_key is one-time
associated_data = Key-id || eC_kid_pk || eS_kid_pk
token = AEAD_ENCRYPT(input = request_string, key = aes_key, ad = associated_data, nonce=nonce)
*/

const TOKEN_PROTOCOL: &[u8] = b"3DH_25519_AESGCM";
const TOKEN_SALT: &[u8] = b"RelaySMS v1";
const TOKEN_DS: &[u8] = b"RelaySMS Encryption Key v1";

#[uniffi::export]
fn v1_token_encrypt(
    ec_kid: Vec<u8>,
    ss_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    method_name: Vec<u8>,
    token_hash: Vec<u8>,
    key_id: u8,
) -> Result<Vec<u8>, V1CryptographicError> {
    if method_name.len() > u8::MAX as usize {
        return Err(V1CryptographicError::FailedToEncrypt {
            err: "Method length too long: ".to_string() + method_name.len().to_string().as_str()
        })
    }
    let len = [method_name.len() as u8]; // guarantee = 1 byte
    let request_string = [len.as_slice(), method_name.as_slice(), token_hash.as_slice()]
        .concat();

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
        msg: request_string.as_slice(),
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
    let associated_data = [
        key_id.to_le_bytes().to_vec(),
        ec_kid_pk.clone(),
        es_kid_pk.clone()
    ].concat();
    let payload = Payload {
        msg: received_payload.as_slice(),
        aad: associated_data.as_slice(),
    };

    let ec_kid_pk: [u8; 32] = ec_kid_pk.try_into().expect("ec_kid_pk should be 32 bytes");
    let ec_kid_pk= PublicKey::from(ec_kid_pk);

    let ss_kid: [u8; 32] = ss_kid.try_into().expect("ss_kid should be 32 bytes");
    let ss_kid_secret = StaticSecret::from(ss_kid);

    let es_kid: [u8; 32] = es_kid.try_into().expect("es_kid should be 32 bytes");
    let es_kid_secret = StaticSecret::from(es_kid);

    let dh = ss_kid_secret.diffie_hellman(&ec_kid_pk);
    let dh1 = es_kid_secret.diffie_hellman(&ec_kid_pk);

    let aes_key = token_key_derivation(dh, dh1);
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");

    match cipher.decrypt(&nonce, payload) {
        Ok(decrypted_payload) => {
            Ok(parse_for_token_hash(decrypted_payload))
        },
        Err(e) => Err(V1CryptographicError::FailedToDecrypt {
            err: e.to_string(),
        })
    }
}

fn parse_for_token_hash(data: Vec<u8>) -> Vec<u8> {
    let method_name_len = data[0];
    data[1 + method_name_len as usize..].to_vec()
}


/*
protocol = “3DH_25519_ChaCha20-Poly1305”
salt = “RelaySMS v1”
ds = “RelaySMS Publishing Key v1”

dh = DH(eC_kid, sS_kid_pk)
dh1 = DH(eC_kid, eS_kid_pk)
prk = HKDF-Extract(salt=salt, ikm=dh || dh1)
chacha_key= HKDF-Expand(prk=prk, info = protocol || 0x00 || ds, len=32)

nonce = 0x00 * 12 // safe because chacha_key is one-time
associated_data = Key-id || eC_kid_pk || eS_kid_pk
ciphertext = AEAD_ENCRYPT(
input = request_string,
key = chacha_key,
ad = associated_data,
nonce = nonce
)
*/


#[uniffi::export]
fn v1_platform_publisher(
    ec_kid: Vec<u8>,
    ec_kid_pk: Vec<u8>,
    ss_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    key_id: Vec<u8>,
    plaintext: Vec<u8>,
) -> Result<Vec<u8>, V1CryptographicError> {
    let protocol = b"3DH_25519_ChaCha20-Poly1305";
    let salt = b"RelaySMS v1";
    let ds = b"RelaySMS Publishing Key v1";

    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();
    let associated_data = [key_id, ec_kid_pk, es_kid_pk.clone()].concat();
    let payload = Payload {
        msg: plaintext.as_slice(),
        aad: associated_data.as_slice(),
    };
    let ss_kid_pk: [u8; 32] = ss_kid_pk.try_into().expect("ss_kid_pk should be 32 bytes");
    let ss_kid_pk= PublicKey::from(ss_kid_pk);

    let es_kid_pk: [u8; 32] = es_kid_pk.try_into().expect("es_kid_pk should be 32 bytes");
    let es_kid_pk= PublicKey::from(es_kid_pk);

    let ec_kid: [u8; 32] = ec_kid.try_into().expect("ec_kid should be 32 bytes");
    let ec_kid_secret = StaticSecret::from(ec_kid);

    let dh = ec_kid_secret.diffie_hellman(&ss_kid_pk);
    let dh1 = ec_kid_secret.diffie_hellman(&es_kid_pk);

    let mut iter = dh.to_bytes().into_iter().chain(dh1.to_bytes());
    let con_dh_dh1: [u8; 64] = std::array::from_fn(|i| { iter.next().unwrap() });
    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(salt.as_slice()), con_dh_dh1.as_slice());

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

    let method_name= b"Sample method name";
    let token_hash= b"Sample token hash";
    let key_id= 255u8;
    let ciphertext = v1_token_encrypt(
        ec_kid.as_bytes().to_vec(),
        ss_kid_pk,
        es_kid_pk,
        method_name.to_vec(),
        token_hash.to_vec(),
        key_id,
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