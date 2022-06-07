use aes_gcm::{
    aead::{consts::U16, Aead},
    Aes128Gcm, NewAead, Nonce,
};
use rand::Rng;

pub type Key = aes_gcm::Key<U16>;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 16;
const DECRYPTION_ERR: &str = "decryption error";

pub fn random_key() -> Key {
    let key: [u8; KEY_LEN] = rand::thread_rng().gen();
    *Key::from_slice(&key)
}

pub fn seal(key: &Key, data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = Aes128Gcm::new(key);

    let nonce: [u8; NONCE_LEN] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce);

    let mut ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|_| anyhow::anyhow!("encryption error"))?;

    ciphertext.extend_from_slice(nonce.as_slice());
    Ok(ciphertext)
}

pub fn open(key: &Key, ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
    if ciphertext.len() < NONCE_LEN {
        anyhow::bail!("{}", DECRYPTION_ERR);
    }
    let (ciphertext, nonce) = ciphertext.split_at(ciphertext.len() - NONCE_LEN);
    let nonce = Nonce::from_slice(nonce);
    let cipher = Aes128Gcm::new(key);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("{}", DECRYPTION_ERR))
}

#[test]
fn test_seal_open() {
    let key = random_key();
    let plaintext = b"Hello Internet people";
    let ciphertext = seal(&key, plaintext).unwrap();
    let decrypted = open(&key, &ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);
}
#[test]
fn test_seal_change_open() {
    let key = random_key();
    let plaintext = b"Hello Internet people";
    let mut ciphertext = seal(&key, plaintext).unwrap();
    ciphertext[5] ^= ciphertext[5];
    let decrypted = open(&key, &ciphertext);
    assert!(decrypted.is_err());
}
