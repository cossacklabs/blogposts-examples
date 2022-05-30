use colored::Colorize;
use hex::ToHex;

fn param_set_default_encryption_key(key_length: u8) -> Vec<u8>
{
    let default_key = vec![0x62u8; key_length as usize];
    return default_key
}

fn read_hex_nibble(c: char) -> (bool, String)
{
    if (c >='0') && (c <= '9'){
        return (true, c.to_string());
    }
    else if (c >='A') && (c <= 'F')
    {
        return (true, c.to_string());
    }
    else if (c >='a') && (c <= 'f')
    {
        return (true, c.to_string());
    }
    else
    {
        // printf("[%u] read_hex_nibble: Error char not in supported range",nodeId);
        return (false, '0'.to_string().on_color("green").to_string());
    }
}

fn convert_to_hex(key_input: &String, key_length: u8) -> (bool, String) {
    let mut key_output = String::new();
    let mut bad_chars: bool = false;

    for c in key_input.chars().enumerate(){
        if c.0 > ((key_length as usize) * 2) - 1 {
            break;
        }
        let num = read_hex_nibble(c.1);
        key_output.push_str(&num.1);
        bad_chars |= !num.0;
    }

    return (bad_chars, key_output);
}

pub(crate) fn param_set_encryption_key(key_input: &String) -> (String, String)
{
    // Use the new encryption level to help with key changes before reboot
    // Deduce key length (bytes) from level 1 -> 16, 2 -> 24, 3 -> 32
    let len: u8 = key_input.len() as u8;
    let key_length: u8;

    if len >= 32 * 2 {
        key_length = 32;
    }
    else if len >= 24 * 2{
        key_length = 24;
    }
    else {
        key_length = 16;
    }

    // let's not initialize it here, because compiler pulling warnings out
    // of his hand:
    // warning: value assigned to `output_key` is never read
    let mut output_key: String;
    let mut output_str_desc: String = String::new();

    // If not enough characters (2 char per byte), then set default
    if len < 2 * key_length {
        output_key = param_set_default_encryption_key(key_length)
            .encode_hex::<String>()
            .color("white")
            .on_color("red")
            .bold()
            .to_string();
        //println!("{}\n",output_key);
        output_str_desc += &*"- using default key\n".to_string().bold().to_string();
        output_str_desc += &*" ERROR ".to_string().color("white").on_color("red").bold().to_string();
        output_str_desc += &*format!(" - Key length: {}, Required: {}", len, 2 * key_length);
        //output_str_desc = format!("- using default key, because:\n ERROR - Key length:{}, Required {}\n".red(), len, 2 * key_length);
        return (output_key, output_str_desc);
    }
    else {
        // We have sufficient characters for the encryption key.
        // If too many characters, then it will just ignore extra ones
        //println!("key len {}\n",key_length);
        let bad_chars: bool;
        (bad_chars, output_key) = convert_to_hex(key_input, key_length);
        if bad_chars {
            output_str_desc += &*"\n".to_string();
            output_str_desc += &*" WARNING ".to_string().color("black").on_color("bright yellow").bold().to_string();
            output_str_desc += &*" Wrong chars appeared!";
        }
        if len > (key_length * 2) {
            let trimmed_elements_count = len - (key_length * 2);
            let trimmed_bytes: String = " ".repeat(trimmed_elements_count as usize).on_color("red").to_string();
            output_key += &*trimmed_bytes;

            output_str_desc += &*"\n".to_string();
            output_str_desc += &*" WARNING ".to_string().color("black").on_color("bright yellow").bold().to_string();
            output_str_desc += &*format!(" Too long input key. It was trimmed! {} chars were trimmed!", trimmed_elements_count);
        }
    }

    return (output_key, output_str_desc);
}
