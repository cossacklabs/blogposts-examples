use aes::cipher::KeyIvInit;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut};

use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub const BLOCK_SIZE: usize = 16;

/// An Oracle is a `black box` which can only tell whether some ciphertext
/// is a valid cbc or not.
pub struct Oracle {
    key: [u8; 16],
    iv: [u8; 16],

    counter: AtomicUsize,
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

impl Oracle {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            key: rng.gen(),
            iv: rng.gen(),
            counter: AtomicUsize::new(0),
        }
    }

    pub fn iv(&self) -> [u8; 16] {
        self.iv
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        Aes128CbcEnc::new(&self.key.into(), &self.iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data)
    }

    /// Returns true if decryption is ok (which means padding is ok)
    pub fn query_decryption(&self, ciphertext: &[u8]) -> bool {
        self.counter.fetch_add(1, Ordering::Relaxed);
        Aes128CbcDec::new(&self.key.into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .is_ok()
    }

    /// Returns the number of queries
    pub fn counter(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Starting state
    Start,
    /// Prompting use to start bruteforcing
    ReadyToIterate,
    /// The bruteforcing a byte by quering to the oracle
    IteratingByte,
    /// Found a byte that produces valid padding
    FoundByte,
    /// Deriving plaintext byte from the our knowledge about the padding
    CalculatingPlainByte,
    /// Reloading `machine.counter` to find the next byte
    ReloadingCounter,
    /// Reloading the next block from `machine.ciphertext`.
    ReloadingBlock,
    /// Finished decryption
    Finished,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start => write!(f, "Start"),
            Self::ReadyToIterate => write!(f, "Ready to iterate"),
            Self::IteratingByte => write!(f, "Iterating"),
            Self::FoundByte => write!(f, "Found a hit"),
            Self::CalculatingPlainByte => write!(f, "Calculating plaintext byte"),
            Self::ReloadingCounter => write!(f, "Reloading counter"),
            Self::ReloadingBlock => write!(f, "Reloading block"),
            Self::Finished => write!(f, "Finished"),
        }
    }
}

/// DecryptingMachine is a state machine, which if given oracle and ciphertext,
/// will be able to decrypt it without the key.
pub struct DecryptingMachine {
    /// Current block that we are decrypting
    pub block: Vec<u8>,
    /// iv or previous block, because that's how cbc works
    pub iv: Vec<u8>,
    /// Already decrypted bytes from the current block
    pub known: Vec<u8>,
    /// Padding we try to hit
    pub padding: u8,
    /// Our custom crafted block, that we use as iv
    pub counter: Vec<u8>,
    /// If Some(ok), machine is in the `FoundByte` state
    pub ok: Option<bool>,

    /// Our padding oracle. Note that we never use its key directly
    pub oracle: Oracle,
    /// Full ciphertext that we want to decrypt
    pub ciphertext: Vec<u8>,
    /// Index of block that we are decrypting. The block itself is in `self.block`
    pub block_num: Option<usize>,
    /// Already decrypted text
    pub decrypted: Vec<u8>,

    /// The state of the machine
    pub state: State,
}

impl DecryptingMachine {
    pub fn new(oracle: Oracle, ciphertext: Vec<u8>) -> Self {
        Self {
            block: vec![],
            iv: vec![],
            known: vec![],
            padding: 0x01,
            counter: vec![0; BLOCK_SIZE],
            ok: None,

            oracle,
            ciphertext,
            block_num: None,
            decrypted: vec![],
            state: State::Start,
        }
    }

    fn check(&self) -> bool {
        // Append as a first block, so the rest will not be affected by the iv
        let zeroes = [0; BLOCK_SIZE];
        let ciphertext = [&zeroes[..], &self.counter, &self.block].concat();
        let ok = self.oracle.query_decryption(&ciphertext);
        // Double check padding when 0x01
        // This is because when wy try to iterate and hit 0x01,
        // we may hot any other padding, for example 0x02 0x02.
        // We don't know for sure, so change the next byte and try again
        if ok && self.padding == 0x01 {
            let mut counter = self.counter.clone();
            // Increase second to last byte, so if it was 0x02 0x02
            // It would be 0x03 0x02, and the second decryption will fail
            // But if the padding is 0x01, changing the second to last byte
            // will not do anything
            counter[BLOCK_SIZE - 2] = counter[BLOCK_SIZE - 2].wrapping_add(1);
            let ciphertext = [&zeroes[..], &counter, &self.block].concat();
            self.oracle.query_decryption(&ciphertext)
        } else {
            ok
        }
    }

    /// Generates next byte, checks and updates `self.ok` if the padding is ok
    fn advance_iterating(&mut self) -> bool {
        if self.ok.is_some() {
            let inc_idx = BLOCK_SIZE - self.padding as usize;
            self.counter[inc_idx] = self.counter[inc_idx].wrapping_add(1);
        }
        let ok = self.check();
        self.ok = Some(ok);
        ok
    }

    fn reset_block_state(&mut self) {
        self.padding = 0x01;
        self.counter.fill(0);
        self.ok = None;
    }

    /// After hitting the right byte, derives the next plainbyte
    fn calculate_next_plainbyte(&mut self) {
        let inc_idx = BLOCK_SIZE - self.padding as usize;
        let plainbyte = self.counter[inc_idx] ^ self.padding ^ self.iv[inc_idx];
        self.known.insert(0, plainbyte);
    }

    // Relods counter for searching for the next byte
    fn reload_counter(&mut self) {
        self.padding += 1;
        self.counter.fill(0);
        // Before searching for the next padding, we should reset our counter
        // so that after decryption it already contains the part of valid pading.
        // For example: we know 4 bytes, and try to find the fifth from the end.
        // For that, we want to search for a padding, that looks like:
        //
        //    0x05 0x05 0x05 0x05 0x05
        //
        // To do so, we should set the last four bytes of plaintext to
        //
        //      XX 0x05 0x05 0x05 0x05
        //
        // In CBC, after decryption, and before xoring, the oracle gets
        // `iv + plaintext`, so to change it to our padding, we should cancel
        // out `iv` and known `plaintext`, and set desired padding.
        // To do this, we just xor the last bytes of our counter with `known`
        // bytes, `iv` and `padding`.
        for (((dst, known), padding), iv) in self
            .counter
            .iter_mut()
            .rev()
            .zip(self.known.iter().rev())
            .zip(std::iter::repeat(self.padding))
            .zip(self.iv.iter().rev())
        {
            *dst = *known ^ padding ^ iv;
        }
        self.ok = None;
    }

    /// Reloads the next block for decrypting
    fn reload_block(&mut self) -> State {
        let last_block = self.ciphertext.len() / BLOCK_SIZE - 1;
        match self.block_num {
            Some(num) if num == last_block => {
                self.decrypted.append(&mut self.known);
                State::Finished
            }
            Some(num) => {
                let block = self
                    .ciphertext
                    .chunks(BLOCK_SIZE)
                    .nth(num + 1)
                    .expect("should be checked");
                // in cbc the next plainblock is xored with a previous cipherblock
                // so, our next iv is the currect cipherblock
                self.iv = self.block.clone();
                self.decrypted.append(&mut self.known);
                self.block = block.into();
                self.block_num = Some(num + 1);
                State::ReadyToIterate
            }
            // First block is special, because it uses the oracle's iv
            None => {
                let iv = self.oracle.iv();
                let block = self
                    .ciphertext
                    .chunks(BLOCK_SIZE)
                    .next()
                    .expect("there is always at least one block");
                self.block = block.into();
                self.iv = iv.into();
                self.block_num = Some(0);
                State::ReadyToIterate
            }
        }
    }

    /// Advances the state machine. Most of the time, it just steps and makes
    /// a query to the oracle.
    pub fn advance(&mut self) {
        self.state = match self.state {
            State::Start => State::ReloadingBlock,
            State::ReadyToIterate => State::IteratingByte,
            State::IteratingByte => {
                let ok = self.advance_iterating();
                if ok {
                    State::FoundByte
                } else {
                    State::IteratingByte
                }
            }
            State::FoundByte => State::CalculatingPlainByte,
            State::CalculatingPlainByte => {
                self.calculate_next_plainbyte();
                State::ReloadingCounter
            }
            State::ReloadingCounter => {
                self.reload_counter();
                if usize::from(self.padding) == BLOCK_SIZE + 1 {
                    State::ReloadingBlock
                } else {
                    State::ReadyToIterate
                }
            }
            State::ReloadingBlock => {
                self.reset_block_state();
                self.reload_block()
            }
            State::Finished => State::Finished,
        };
    }
}
