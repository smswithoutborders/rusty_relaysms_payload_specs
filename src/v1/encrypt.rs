use aead::Payload;
use sha2::Sha256;
use hkdf::Hkdf;
use x25519_dalek::{PublicKey, StaticSecret};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use chacha20poly1305::ChaCha20Poly1305;

fn get_key(
    protocol: &[u8],
    salt: &[u8],
    ds: &[u8],
    ec_kid: Vec<u8>,
    ss_pk_kid: Vec<u8>,
    es_pk_kid: Vec<u8>,
) -> Vec<u8> {
    let ss_pk_kid: [u8; 32] = ss_pk_kid.try_into().expect("ss_pk_kid should be 32 bytes");
    let ss_pk_kid= PublicKey::from(ss_pk_kid);

    let es_pk_kid: [u8; 32] = es_pk_kid.try_into().expect("es_pk_kid should be 32 bytes");
    let es_pk_kid= PublicKey::from(es_pk_kid);

    let ec_kid: [u8; 32] = ec_kid.try_into().expect("ec_kid should be 32 bytes");
    let pub_ec_pk_kid_secret = StaticSecret::from(ec_kid);
    let dh = pub_ec_pk_kid_secret.diffie_hellman(&ss_pk_kid);
    let dh1 = pub_ec_pk_kid_secret.diffie_hellman(&es_pk_kid);
    let mut iter = dh.to_bytes().into_iter().chain(dh1.to_bytes());
    let con_dh_dh1: [u8; 64] = std::array::from_fn(|i| { iter.next().unwrap() });
    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(salt), con_dh_dh1.as_slice());
    let info = [protocol, [0x00u8].as_slice(), ds].concat();
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

dh = DH(eC_kid, sS_pk_kid)
dh1 = DH(eC_kid, eS_pk_kid)
prk = HKDF-Extract(salt=salt, ikm=dh || dh1)
aes_key = HKDF-Expand(prk=prk, info = protocol || 0x00 || ds)

nonce = 0x00 * 12 // safe because aes_key is one-time
associated_data = Key-id || eC_pk_kid || eS_pk_kid
token = AEAD_ENCRYPT(input = request_string, key = aes_key, ad = associated_data, nonce=nonce)
*/

#[uniffi::export]
fn v1_token_derivation(
    ec_kid: Vec<u8>,
    ec_pk_kid: Vec<u8>,
    ss_pk_kid: Vec<u8>,
    es_pk_kid: Vec<u8>,
    method_name: Vec<u8>,
    key_id: Vec<u8>,
) -> Vec<u8> {
    let protocol = "3DH_25519_AESGCM".as_bytes();
    let salt = "RelaySMS v1".as_bytes();
    let ds = "RelaySMS Encryption Key v1".as_bytes();

    let len = method_name.len().to_le_bytes();
    let request_string = [len.as_slice(), method_name.as_slice()].concat();

    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();
    let associated_data = [key_id, ec_pk_kid, es_pk_kid.clone()].concat();
    let payload = Payload {
        msg: request_string.as_slice(),
        aad: associated_data.as_slice(),
    };
    let aes_key = get_key(
        protocol,
        salt,
        ds,
        ec_kid,
        ss_pk_kid,
        es_pk_kid
    );
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .expect("Aes256Gcm::new_from_slice failed");
    cipher.encrypt(&nonce, payload).unwrap_or_else(|_| Vec::new())
}


/*
protocol = “3DH_25519_ChaCha20-Poly1305”
salt = “RelaySMS v1”
ds = “RelaySMS Publishing Key v1”

dh = DH(eC_kid, sS_pk_kid)
dh1 = DH(eC_kid, eS_pk_kid)
prk = HKDF-Extract(salt=salt, ikm=dh || dh1)
chacha_key= HKDF-Expand(prk=prk, info = protocol || 0x00 || ds, len=32)

nonce = 0x00 * 12 // safe because chacha_key is one-time
associated_data = Key-id || eC_pk_kid || eS_pk_kid
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
    ec_pk_kid: Vec<u8>,
    ss_pk_kid: Vec<u8>,
    es_pk_kid: Vec<u8>,
    key_id: Vec<u8>,
    plaintext: Vec<u8>,
) -> Vec<u8> {
    let protocol = "3DH_25519_ChaCha20-Poly1305".as_bytes();
    let salt = "RelaySMS v1".as_bytes();
    let ds = "RelaySMS Publishing Key v1".as_bytes();

    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();
    let associated_data = [key_id, ec_pk_kid, es_pk_kid.clone()].concat();
    let payload = Payload {
        msg: plaintext.as_slice(),
        aad: associated_data.as_slice(),
    };
    let chacha_key = get_key(
        protocol,
        salt,
        ds,
        ec_kid,
        ss_pk_kid,
        es_pk_kid
    );
    let cipher = ChaCha20Poly1305::new_from_slice(&chacha_key)
        .expect("Aes256Gcm::new_from_slice failed");
    cipher.encrypt(&nonce, payload).unwrap_or_else(|_| Vec::new())
}