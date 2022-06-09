use aes::cipher::block_padding::UnpadError;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use colored::Color::TrueColor;
use colored::{Color, Colorize};
use std::fmt::Write;

use prettytable::format::{FormatBuilder, LinePosition, LineSeparator};
use prettytable::{Cell, Row, Table};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

use rand::Rng;

const BLOCK_SIZE: usize = 16;

pub struct CBCGame {
    key: [u8; BLOCK_SIZE],
    iv: [u8; BLOCK_SIZE],

    known_plaintext: Vec<u8>,
    known_ciphertext: Vec<u8>,

    plaintexts_vec: Vec<Vec<u8>>,
    ciphertexts_vec: Vec<Vec<u8>>,

    cbc_same_iv_ascii_art: String,
}

impl Default for CBCGame {
    fn default() -> Self {
        Self::new()
    }
}

impl CBCGame {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            key: rng.gen(),
            iv: rng.gen(),

            known_plaintext: br#"{  "UserType": "UsualUser",     "Name": "Alex", "Surname": "S"}"#
                .to_vec(),
            known_ciphertext: vec![],

            plaintexts_vec: vec![
                br#"{  "UserType": "UsualUser",     "Name": "Alex", "Surname": "S"}"#.to_vec(),
                br#"{  "UserType": "UsualUser",     "Name": "Alex", "Surname": "M"}"#.to_vec(),
                br#"{  "UserType": "UsualUser",     "Name": "Tom",  "Surname": "S"}"#.to_vec(),
                br#"{  "UserType": "Administrator", "Name": "Alex", "Surname": "S"}"#.to_vec(),
                br#"{ "Heartbeat": "Alive",         "Since": "1653924357" }"#.to_vec(),
            ],
            ciphertexts_vec: vec![],

            cbc_same_iv_ascii_art: "

     ▄████████ ▀█████████▄   ▄████████         ▄████████    ▄████████   ▄▄▄▄███▄▄▄▄      ▄████████       ▄█   ▄█    █▄
    ███    ███   ███    ███ ███    ███        ███    ███   ███    ███ ▄██▀▀▀███▀▀▀██▄   ███    ███      ███  ███    ███
    ███    █▀    ███    ███ ███    █▀         ███    █▀    ███    ███ ███   ███   ███   ███    █▀       ███▌ ███    ███
    ███         ▄███▄▄▄██▀  ███               ███          ███    ███ ███   ███   ███  ▄███▄▄▄          ███▌ ███    ███
    ███        ▀▀███▀▀▀██▄  ███             ▀███████████ ▀███████████ ███   ███   ███ ▀▀███▀▀▀          ███▌ ███    ███
    ███    █▄    ███    ██▄ ███    █▄                ███   ███    ███ ███   ███   ███   ███    █▄       ███  ███    ███
    ███    ███   ███    ███ ███    ███         ▄█    ███   ███    ███ ███   ███   ███   ███    ███      ███  ███    ███
    ████████▀  ▄█████████▀  ████████▀        ▄████████▀    ███    █▀   ▀█   ███   █▀    ██████████      █▀    ▀██████▀

".to_string()
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let mut iter = 0;
        let mut line = String::new();

        println!("{}", self.cbc_same_iv_ascii_art.on_color(Color::Green));

        self.reinitialize_enc_dec();

        self.print_known_texts();

        while iter < self.ciphertexts_vec.len() {
            line.clear();
            println!(
                "\n{}",
                "❱".repeat(40).color(Color::Black).on_color(Color::Yellow)
            );
            println!("Input key (c(continue)/q(quit)):");
            std::io::stdin().read_line(&mut line)?;

            // remove new line char if present and leading and tracing spaces
            match line.trim().to_lowercase().as_str() {
                "q" | "quit" | "exit" => break,
                "c" | "continue" => {
                    self.example_eavesdropped_package(iter)?;
                    iter += 1;
                }
                _ => continue,
            }
        }
        println!(
            "\n{}",
            "❱".repeat(40).color(Color::Black).on_color(Color::Yellow)
        );
        println!("The end of small interactive :D");

        Ok(())
    }

    fn reinitialize_enc_dec(&mut self) {
        let mut rng = rand::thread_rng();
        self.key = rng.gen();
        self.iv = rng.gen();

        self.encrypt_all_texts();
    }

    fn encrypt_all_texts(&mut self) {
        // clear every CT in CT_vec
        for ciphertext_vec in &mut self.ciphertexts_vec {
            ciphertext_vec.clear();
        }
        self.ciphertexts_vec.clear();
        // clear known CT
        self.known_ciphertext.clear();

        self.known_ciphertext = self.encrypt_text(&self.known_plaintext);

        for text_vec in &self.plaintexts_vec {
            let encrypted_value = self.encrypt_text(text_vec);
            self.ciphertexts_vec.push(encrypted_value);
        }
    }

    fn encrypt_text(&self, plaintext_vec: &[u8]) -> Vec<u8> {
        Aes128CbcEnc::new(&self.key.into(), &self.iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext_vec)
    }

    fn decrypt_text(&self, ciphertext_vec: &[u8]) -> Result<Vec<u8>, UnpadError> {
        Aes128CbcDec::new(&self.key.into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext_vec)
    }

    fn print_known_texts(&self) {
        let (formatted_blocks_pt, formatted_known_pt) =
            CBCGame::format_blocks_utf8(&self.known_plaintext);
        let (formatted_blocks_ct, formatted_known_ct) =
            CBCGame::format_blocks_hex(&self.known_ciphertext);

        println!("Getting corresponding known CipherText to leaked PlainText:");
        println!("Here's leaked plaintext:");
        println!("{formatted_blocks_pt}");
        println!("{formatted_known_pt}\n");
        println!("Here's corresponding CipherText to leaked PlainText:");
        println!("{formatted_blocks_ct}");
        println!("{formatted_known_ct}\n\n");

        println!("Let's wait and eavesdrop other CipherTexts:");
        for eavesdropped_ciphertext in &self.ciphertexts_vec {
            let formatted_ct = CBCGame::format_blocks_hex(eavesdropped_ciphertext).1;
            println!("{formatted_ct}");
        }
    }

    fn example_eavesdropped_package(&self, iter: usize) -> anyhow::Result<()> {
        // Stored strings with description for every iteration.
        let (example_desc_start, example_desc_difference, ascii_art) =
            CBCGame::get_desc_strings(iter);

        // Let's take an example eavesdropped packet
        // And decrypt it so we can assure, that our presumptions are correct
        let eavesdropped_packet = &self.ciphertexts_vec[iter];
        let decrypted_eavesdrop = &self
            .decrypt_text(eavesdropped_packet)
            .expect("Can not fail, as we are decrypting value, that we had encrypted");

        // Do some formatting staff
        // Format and prettify Known CT and PT
        // Also do the same procedures to an eavesdropped packets
        let (formatted_blocks_pt, formatted_known_pt) =
            CBCGame::format_blocks_utf8(&self.known_plaintext);
        let (formatted_blocks_ct, formatted_known_ct) =
            CBCGame::format_blocks_hex(&self.known_ciphertext);
        let formatted_eavesdropped_ct = CBCGame::format_blocks_hex(eavesdropped_packet).1;
        let formatted_decrypted_eavesdrop = CBCGame::format_blocks_utf8(decrypted_eavesdrop).1;
        let (blocks_status_utf8, blocks_status_hex) = self.get_blocks_status(iter);

        // Let's create ascii tables for each step
        let ascii_table_step0 = CBCGame::create_table(
            "Step #0",
            vec![vec![&format!(
                "\n{}\n{}\n{}\n{}\n\n",
                "Given variables:",
                &format!(
                    "{}  - known PlainText",
                    String::from_utf8_lossy(&self.known_plaintext)
                ),
                &format!(
                    "{}  - known CipherText",
                    hex::encode(&self.known_ciphertext)
                ),
                &format!(
                    "{}  - eavesdropped CipherText",
                    hex::encode(eavesdropped_packet)
                ),
            )]],
            3,
        )?;

        let ascii_table_step1 = CBCGame::create_table(
            "Step 1:",
            vec![vec![&format!(
                "\n{}\n{}\n{}\n{}\n{}\n\n",
                &"Let's compare known CipherText with the leaked one:".to_string(),
                &formatted_blocks_ct,
                &format!("{}  - known CipherText", formatted_known_ct),
                &format!("{}  - eavesdropped CipherText", formatted_eavesdropped_ct),
                &blocks_status_hex
            )]],
            3,
        )?;

        let ascii_table_step2 = CBCGame::create_table(
            "",
            vec![vec![&format!(
                "\n{}\n{}\n{}\n{}\n{}\n\n",
                &"| ✨✨ Let's bring some magic and decrypt eavesdropped packet: ✨✨ |"
                    .to_string(),
                &formatted_blocks_pt,
                &format!("{}  - known PlainText", formatted_known_pt),
                &format!(
                    "{}  - eavesdropped PlainText",
                    formatted_decrypted_eavesdrop
                ),
                &blocks_status_utf8
            )]],
            3,
        )?;

        // Print formed output of current iteration
        println!("{}", ascii_art.on_color(Color::Green));
        println!("\n{}:", example_desc_start.bold());
        println!();

        println!("{}", ascii_table_step0);
        println!();

        println!("{}", ascii_table_step1);
        println!();

        println!("{example_desc_difference}");
        println!("{}", ascii_table_step2);
        println!();

        Ok(())
    }

    // outputting beautified string, with painted blocks
    // it shows on each step which block stays the same,
    // which one is different, and which one in unknown state
    // Example output on iter 2:
    // |----- Same -----|----- Same -----|++++ Differs +++|✕✕✕✕ UnKnown ✕✕✕|
    fn get_blocks_status(&self, iter: usize) -> (String, String) {
        let mut blocks_status_utf8 = String::new();
        let mut blocks_status_hex = String::new();

        let orange = TrueColor {
            r: 255,
            g: 140,
            b: 0,
        };

        write!(
            blocks_status_utf8,
            "{}{}{}{}",
            "|----- Same -----"
                .repeat(self.ciphertexts_vec.len() - iter - 1)
                .on_color(Color::Green),
            "|++++ Differs +++"
                .repeat(if iter == 0 { 0 } else { 1 })
                .on_color(orange),
            "|✕✕✕✕ UnKnown ✕✕✕"
                .repeat(if iter == 0 { 0 } else { iter - 1 })
                .on_color(Color::Red),
            "|".on_color(if iter == 0 {
                Color::Green
            } else if iter == 1 {
                orange
            } else {
                Color::Red
            })
        )
        .expect("Write to a string cannot fail");
        write!(
            blocks_status_hex,
            "{}{}{}{}",
            "|------------- Same -------------"
                .repeat(self.ciphertexts_vec.len() - iter - 1)
                .on_color(Color::Green),
            "|+++++++++++++ Differs ++++++++++"
                .repeat(if iter == 0 { 0 } else { 1 })
                .on_color(orange),
            "|✕✕✕✕✕✕✕✕✕✕✕✕✕ UnKnown ✕✕✕✕✕✕✕✕✕✕"
                .repeat(if iter == 0 { 0 } else { iter - 1 })
                .on_color(Color::Red),
            "|".on_color(if iter == 0 {
                Color::Green
            } else if iter == 1 {
                orange
            } else {
                Color::Red
            })
        )
        .expect("Write to a string cannot fail");
        (blocks_status_utf8, blocks_status_hex)
    }

    fn get_desc_strings(iter: usize) -> (String, String, String) {
        match iter {
            0 => (
                "Example with the same package being intercepted".to_string(),
                "As we can see all blocks are the same. That's why we can definitely tell, that \
                message in the eavesdropped packet is the same as the leaked one!".to_string(),
                "

      ▄████████    ▄████████   ▄▄▄▄███▄▄▄▄      ▄████████
      ███    ███   ███    ███ ▄██▀▀▀███▀▀▀██▄   ███    ███
      ███    █▀    ███    ███ ███   ███   ███   ███    █▀
      ███          ███    ███ ███   ███   ███  ▄███▄▄▄
    ▀███████████ ▀███████████ ███   ███   ███ ▀▀███▀▀▀
             ███   ███    ███ ███   ███   ███   ███    █▄
       ▄█    ███   ███    ███ ███   ███   ███   ███    ███
     ▄████████▀    ███    █▀   ▀█   ███   █▀    ██████████
                                                       ".to_string()
            ),
            1 => (
                "Example with package that has the different surname field being intercepted"
                    .to_string(),
                "Only the last block differs. We can assume that data in the last block has \
                changed. Also keeping in mind, that the first block is the same, so structure \
                of json can be the same. That's why we can guess that only surname has changed..."
                    .to_string(),
                "
    ▄▄▄▄▄▄▄▄▄▄   ▄            ▄▄▄▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄▄▄  ▄    ▄         ▄         ▄  ▄▄▄▄▄▄▄▄▄▄▄
   ▐░░░░░░░░░░▌ ▐░▌          ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌▐░░░░░░░░░░░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░█▀▀▀▀▀▀▀█░▌▐░█▀▀▀▀▀▀▀▀▀ ▐░▌ ▐░▌       ▄█░█▄▄▄▄▄▄▄█░█▄▀▀▀▀▀▀▀▀▀█░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌        ▐░▌
   ▐░█▄▄▄▄▄▄▄█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▀█░█▀▀▀▀▀▀▀█░█▀▄▄▄▄▄▄▄▄▄█░▌
   ▐░░░░░░░░░░▌ ▐░▌          ▐░▌       ▐░▌▐░▌          ▐░░▌           ▐░▌       ▐░▌▐░░░░░░░░░░░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▄█░█▄▄▄▄▄▄▄█░█▄▀▀▀▀▀▀▀▀▀█░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌        ▐░▌
   ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░▌ ▐░▌       ▀█░█▀▀▀▀▀▀▀█░█▀▄▄▄▄▄▄▄▄▄█░▌
   ▐░░░░░░░░░░▌ ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌▐░░░░░░░░░░░▌
    ▀▀▀▀▀▀▀▀▀▀   ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀    ▀         ▀         ▀  ▀▀▀▀▀▀▀▀▀▀▀"
                    .to_string()
            ),
            2 => (
                "Example with package that has the different name field being intercepted"
                    .to_string(),
                "Block #2 differs. We can assume that data in that block has changed. Also keeping \
                in mind, that the first block is the same, so structure of json can be the same. \
                That's why we can guess that name has changed and maybe blocks after that..."
                    .to_string(),
                "
    ▄▄▄▄▄▄▄▄▄▄   ▄            ▄▄▄▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄▄▄  ▄    ▄         ▄         ▄    ▄▄▄▄▄▄▄▄▄▄▄
   ▐░░░░░░░░░░▌ ▐░▌          ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌  ▐░░░░░░░░░░░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░█▀▀▀▀▀▀▀█░▌▐░█▀▀▀▀▀▀▀▀▀ ▐░▌ ▐░▌       ▄█░█▄▄▄▄▄▄▄█░█▄  ▀▀▀▀▀▀▀▀▀█░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌          ▐░▌
   ▐░█▄▄▄▄▄▄▄█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▀█░█▀▀▀▀▀▀▀█░█▀           ▐░▌
   ▐░░░░░░░░░░▌ ▐░▌          ▐░▌       ▐░▌▐░▌          ▐░░▌           ▐░▌       ▐░▌   ▄▄▄▄▄▄▄▄▄█░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▄█░█▄▄▄▄▄▄▄█░█▄ ▐░░░░░░░░░░░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌▐░█▀▀▀▀▀▀▀▀▀
   ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░▌ ▐░▌       ▀█░█▀▀▀▀▀▀▀█░█▀ ▐░█▄▄▄▄▄▄▄▄▄
   ▐░░░░░░░░░░▌ ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌  ▐░░░░░░░░░░░▌
    ▀▀▀▀▀▀▀▀▀▀   ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀    ▀         ▀         ▀    ▀▀▀▀▀▀▀▀▀▀▀"
                    .to_string()
            ),
            3 => (
                "Example with package that has the different name field being intercepted"
                    .to_string(),
                "Block #1 differs. We can assume that data in that block has changed. Also keeping \
                in mind, that the first block is the same, so structure of json can be the same. \
                That's why we can guess that usertype has changed and maybe blocks after that..."
                    .to_string(),
                "
    ▄▄▄▄▄▄▄▄▄▄   ▄            ▄▄▄▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄▄▄  ▄    ▄         ▄         ▄     ▄▄▄▄
   ▐░░░░░░░░░░▌ ▐░▌          ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌  ▄█░░░░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░█▀▀▀▀▀▀▀█░▌▐░█▀▀▀▀▀▀▀▀▀ ▐░▌ ▐░▌       ▄█░█▄▄▄▄▄▄▄█░█▄▐░░▌▐░░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌▀▀ ▐░░▌
   ▐░█▄▄▄▄▄▄▄█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▀█░█▀▀▀▀▀▀▀█░█▀    ▐░░▌
   ▐░░░░░░░░░░▌ ▐░▌          ▐░▌       ▐░▌▐░▌          ▐░░▌           ▐░▌       ▐░▌     ▐░░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▄█░█▄▄▄▄▄▄▄█░█▄    ▐░░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌   ▐░░▌
   ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░▌ ▐░▌       ▀█░█▀▀▀▀▀▀▀█░█▀▄▄▄▄█░░█▄▄▄
   ▐░░░░░░░░░░▌ ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌▐░░░░░░░░░░░▌
    ▀▀▀▀▀▀▀▀▀▀   ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀    ▀         ▀         ▀  ▀▀▀▀▀▀▀▀▀▀▀"
                    .to_string()
            ),
            4 => (
                "Example with package that has the different name field being intercepted"
                    .to_string(),
                "Block #0 differs. We can assume that data in that block has changed. We don't \
                know whether if data changed in other blocks or not. But if first block differs - \
                we can guess that data structure is different this time...".to_string(),
                "
    ▄▄▄▄▄▄▄▄▄▄   ▄            ▄▄▄▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄▄▄  ▄    ▄         ▄         ▄     ▄▄▄▄▄▄▄▄▄
   ▐░░░░░░░░░░▌ ▐░▌          ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌   ▐░░░░░░░░░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░█▀▀▀▀▀▀▀█░▌▐░█▀▀▀▀▀▀▀▀▀ ▐░▌ ▐░▌       ▄█░█▄▄▄▄▄▄▄█░█▄ ▐░█░█▀▀▀▀▀█░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌▐░▌▐░▌    ▐░▌
   ▐░█▄▄▄▄▄▄▄█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▀█░█▀▀▀▀▀▀▀█░█▀ ▐░▌ ▐░▌   ▐░▌
   ▐░░░░░░░░░░▌ ▐░▌          ▐░▌       ▐░▌▐░▌          ▐░░▌           ▐░▌       ▐░▌  ▐░▌  ▐░▌  ▐░▌
   ▐░█▀▀▀▀▀▀▀█░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌░▌         ▄█░█▄▄▄▄▄▄▄█░█▄ ▐░▌   ▐░▌ ▐░▌
   ▐░▌       ▐░▌▐░▌          ▐░▌       ▐░▌▐░▌          ▐░▌▐░▌       ▐░░░░░░░░░░░░░░░▌▐░▌    ▐░▌▐░▌
   ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░█▄▄▄▄▄▄▄█░▌▐░█▄▄▄▄▄▄▄▄▄ ▐░▌ ▐░▌       ▀█░█▀▀▀▀▀▀▀█░█▀ ▐░█▄▄▄▄▄█░█░▌
   ▐░░░░░░░░░░▌ ▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░░░░░░░░░░░▌▐░▌  ▐░▌       ▐░▌       ▐░▌   ▐░░░░░░░░░▌
    ▀▀▀▀▀▀▀▀▀▀   ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀▀  ▀    ▀         ▀         ▀     ▀▀▀▀▀▀▀▀▀"
                    .to_string()
            ),
            _ => (
                "What are you doing here, step-bro??? UwU".to_string(),
                "Okay, here's some small secret crypto-tip for you:".to_string(),
                "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()
            )
        }
    }

    // Creating beautified ascii table
    // User must send strings (lines), length of each line and padding
    // We need to use old (3 years ago last commit) crate, because new one
    // Can't handle Colorized strings properly
    fn create_table(
        title: &str,
        data: Vec<Vec<&String>>,
        padding: usize,
    ) -> anyhow::Result<String> {
        // At first let's create and build custom styles for title and main table
        let main_table_format = FormatBuilder::new()
            .column_separator('║')
            .borders('║')
            .separators(&[LinePosition::Top], LineSeparator::new('═', '╦', '╔', '╗'))
            .separators(
                &[LinePosition::Intern],
                LineSeparator::new('═', '╬', '╠', '╣'),
            )
            .separators(
                &[LinePosition::Bottom],
                LineSeparator::new('═', '╩', '╚', '╝'),
            )
            .padding(padding, padding)
            .build();

        let title_table_format = FormatBuilder::new()
            .column_separator('║')
            .borders('║')
            .separators(&[LinePosition::Top], LineSeparator::new('═', '╦', '╔', '╗'))
            .separators(
                &[LinePosition::Intern],
                LineSeparator::new('═', '╬', '╠', '╣'),
            )
            .padding(padding, padding)
            .build();

        // Then let's create title table
        let mut title_table = Table::new();
        title_table.set_format(title_table_format);
        // And set it with content
        title_table.add_row(Row::new(vec![Cell::new(title)]));

        // Creating main table
        let mut main_table = Table::new();
        main_table.set_format(main_table_format);

        // Filling data into main table
        for row in data {
            // Fill row with cells
            let mut cells_vector = vec![];

            for cell in row {
                cells_vector.push(Cell::new(cell.as_str()));
            }

            // Create new row and add it into main table
            main_table.add_row(Row::new(cells_vector));
        }
        // Get main table as string variable
        let mut main_table_str = main_table.to_string();
        // And remove 'new line char' at the end of the string
        main_table_str.pop();

        // If there is no title => return only main table
        if title.is_empty() {
            Ok(main_table_str)
        }
        // Otherwise we need to pair them
        else {
            // Replace first char of the main table with '╠' char
            main_table_str.replace_range(
                main_table_str
                    .char_indices()
                    .next()
                    .map(|(pos, ch)| (pos..pos + ch.len_utf8()))
                    .expect("Any table should have at least 4 chars + new line character"),
                "╠",
            );

            // Replace one of top characters with '╩'
            // So we can smoothly pair title and main table
            let end_of_title_table = (title_table.to_string().find('\n').expect("")) / 3;
            main_table_str.replace_range(
                main_table_str
                    .char_indices()
                    .nth(end_of_title_table - 1)
                    .map(|(pos, ch)| (pos..pos + ch.len_utf8()))
                    .expect("Any table should have at least one new line character"),
                "╩",
            );

            Ok(format!("{title_table}{main_table_str}"))
        }
    }

    // Formatting PT:
    // input_vec: input plaintext as vector
    // block_size: size of chunks
    // Example:                 Here's my small plaintext as an input to this function
    // format_blocks_utf8.0:   |    Block #0    |    Block #1    |    Block #2    |    Block #3    |
    // format_blocks_utf8.1:   |Here's my small |plaintext as an |input to this fu|nction██████████|
    fn format_blocks_utf8(input_vec: &[u8]) -> (String, String) {
        let mut formatted_blocks = String::new();
        let mut formatted_input = String::new();

        for (block_num, block) in input_vec.chunks(BLOCK_SIZE).enumerate() {
            write!(formatted_blocks, "|    Block #{}    ", block_num)
                .expect("Write to a string cannot fail");
            write!(
                formatted_input,
                "|{}{}",
                String::from_utf8_lossy(block),
                "█".repeat(BLOCK_SIZE - block.len())
            )
            .expect("Write to a string cannot fail");
        }
        formatted_blocks.push('|');
        formatted_input.push('|');

        (formatted_blocks, formatted_input)
    }

    // Same as format_blocks_utf8, but created for formatting CTs
    // It converts input_vec to hex-encoded string
    fn format_blocks_hex(input_vec: &[u8]) -> (String, String) {
        let mut formatted_blocks = String::new();
        let mut formatted_input = String::new();

        for (block_num, block) in input_vec.chunks(BLOCK_SIZE).enumerate() {
            write!(
                formatted_blocks,
                "|            Block #{}            ",
                block_num
            )
            .expect("Write to a string cannot fail");
            write!(
                formatted_input,
                "|{}{}",
                hex::encode(block),
                "█".repeat(BLOCK_SIZE - block.len())
            )
            .expect("Write to a string cannot fail");
        }
        formatted_blocks.push('|');
        formatted_input.push('|');

        (formatted_blocks, formatted_input)
    }
}
