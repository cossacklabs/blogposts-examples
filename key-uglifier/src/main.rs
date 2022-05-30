pub mod uglifier;

fn main() {
    let mut line: String = String::new();
    loop {
        line.clear();
        println!("Input key: \t");
        std::io::stdin().read_line(&mut line).unwrap();
        // remove new line char if present and leading and tracing spaces
        line = line.trim().to_string();
        match line.to_lowercase().as_str() {
            "q" => { break },
            _ => {
                let encryption_key = uglifier::param_set_encryption_key(&line);
                println!("{} - uglified key {}\n", encryption_key.0, encryption_key.1);
            }
        }
    }
}
