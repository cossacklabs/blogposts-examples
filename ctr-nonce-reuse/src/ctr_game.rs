use std::cmp::min;
use std::ops::BitXorAssign;

use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt::Write;

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
                br#"{  "UserType": "UsualUser",     "Name": "Alex", "Surname": "S"}"#.to_vec(),
                br#"{  "UserType": "UsualUser",     "Name": "Alex", "Surname": "M"}"#.to_vec(),
                br#"{  "UserType": "UsualUser",     "Name": "Tom",  "Surname": "S"}"#.to_vec(),
                br#"{  "UserType": "Administrator", "Name": "Alex", "Surname": "S"}"#.to_vec(),
                //br#"{ "Heartbeat": "Alive",         "Since": "1653924358" }"#.to_vec(),
                br#"{  "UserType": "UsualUser",     "Name": "Alex", "Surname": "S"}"#.to_vec(),
                br#"{  "UserType": "UsualUser",     "Name": "Alex", "Surname": "M"}"#.to_vec(),
                br#"{  "UserType": "UsualUser",     "Name": "Tom",  "Surname": "S"}"#.to_vec(),
                br#"{  "UserType": "Administrator", "Name": "Alex", "Surname": "S"}"#.to_vec(),
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

        let left_random_bytes: [u8; 4] = rng.gen();
        let right_random_bytes: [u8; 4] = rng.gen();
        let mut flag_str = String::new();

        write!(
            flag_str,
            "FLAG{{{}_CtR_1sNT_SeCC_{}}}",
            Self::byte_to_hex(&left_random_bytes),
            Self::byte_to_hex(&right_random_bytes)
        )
        .expect("String formatting can not fail");

        flag_str
    }

    fn create_ciphertexts(&mut self, count: usize) {
        let mut rng = rand::thread_rng();
        self.ciphertexts_vec.clear();
        let mut chosen_plaintext: Vec<_> = self
            .plaintexts_vec
            .choose_multiple(&mut rng, count)
            .collect();
        self.known_plaintext = (*chosen_plaintext.choose(&mut rng).expect("a")).clone();
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
        flag_request.into_bytes().eq(&self.flag_plaintext)
    }

    fn encrypt_bytes(&self, plaintext_vec: &[u8]) -> Vec<u8> {
        let mut result = plaintext_vec.to_owned();

        let mut cipher = Aes128Ctr64LE::new(&self.key.into(), &self.nonce.into());
        cipher.apply_keystream(&mut result);

        result
    }

    /*
    Dead Code
    But we can leave it to show how CTR mode is decrypting
    fn decrypt_bytes(&self, ciphertext_vec: &[u8]) -> Vec<u8> {
        self.encrypt_bytes(ciphertext_vec)
    }
    */

    pub fn byte_to_hex(data: &[u8]) -> String {
        hex::encode(data)
    }

    pub fn hex_to_text(data: String) -> String {
        let decoded = hex::decode(data);
        if decoded.is_err() {
            return String::from("[unprintable]");
        }
        let decoded = decoded.expect("We have already handled the error");

        let decoded = String::from_utf8(decoded);
        if decoded.is_err() {
            return String::from("[unprintable]");
        }
        decoded.expect("We have already handled the error")
    }

    pub fn text_to_hex(data: String) -> String {
        hex::encode(data)
    }

    pub fn xor_hex(data: Vec<&String>) -> String {
        let mut data_vec = Vec::new();
        for elem in data {
            let decoded = hex::decode(elem);
            if decoded.is_err() {
                return String::from("[unprintable]");
            }
            data_vec.push(decoded.expect("We have already handled the error"));
        }
        let vec_result = Self::xor_vec(data_vec);

        hex::encode(vec_result)
    }

    pub fn xor_vec(data: Vec<Vec<u8>>) -> Vec<u8> {
        let xor_result = Vec::new();
        if data.len() < 2 {
            return xor_result;
        }
        let mut xor_result = data[0].clone();
        for elem in data.iter().skip(1) {
            let xor_result_len = min(xor_result.len(), elem.len());
            for i in 0..xor_result_len {
                xor_result[i].bitxor_assign(elem[i])
            }
            xor_result.resize(xor_result_len, 0);
        }
        xor_result
    }
}
