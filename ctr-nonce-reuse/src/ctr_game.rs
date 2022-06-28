use std::cmp::min;
use std::ops::BitXorAssign;

use rand::seq::SliceRandom;
use rand::Rng;

use aes::cipher::{KeyIvInit, StreamCipher};

type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes128>;

const BLOCK_SIZE: usize = 16;
const EAVESDROPPED_MSG_COUNT: usize = 3;

pub struct CtrGame {
    key: [u8; BLOCK_SIZE],
    nonce: [u8; BLOCK_SIZE],

    known_plaintext: Vec<u8>,
    flag_plaintext: Vec<u8>,

    plaintexts_vec: Vec<Vec<u8>>,
    ciphertexts_vec: Vec<Vec<u8>>,
}

impl Default for CtrGame {
    fn default() -> Self {
        Self::new()
    }
}

impl CtrGame {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            key: rng.gen(),
            nonce: rng.gen(),

            known_plaintext: vec![],
            flag_plaintext: vec![],

            plaintexts_vec: vec![
                // Ukraine
                br#"Free Ukraine, Donate on u24.gov.ua !"#.to_vec(),
                // Ukrainian Quotes
                br#"Only struggle means to live (c) Ivan Franko"#.to_vec(),
                br#"Contra spem spero (c) Lesya Ukrainka"#.to_vec(),
                br#"The world chased, but never caught me (c) Skovoroda"#.to_vec(),
                br#"When I am dead, bury me In my beloved Ukraine (c) Shevchenko"#.to_vec(),
                // Churchill
                br#"We shall never surrender. We shall go on to the end."#.to_vec(),
                br#"We shall fight in Kherson and Mariupol..."#.to_vec(),
                br#"We shall fight on the beaches of Odesa"#.to_vec(),
                br#"We shall fight on the seas of Azov and Black Sea."#.to_vec(),
                br#"We shall fight on the landing grounds (Hostomel)"#.to_vec(),
                // 1984
                br#"Who controls the past controls the future"#.to_vec(),
                br#"Who controls the present controls the past"#.to_vec(),
                br#"War is peace. Freedom is slavery"#.to_vec(),
                br#"Freedom is slavery. Ignorance is strength"#.to_vec(),
                br#"If you want to keep a secret, you must also hide it from yourself"#.to_vec(),
                br#"We shall meet in the place where there is no darkness"#.to_vec(),
                br#"In the face of pain there are no heroes"#.to_vec(),
                br#"Big Brother is Watching You! Even in free countries"#.to_vec(),
                br#"It's a beautiful thing, the destruction of words"#.to_vec(),
                // Crypto
                br#"You should bought bitcoin in 2009"#.to_vec(),
                br#"This mode is so unsecure :doggo-face:"#.to_vec(),
                br#"This is my neighbour, Nushuktan Akbai"#.to_vec(),
                br#"Veni, vidi, crack it (Caesar about CTR)"#.to_vec(),
                br#"Who owns the information, he owns the world"#.to_vec(),
                br#"Payne, I can't feel my security.."#.to_vec(),
                br#"Bubba, that's cuz security ain't there"#.to_vec(),
                br#"Do you still think that bare CTR or CBC is a good idea?"#.to_vec(),
            ],
            ciphertexts_vec: vec![],
        }
    }

    pub fn restart(&mut self) {
        let mut rng = rand::thread_rng();

        self.key = rng.gen();
        self.nonce = rng.gen();

        let flag_str = self.create_random_flag();
        self.flag_plaintext = flag_str.into();

        self.create_ciphertexts(EAVESDROPPED_MSG_COUNT);
    }

    fn create_random_flag(&self) -> String {
        let mut rng = rand::thread_rng();

        let left_random_bytes: [u8; 2] = rng.gen();
        let right_random_bytes: [u8; 2] = rng.gen();
        let flag_str = format!(
            "FLAG{{{}_CtR_1sNT_SeCC_{}}}",
            Self::byte_to_hex(&left_random_bytes),
            Self::byte_to_hex(&right_random_bytes)
        );

        flag_str
    }

    fn create_ciphertexts(&mut self, count: usize) {
        let mut rng = rand::thread_rng();
        self.ciphertexts_vec.clear();
        let mut chosen_plaintext: Vec<_> = self
            .plaintexts_vec
            .choose_multiple(&mut rng, count)
            .collect();
        self.known_plaintext = chosen_plaintext
            .choose(&mut rng)
            .expect("non-empty")
            .to_vec();
        chosen_plaintext.push(&self.flag_plaintext);
        chosen_plaintext.shuffle(&mut rng);
        for plaintext in chosen_plaintext {
            let ciphertext = self.encrypt_bytes(plaintext);
            self.ciphertexts_vec.push(ciphertext);
        }
    }

    pub fn get_hex_ciphertexts(&self) -> Vec<String> {
        let mut result = Vec::new();

        for ciphertext in &self.ciphertexts_vec {
            result.push(Self::byte_to_hex(ciphertext));
        }

        result
    }

    pub fn get_known_plaintext(&self) -> String {
        String::from_utf8_lossy(&self.known_plaintext).to_string()
    }

    pub fn submit_flag(&self, flag_request: String) -> bool {
        flag_request.as_bytes() == self.flag_plaintext
    }

    fn encrypt_bytes(&self, plaintext_vec: &[u8]) -> Vec<u8> {
        let mut result = plaintext_vec.to_owned();

        let mut cipher = Aes128Ctr64LE::new(&self.key.into(), &self.nonce.into());
        cipher.apply_keystream(&mut result);

        result
    }

    pub fn decrypt_bytes(&self, ciphertext_vec: &[u8]) -> Vec<u8> {
        self.encrypt_bytes(ciphertext_vec)
    }

    pub fn byte_to_hex(data: &[u8]) -> String {
        hex::encode(data)
    }

    pub fn hex_to_text(data: String) -> String {
        hex::decode(data)
            .ok()
            .and_then(|data| String::from_utf8(data).ok())
            .unwrap_or_else(|| String::from("[unprintable]"))
    }

    pub fn text_to_hex(data: String) -> String {
        hex::encode(data)
    }

    pub fn xor_hex(data: Vec<&String>) -> String {
        let mut data_vec = Vec::new();
        for elem in data {
            let decoded = match hex::decode(elem) {
                Ok(decoded) => decoded,
                Err(_) => {
                    return String::from("[unprintable]");
                }
            };
            data_vec.push(decoded);
        }
        let vec_result = Self::xor_vec(data_vec);

        hex::encode(vec_result)
    }

    pub fn xor_vec(mut data: Vec<Vec<u8>>) -> Vec<u8> {
        let xor_result = Vec::new();
        if data.len() < 2 {
            return xor_result;
        }
        let mut xor_result = data.pop().expect("length is checked");
        for elem in data.iter() {
            let xor_result_len = min(xor_result.len(), elem.len());
            for i in 0..xor_result_len {
                xor_result[i].bitxor_assign(elem[i])
            }
            xor_result.resize(xor_result_len, 0);
        }
        xor_result
    }
}
