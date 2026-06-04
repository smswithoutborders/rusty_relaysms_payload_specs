use aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngExt;
use x25519_dalek::{PublicKey, StaticSecret};
use sha256::{digest, try_digest};
use sha2::Sha256;
use crate::v1::cryptography::{v1_publishing_decrypt, v1_publishing_encryption, V1CryptographicError};

const PROTOCOL_ONLINE_FIRST: &[u8] = b"3DH_25519_ChaCha20-Poly1305";
const SALT_ONLINE_FIRST: &[u8] = b"RelaySMS v1";
const DS_ONLINE_FIRST: &[u8] = b"RelaySMS Bridge Key v1";


#[uniffi::export]
fn v1_bridge_online_first_publisher_encrypt(
    ec_kid: Vec<u8>,
    ss_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    key_id: u8,
    plaintext: Vec<u8>,
) -> Result<Vec<u8>, V1CryptographicError> {
    v1_publishing_encryption(
        PROTOCOL_ONLINE_FIRST,
        SALT_ONLINE_FIRST,
        DS_ONLINE_FIRST,
        ec_kid,
        ss_kid_pk,
        es_kid_pk,
        key_id,
        plaintext,
    )
}

#[uniffi::export]
fn v1_bridge_online_first_publisher_decrypt(
    ec_kid_pk: Vec<u8>,
    es_kid_pk: Vec<u8>,
    ss_kid: Vec<u8>,
    es_kid: Vec<u8>,
    key_id: u8,
    received_payload: Vec<u8>,
) -> Result<Vec<u8>, V1CryptographicError> {
    v1_publishing_decrypt(
        PROTOCOL_ONLINE_FIRST,
        SALT_ONLINE_FIRST,
        DS_ONLINE_FIRST,
        ec_kid_pk,
        es_kid_pk,
        ss_kid,
        es_kid,
        key_id,
        received_payload,
    )
}


const PROTOCOL_OFFLINE_FIRST: &[u8] = b"Noise_IK_25519_AESGCM_SHA256";
const INFO_OFFLINE_FIRST: &[u8] = b"RelaySMS C2S DR v1";

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct OfflineFirstEncryptionResponse {
    tx_payload: Vec<u8>,
    sc_pk_enc: Vec<u8>,
    h: Vec<u8>,
}

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct OfflineFirstDecryptionResponse {
    payload: Vec<u8>,
    h: Vec<u8>,
}

#[uniffi::export]
fn v1_bridge_offline_first_publisher_encrypt(
    ss_pk: Vec<u8>,
    ec: Vec<u8>,
    sc: Vec<u8>,
    payload: Vec<u8>,
) -> Result<OfflineFirstEncryptionResponse, V1CryptographicError> {
    let mut h = digest(PROTOCOL_OFFLINE_FIRST);
    let ck = h.clone();

    h = digest([h.into_bytes().to_vec(), ss_pk.clone()].concat());

    let ec: [u8; 32] = ec.try_into().expect("ec should be 32 bytes");
    let ec = StaticSecret::from(ec);
    let ec_pk = PublicKey::from(&ec);
    h = digest([h.into_bytes().to_vec(), ec_pk.to_bytes().to_vec()].concat());

    let ss_pk: [u8; 32] = ss_pk.try_into().expect("ss_pk should be 32 bytes");
    let ss_pk = PublicKey::from(ss_pk);
    let dh_es = ec.diffie_hellman(&ss_pk);

    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(ck.as_bytes()), dh_es.to_bytes().as_slice());
    let mut ck = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut ck).expect("expansion should be ok");
    let mut k = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut k).expect("expansion should be ok");

    let sc: [u8; 32] = sc.try_into().expect("sc should be 32 bytes");
    let sc = StaticSecret::from(sc);
    let sc_pk = PublicKey::from(&sc);

    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();
    let cipher = Aes256Gcm::new_from_slice(k.as_slice())
        .expect("Aes256Gcm::new_from_slice failed");

    let aad = Payload {
        msg: sc_pk.as_ref(),
        aad: h.as_bytes(),
    };
    let sc_pk_enc = cipher.encrypt(&nonce, aad)
        .expect("encryption should be ok");
    // let aad = Payload {
    //     msg: sc_pk_enc.as_ref(),
    //     aad: h.as_bytes(),
    // };
    // let test = cipher.decrypt(&nonce, aad).expect("decryption should be ok");
    // assert_eq!(sc_pk.as_bytes().to_vec(), test);

    h = digest([h.into_bytes().to_vec(), sc_pk_enc.to_vec()].concat());
    let dh_ss = sc.diffie_hellman(&ss_pk);

    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(ck.as_slice()), dh_ss.to_bytes().as_slice());
    let mut ck = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut ck).expect("expansion should be ok");
    let mut k = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut k).expect("expansion should be ok");

    let cipher = Aes256Gcm::new_from_slice(k.as_slice())
        .expect("Aes256Gcm::new_from_slice failed");

    let aad = Payload {
        msg: payload.as_slice(),
        aad: h.as_bytes(),
    };
    let tx_payload = cipher.encrypt(&nonce, aad).expect("encryption should be ok");

    h = digest([h.into_bytes().to_vec(), sc_pk_enc.to_vec()].concat());

    Ok(OfflineFirstEncryptionResponse {
        tx_payload,
        sc_pk_enc,
        h: h.into_bytes().to_vec(),
    })
}

/*
dh_es = DH(sS, eC_pk)
ck, k = HKDF(salt = ck, input = dh_es, info = info)
sC_pk = DECRYPT(key = k, input = sC_pk_enc, ad = h)
h = SHA256(h || sC_pk_enc)
dh_ss = DH(sS, sC_pk)
ck, k = HKDF(salt = ck, input = dh_ss, info = Info)
payload = DECRYPT(key = k, input = rx_payload, ad = h)
 */
#[uniffi::export]
fn v1_bridge_offline_first_publisher_decrypt(
    ss: Vec<u8>,
    ec_pk: Vec<u8>,
    sc_pk_enc: Vec<u8>,
    rx_payload: Vec<u8>,
) -> Result<OfflineFirstDecryptionResponse, V1CryptographicError> {
    let mut h = digest(PROTOCOL_OFFLINE_FIRST);
    let ck = h.clone();

    let ss: [u8; 32] = ss.try_into().expect("ss should be 32 bytes");
    let ss = StaticSecret::from(ss);
    let ss_pk = PublicKey::from(&ss);
    h = digest([h.into_bytes().to_vec(), ss_pk.as_bytes().to_vec()].concat());

    let ec_pk: [u8; 32] = ec_pk.try_into().expect("ec should be 32 bytes");
    let ec_pk = PublicKey::from(ec_pk);
    h = digest([h.into_bytes().to_vec(), ec_pk.to_bytes().to_vec()].concat());

    let dh_es = ss.diffie_hellman(&ec_pk);

    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(ck.as_bytes()), dh_es.to_bytes().as_slice());
    let mut ck = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut ck).expect("expansion should be ok");
    let mut k = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut k).expect("expansion should be ok");

    let nonce = Nonce::try_from([0x00u8; 12]).unwrap();
    let cipher = Aes256Gcm::new_from_slice(k.as_slice())
        .expect("Aes256Gcm::new_from_slice failed");
    let aad = Payload {
        msg: sc_pk_enc.as_ref(),
        aad: h.as_bytes(),
    };
    let sc_pk = cipher.decrypt(&nonce, aad)
        .expect("decryption should be ok");

    h = digest([h.into_bytes().to_vec(), sc_pk_enc.to_vec()].concat());

    let sc_pk: [u8; 32] = sc_pk.try_into().expect("sc pk should be 32 bytes");
    let sc_pk = PublicKey::from(sc_pk);
    let dh_ss = ss.diffie_hellman(&sc_pk);

    let (_, hk) = Hkdf::<Sha256>::extract(
        Option::from(ck.as_slice()), dh_ss.to_bytes().as_slice());
    let mut ck = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut ck).expect("expansion should be ok");
    let mut k = [0u8; 32];
    hk.expand(INFO_OFFLINE_FIRST, &mut k).expect("expansion should be ok");

    let cipher = Aes256Gcm::new_from_slice(k.as_slice())
        .expect("Aes256Gcm::new_from_slice failed");
    let aad = Payload {
        msg: rx_payload.as_ref(),
        aad: h.as_bytes(),
    };
    let payload = cipher.decrypt(&nonce, aad)
        .expect("decryption should be ok");
    Ok(OfflineFirstDecryptionResponse {
        payload,
        h: h.into_bytes().to_vec(),
    })
}

#[test]
fn test_bridge_online_first_publisher_encrypt_decrypt() {
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
    let ciphertext = v1_bridge_online_first_publisher_encrypt(
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
    let decrypted = v1_bridge_online_first_publisher_decrypt(
        ec_kid_pk,
        es_kid_pk,
        ss_kid,
        es_kid,
        key_id,
        ciphertext,
    ).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_bridge_offline_first_publisher_encrypt_decrypt() {
    let rng: [u8; 32] = rand::rng().random();
    let ec_kid = StaticSecret::from(rng);

    let rng: [u8; 32] = rand::rng().random();
    let sc = StaticSecret::from(rng);

    let rng: [u8; 32] = rand::rng().random();
    let ss_kid = StaticSecret::from(rng);
    let ss_kid_pk = PublicKey::from(&ss_kid).as_bytes().to_vec();

    let plaintext= b"Hello world";
    let ciphertext = v1_bridge_offline_first_publisher_encrypt(
        ss_kid_pk,
        ec_kid.as_bytes().to_vec(),
        sc.as_bytes().to_vec(),
        plaintext.to_vec(),
    ).unwrap();

    let ec_kid_pk = PublicKey::from(&ec_kid).as_bytes().to_vec();

    let ss_kid = ss_kid.as_bytes().to_vec();
    let decrypted = v1_bridge_offline_first_publisher_decrypt(
        ss_kid,
        ec_kid_pk,
        ciphertext.sc_pk_enc,
        ciphertext.tx_payload
    ).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted.payload);
}
