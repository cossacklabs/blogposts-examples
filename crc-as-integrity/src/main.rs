use std::fmt::Display;

use aes::cipher::{KeyIvInit, StreamCipher};
use clap::Parser;
use crc::Crc;
use rand::Rng;
use shell::Shell;
use yansi::Paint;

type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes128>;
const CRC_ALGO: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
const CRC_LEN: usize = 2;

const EXPECTED_CMD: &str = "detonate";
const DONT_ROLL_YOUR_CRYPTO: &str = "don't roll your own crypto";

const EXPLOSION: &str = r#"
           _.-^^---....,,--
       _--                  --_
       <                        >)
       |                         |
       \._                   _./
          ```--. . , ; .--'''
                | |   |
             .-=||  | |=-.
             `-=#$%&%$#=-'
                | ;  :|
____________.,-#%&$@%#&#~,.____________"#;
const SECRET: &str = "c7278ae7d828b59196d9007ee679695ae66bbbc35dbf5c8ee689574e1e196258eb090aa2d14319d11b3a77791ca836e65282e7b3cd25be1e2051341faadc93ebd13eafb1e6ffab0ecf2c1c68d32c8c";
const SECRET_KEY: &str = "d366944d67314d26f1d6e04e212707ce";
const SECRET_IV: &str = "5ddbca6b7f03c7d4df1d9c73d2ebcd21";

#[derive(Debug)]
struct HexString(Vec<u8>);

impl std::str::FromStr for HexString {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex::decode(s).map(Self)
    }
}

mod shell;

fn encrypt(data: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut cipher = Aes128Ctr64LE::new(key.into(), iv.into());
    cipher.apply_keystream(data);
}

fn decrypt(data: &mut [u8], key: &[u8], iv: &[u8]) {
    // hehe
    encrypt(data, key, iv)
}

fn crc(data: &[u8]) -> [u8; CRC_LEN] {
    CRC_ALGO.checksum(data).to_be_bytes()
}

fn seal_packet(data: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut data = data.to_owned();
    encrypt(&mut data, key, iv);
    let crc = crc(&data);
    data.extend_from_slice(&crc);
    data
}

fn open_packet(data: &[u8], key: &[u8], iv: &[u8]) -> anyhow::Result<Vec<u8>> {
    if data.len() < CRC_LEN {
        anyhow::bail!("length is too small");
    }

    let (data, crc_bytes) = data.split_at(data.len() - CRC_LEN);
    if crc_bytes != crc(data) {
        anyhow::bail!("decryption error");
    }

    let mut data = data.to_owned();
    decrypt(&mut data, key, iv);
    Ok(data)
}

fn xor(a: &mut [u8], b: &[u8]) {
    a.iter_mut().zip(b).for_each(|(a, b)| *a ^= *b);
}

#[derive(Parser, Debug)]
#[clap(about = None, long_about = None)]
enum Commands {
    /// Xor two hex-strings together
    Xor {
        /// First hex-encoded string
        a: HexString,
        /// Second hex-encoded string
        b: HexString,
    },
    /// Send encrypted command to the server in a hex format.
    Send {
        /// Hex-encoded string
        hex: HexString,
    },
    /// Calculate crc on a hex-string
    Crc {
        /// Hex-encoded string
        hex: HexString,
    },
    /// Convert string into hex
    Hex {
        /// utf8 string
        string: String,
    },

    /// Print intercepted packet
    Intercept,
}

fn main() -> anyhow::Result<()> {
    let key: [u8; 16] = rand::thread_rng().gen();
    let iv: [u8; 16] = rand::thread_rng().gen();

    let intercepted = seal_packet(DONT_ROLL_YOUR_CRYPTO.as_bytes(), &key, &iv);

    print_greetings(&intercepted);

    let shell = Shell::<Commands>::new("~> ");

    shell.start_loop(|cmd| {
        match cmd {
            Commands::Xor {
                a: HexString(a),
                b: HexString(b),
            } => xor_cmd(a, b),
            Commands::Send {
                hex: HexString(data),
            } => send_cmd(data, &key, &iv)?,
            Commands::Crc {
                hex: HexString(data),
            } => println!("{}", hex::encode(crc(&data))),
            Commands::Hex { string } => println!("{}", hex::encode(string)),
            Commands::Intercept => print_intercepted(&intercepted),
        }
        Ok(())
    })?;

    Ok(())
}

fn print_intercepted(intercepted: &[u8]) {
    println!(
        "Congratulations! You've intercepted a packet with command {:?}",
        Paint::green(DONT_ROLL_YOUR_CRYPTO)
    );
    println!("The packet is encrypted in CTR mode with 16-bit crc check appended");
    println!("after the packet:");
    println!();
    println!("  {}", Sealed(intercepted));
    println!();
}

fn send_cmd(data: Vec<u8>, key: &[u8], iv: &[u8]) -> anyhow::Result<()> {
    let command = open_packet(&data, key, iv)?;
    match command {
        cmd if cmd == EXPECTED_CMD.as_bytes() => {
            print_secret();
        }
        cmd if cmd == DONT_ROLL_YOUR_CRYPTO.as_bytes() => {
            println!("why not? :)");
        }
        _ => {
            anyhow::bail!("don't know what are you talking about");
        }
    }
    Ok(())
}

fn xor_cmd(mut a: Vec<u8>, b: Vec<u8>) {
    println!();
    println!("  {} xor", Paint::green(hex::encode(&a)));
    println!("  {} = ", Paint::green(hex::encode(&b)));
    xor(&mut a, &b);
    println!("  {}", Paint::green(hex::encode(&a)));
    println!();
}

fn print_secret() {
    let key = hex::decode(SECRET_KEY).expect("checked");
    let iv = hex::decode(SECRET_IV).expect("checked");
    let mut secret = hex::decode(SECRET).expect("checked");
    decrypt(&mut secret, &key, &iv);

    let footer = "---------------------------------------";
    println!("{}", Paint::yellow(EXPLOSION).bold());
    println!();
    println!("_____________ Memory dump _____________");

    for chunk in secret.chunks(footer.len() - 2) {
        let padding = footer.len() - 2 - chunk.len();
        println!(
            "|{}{}|",
            String::from_utf8_lossy(chunk),
            " ".repeat(padding)
        );
    }
    println!("{}", footer);
    println!();
}

fn print_greetings(intercepted: &[u8]) {
    print_intercepted(intercepted);
    println!(
        "Your next task: forge {:?} command.",
        Paint::green(EXPECTED_CMD)
    );
    println!("Commands");
    println!("  crc          Calculate crc on a hex-string");
    println!("  help         Print this message or the help of the given subcommand(s)");
    println!("  hex          Convert string into hex");
    println!("  intercept    Print intercepted packet");
    println!("  send         Send encrypted command to the server in a hex format");
    println!("  xor          Xor two hex-strings together");
    println!();
}

struct Sealed<'a>(&'a [u8]);

impl<'a> Display for Sealed<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (data, crc) = self.0.split_at(self.0.len() - CRC_LEN);
        let data = hex::encode(data);
        let crc = hex::encode(crc);
        write!(f, "{}{}", Paint::green(data), Paint::blue(crc))
    }
}
