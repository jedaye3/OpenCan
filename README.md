# OpenCan

[![CI](https://github.com/jedaye3/OpenCan/actions/workflows/ci.yml/badge.svg)](https://github.com/jedaye3/OpenCan/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

OpenCan is a lightweight Rust OpenClaw alternative for local-first terminal workflows. It combines an interactive assistant loop with durable memory, session logs, and predictable configuration.

## Highlights

- Interactive assistant session (`opencan agent`)
- Guided onboarding and environment checks (`opencan onboard`, `opencan doctor`)
- Long-term project memory in `~/.opencan/MEMORY.md`
- Structured session logging in `~/.opencan/sessions/*.jsonl`
- OpenAI-compatible backend configuration via TOML
- CI pipeline for format, lint, and tests

## Quick Start (macOS)

### 1) Install Rust

```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```

### 2) Build OpenCan

```bash
git clone https://github.com/jedaye3/OpenCan.git
cd OpenCan
cargo build
```

### 3) Initialize config

```bash
cargo run -- onboard
```

### 4) Set API key

```bash
export OPEN_API_KEY="<api_key>"
```

### 5) Validate setup and start

```bash
cargo run -- doctor
cargo run -- agent
```

## Command Reference

- `opencan onboard [--force] [--model <model>] [--base-url <url>] [--api-key-env <name>]`
- `opencan doctor`
- `opencan agent`

In-session commands:

- `/help`
- `/new`
- `/remember <note>`
- `/exit`

## Configuration

Default config file: `~/.opencan/config.toml`

Example:

```toml
model = "gpt-5-mini"
base_url = "https://api.openai.com/v1"
api_key_env = "OPEN_API_KEY"
temperature = 1.0
system_prompt = "You are OpenCan, a pragmatic coding assistant. Prefer concise, correct answers and ask for clarification only when blocked."
memory_file = "~/.opencan/MEMORY.md"
```

## Install as a Global CLI

```bash
cargo install --git https://github.com/jedaye3/OpenCan.git opencan
```

Then run:

```bash
opencan onboard
opencan agent
```

## Make API Key Persistent (zsh)

```bash
echo 'export OPEN_API_KEY="<api_key>"' >> ~/.zshrc
source ~/.zshrc
```

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

CI runs on push to `main` and on pull requests.

## Project Structure

- `src/main.rs`: CLI entrypoint and command routing
- `src/agent.rs`: interactive loop and slash commands
- `src/client.rs`: Chat Completions HTTP client
- `src/config.rs`: config lifecycle and memory file management
- `src/session.rs`: JSONL session logging
- `src/model.rs`: chat message model

## Release Workflow

```bash
git checkout -b feature/<name>
git add .
git commit -m "<clear summary>"
git push -u origin feature/<name>
gh pr create --fill --base main --head feature/<name>
```

## License

MIT. See [LICENSE](./LICENSE).
