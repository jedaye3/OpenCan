use anyhow::{Context, Result};
use std::io::{self, Write};

use crate::client;
use crate::config::{self, Config};
use crate::model::Message;
use crate::session::SessionLogger;

pub fn run_agent() -> Result<()> {
    let config = Config::load().context(
        "Failed loading config. Run `opencan onboard` first to create ~/.opencan/config.toml.",
    )?;
    config::ensure_layout(&config)?;

    let mut logger = SessionLogger::new()?;
    let mut messages = vec![Message::system(config.build_system_prompt()?)];

    println!("OpenCan session started.");
    println!("Commands: /help, /new, /remember <note>, /exit");

    let stdin = io::stdin();

    loop {
        print!("you> ");
        io::stdout().flush().context("Failed flushing prompt")?;

        let mut input = String::new();
        let bytes = stdin.read_line(&mut input).context("Failed reading stdin")?;
        if bytes == 0 {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "/exit" || input == "/quit" {
            break;
        }

        if input == "/help" {
            println!("/help                Show commands");
            println!("/new                 Clear short-term context");
            println!("/remember <note>     Save note to long-term memory");
            println!("/exit                End session");
            continue;
        }

        if input == "/new" {
            messages = vec![Message::system(config.build_system_prompt()?)];
            println!("Short-term context cleared.");
            continue;
        }

        if let Some(note) = input.strip_prefix("/remember ") {
            let trimmed = note.trim();
            if trimmed.is_empty() {
                println!("Usage: /remember <note>");
                continue;
            }
            config::append_memory_note(&config, trimmed)?;
            println!("Saved to memory.");
            continue;
        }

        let user_message = Message::user(input.to_string());
        logger.append(&user_message)?;
        messages.push(user_message);

        let assistant = client::chat_completion(&config, &messages)?;
        let assistant_message = Message::assistant(assistant.clone());
        logger.append(&assistant_message)?;
        messages.push(assistant_message);

        println!("opencan> {}", assistant.trim());
    }

    println!("Session ended.");
    Ok(())
}
