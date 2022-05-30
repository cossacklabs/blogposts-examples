pub mod uglifier;

fn main() -> anyhow::Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
        println!("Input key (16/24/32 bytes):");
        std::io::stdin().read_line(&mut line)?;
        // remove new line char if present and leading and tracing spaces
        line = line.trim().to_string();
        match line.to_lowercase().as_str() {
            "q" | "quit" | "exit" => break,
            _ => {
                let encryption_key = uglifier::param_set_encryption_key(&line);
                println!("{} - uglified key {}\n", encryption_key.0, encryption_key.1);
            }
        }
    }

    Ok(())
}
