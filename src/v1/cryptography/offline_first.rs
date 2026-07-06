use aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use chacha20poly1305::ChaCha20Poly1305;
use hkdf::Hkdf;
use rand::RngExt;
use x25519_dalek::{PublicKey, StaticSecret};
use sha256::{digest, try_digest};
use sha2::Sha256;
use crate::v1::cryptography::{v1_publishing_decrypt, v1_publishing_encryption, V1CryptographicError};

const PROTOCOL_OFFLINE_FIRST: &[u8] = b"Noise_IK_25519_ChaChaPoly1305_SHA256";
const INFO_OFFLINE_FIRST: &[u8] = b"RelaySMS C2S DR v1";

#[derive(Debug, uniffi::Object)]
pub struct OfflineFirst {
    payload: Vec<u8>,
    sc_pk_enc: Option<Vec<u8>>,
    ec_pk: Option<Vec<u8>>,
    h: Option<Vec<u8>>,
}
impl PartialEq for OfflineFirst {
    fn eq(&self, other: &Self) -> bool {
        self.payload == other.payload
            && self.sc_pk_enc == other.sc_pk_enc
            && self.ec_pk == other.ec_pk
    }
}

#[uniffi::export]
impl OfflineFirst {
    pub fn get_payload(&self) -> Vec<u8> { self.payload.clone() }
    pub fn get_sc_pk_enc(&self) -> Option<Vec<u8>> { self.sc_pk_enc.clone() }
    pub fn get_ec_pk(&self) -> Option<Vec<u8>> { self.ec_pk.clone() }
    pub fn get_h(&self) -> Option<Vec<u8>> { self.h.clone() }

    #[uniffi::constructor]
    fn encrypt(
        ss_pk: Vec<u8>,
        ec: Vec<u8>,
        sc: Vec<u8>,
        payload: Vec<u8>,
    ) -> Result<OfflineFirst, V1CryptographicError> {
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
        let cipher = ChaCha20Poly1305::new_from_slice(k.as_slice())
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

        let cipher = ChaCha20Poly1305::new_from_slice(k.as_slice())
            .expect("ChaChaPoly1305::new_from_slice failed");

        let aad = Payload {
            msg: payload.as_slice(),
            aad: h.as_bytes(),
        };
        let tx_payload = cipher.encrypt(&nonce, aad).expect("encryption should be ok");

        h = digest([h.into_bytes().to_vec(), sc_pk_enc.to_vec()].concat());

        Ok(OfflineFirst {
            payload: tx_payload,
            sc_pk_enc: Some(sc_pk_enc),
            ec_pk: Some(ec_pk.as_bytes().to_vec()),
            h: Some(h.into_bytes().to_vec()),
        })
    }

    fn serialize(&self) -> Result<Vec<u8>, V1CryptographicError> {
        if self.sc_pk_enc.is_none() {
            return Err(V1CryptographicError::NoEncryptedStaticKey)
        }

        if self.ec_pk.is_none() {
            return Err(V1CryptographicError::NoPublicKeyFound)
        }

        Ok([self.ec_pk.clone().unwrap(),
            self.sc_pk_enc.clone().unwrap(),
            self.payload.as_slice().to_vec()]
            .concat().to_vec())
    }


    #[uniffi::constructor]
    fn decrypt(
        ss: Vec<u8>,
        ec_pk: Vec<u8>,
        sc_pk_enc: Vec<u8>,
        rx_payload: Vec<u8>,
    ) -> Result<OfflineFirst, V1CryptographicError> {
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
        let cipher = ChaCha20Poly1305::new_from_slice(k.as_slice())
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

        let cipher = ChaCha20Poly1305::new_from_slice(k.as_slice())
            .expect("ChaChaPoly1305::new_from_slice failed");
        let aad = Payload {
            msg: rx_payload.as_ref(),
            aad: h.as_bytes(),
        };
        let payload = cipher.decrypt(&nonce, aad)
            .expect("decryption should be ok");
        Ok(OfflineFirst {
            payload,
            sc_pk_enc: None,
            ec_pk: Some(ec_pk.as_bytes().to_vec()),
            h: Some(h.into_bytes().to_vec()),
        })
    }

    #[uniffi::constructor]
    fn deserialize(input: &[u8]) -> Result<OfflineFirst, V1CryptographicError> {
        let ec_pk = input[..32].to_vec();
        let sc_pk_enc = input[32..(64 + 16)].to_vec();
        let payload = input[(64 + 16)..].to_vec();
        Ok(OfflineFirst {
            payload,
            ec_pk: Some(ec_pk),
            sc_pk_enc: Some(sc_pk_enc),
            h: None,
        })
    }
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
    let offline_first = OfflineFirst::encrypt(
        ss_kid_pk,
        ec_kid.as_bytes().to_vec(),
        sc.as_bytes().to_vec(),
        plaintext.to_vec(),
    ).unwrap();

    let des = OfflineFirst::deserialize(
        offline_first.serialize().unwrap().as_slice());
    assert_eq!(des.unwrap(), offline_first);

    let ec_kid_pk = PublicKey::from(&ec_kid).as_bytes().to_vec();

    let ss_kid = ss_kid.as_bytes().to_vec();
    let decrypted = OfflineFirst::decrypt(
        ss_kid,
        ec_kid_pk,
        offline_first.sc_pk_enc.unwrap(),
        offline_first.payload
    ).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted.payload);
}
