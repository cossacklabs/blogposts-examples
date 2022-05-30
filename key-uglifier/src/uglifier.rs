use colored::Colorize;
use hex::ToHex;
use std::fmt::Write;

fn param_set_default_encryption_key(key_length: u8) -> Vec<u8> {
    vec![0x62u8; key_length as usize]
}

fn read_hex_nibble(c: char) -> (bool, String) {
    if "0123456789ABCDEFabcdef".contains(c) {
        (true, c.to_string())
    } else {
        // printf("[%u] read_hex_nibble: Error char not in supported range",nodeId);
        (false, '0'.to_string().on_color("green").to_string())
    }
}

fn convert_to_hex(key_input: &str, key_length: u8) -> (bool, String) {
    let mut key_output = String::new();
    let mut bad_chars = false;

    for (index, character) in key_input.chars().enumerate() {
        if index > ((key_length as usize) * 2) - 1 {
            break;
        }
        let (ok, num) = read_hex_nibble(character);
        key_output.push_str(&num);
        bad_chars |= !ok;
    }

    (bad_chars, key_output)
}

pub(crate) fn param_set_encryption_key(key_input: &str) -> (String, String) {
    // Use the new encryption level to help with key changes before reboot
    // Deduce key length (bytes) from level 1 -> 16, 2 -> 24, 3 -> 32
    let len = key_input.len() as u8;
    let key_length;

    if len >= 32 * 2 {
        key_length = 32;
    } else if len >= 24 * 2 {
        key_length = 24;
    } else {
        key_length = 16;
    }

    // let's not initialize it here, because compiler pulling warnings out
    // of his hand:
    // warning: value assigned to `output_key` is never read
    let mut output_key: String;
    let mut output_str_desc = String::new();

    // If not enough characters (2 char per byte), then set default
    if len < 2 * key_length {
        output_key = param_set_default_encryption_key(key_length)
            .encode_hex::<String>()
            .color("white")
            .on_color("red")
            .bold()
            .to_string();
        //println!("{}\n",output_key);

        write!(
            output_str_desc,
            "{}{} - Key length (in hex chars): {}, Required (in hex chars): {}",
            "- using default key\n".bold(),
            " ERROR ".color("white").on_color("red").bold(),
            len,
            2 * key_length
        )
        .expect("Write to a string cannot fail");

        //output_str_desc = format!("- using default key, because:\n ERROR - Key length:{}, Required {}\n".red(), len, 2 * key_length);
        return (output_key, output_str_desc);
    } else {
        // We have sufficient characters for the encryption key.
        // If too many characters, then it will just ignore extra ones
        //println!("key len {}\n",key_length);
        let bad_chars: bool;
        (bad_chars, output_key) = convert_to_hex(key_input, key_length);
        if bad_chars {
            write!(
                output_str_desc,
                "\n{} Wrong chars appeared!",
                " WARNING ".color("black").on_color("bright yellow").bold()
            )
            .expect("Write to a string cannot fail");
        }
        if len > (key_length * 2) {
            let trimmed_elements_count = len - (key_length * 2);
            let trimmed_bytes = " ".repeat(trimmed_elements_count as usize).on_color("red");
            write!(output_key, "{}", trimmed_bytes).expect("Write to a string cannot fail");

            write!(
                output_str_desc,
                "\n{} Too long input key. It was trimmed! {} hex chars were trimmed!",
                " WARNING ".color("black").on_color("bright yellow").bold(),
                trimmed_elements_count
            )
            .expect("Write to a string cannot fail");
        }
    }

    (output_key, output_str_desc)
}
