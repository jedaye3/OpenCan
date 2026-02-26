use anyhow::{Context, Result};
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::model::Message;

#[derive(Debug, Serialize)]
struct SessionLine<'a> {
    ts_unix: u64,
    role: &'a str,
    content: &'a str,
}

pub struct SessionLogger {
    writer: BufWriter<File>,
}

impl SessionLogger {
    pub fn new() -> Result<Self> {
        let sessions_dir = config::sessions_dir()?;
        std::fs::create_dir_all(&sessions_dir).with_context(|| {
            format!(
                "Failed to create sessions directory at {}",
                sessions_dir.display()
            )
        })?;

        let now = timestamp();
        let path = sessions_dir.join(format!("session-{}.jsonl", now));
        let file = File::create(&path)
            .with_context(|| format!("Failed to create session file at {}", path.display()))?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn append(&mut self, message: &Message) -> Result<()> {
        let line = SessionLine {
            ts_unix: timestamp(),
            role: &message.role,
            content: &message.content,
        };

        let json = serde_json::to_string(&line).context("Failed to serialize session line")?;
        writeln!(self.writer, "{}", json).context("Failed to append to session log")?;
        self.writer.flush().context("Failed to flush session log")?;
        Ok(())
    }
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
