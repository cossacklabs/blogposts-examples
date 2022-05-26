use std::marker::PhantomData;

use rustyline::error::ReadlineError;

pub struct Shell<P: clap::Parser> {
    prompt: String,
    _parser: PhantomData<P>,
}

impl<P: clap::Parser> Shell<P> {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            _parser: PhantomData {},
        }
    }

    pub fn start_loop(
        &self,
        mut callback: impl FnMut(P) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut rl = rustyline::Editor::<()>::new();

        let mut process = |line: String| -> anyhow::Result<()> {
            let mut args = shell_words::split(&line)?;

            // Dirty hack
            // Insert empty string which represents the name of the executable
            args.insert(0, " ".into());
            let parsed = P::try_parse_from(args)?;

            callback(parsed)
        };

        loop {
            let line = match rl.readline(&self.prompt) {
                Ok(line) => line,
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    return Ok(());
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };

            if line.trim().is_empty() {
                continue;
            }

            rl.add_history_entry(line.as_str());

            if let Err(err) = process(line) {
                eprintln!("{}", format_error(err.to_string()));
            }
        }
    }
}

fn format_error(err: String) -> String {
    // This is clap's usage, not an error
    if err.starts_with('-') {
        return err
            .trim_start_matches(|c| matches!(c, '\n' | '-' | ' '))
            .to_owned();
    }

    let ascii = err.to_ascii_lowercase();
    let prefix = if ascii.starts_with("error") {
        ""
    } else {
        "error: "
    };

    format!("{prefix}{err}")
}
