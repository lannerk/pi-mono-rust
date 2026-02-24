# pi-mono-rust

AI Agent Toolkit - Rust Implementation

A comprehensive rewrite of the [pi-mono](https://github.com/badlogic/pi-mono) project in Rust, providing a modular AI agent toolkit with support for multiple LLM providers, terminal UI, web UI, and various integrations.

## Features

- **Unified LLM API** - Support for OpenAI, Anthropic, Google, and other LLM providers
- **Modular Architecture** - Core agent runtime with extensible tool system
- **Terminal UI** - Interactive chat interface with code highlighting
- **Web UI** - Browser-based interface with WebSocket support
- **Slack Integration** - Chat with agents through Slack
- **Coding Agent** - AI-powered coding assistant with file operations
- **Tool System** - Extensible tool registry for custom functionality

## Project Structure

```
pi-mono-rust/
├── crates/
│   ├── pi-ai/            # Unified LLM API client
│   ├── pi-agent-core/    # Core agent runtime
│   ├── pi-tui/           # Terminal UI library
│   ├── pi-coding-agent/  # Coding agent CLI
│   ├── pi-web-ui/        # Web UI server
│   ├── pi-mom/           # Slack bot integration
│   └── pi-pods/           # Tool and integration pods
├── Cargo.toml            # Workspace configuration
└── README.md             # This file
```

## Getting Started

### Prerequisites

- Rust 1.70+ with Cargo
- Tokio runtime (included via dependencies)
- LLM API keys (for OpenAI, Anthropic, etc.)

### Installation

```bash
# Clone the repository
git clone https://github.com/lannerk/pi-mono-rust.git
cd pi-mono-rust

# Build the project
cargo build

# Run the coding agent
cargo run --bin pi chat
```

### Configuration

Create a `.env` file or use environment variables to configure LLM providers:

```bash
# OpenAI
OPENAI_API_KEY=your-api-key

# Anthropic
ANTHROPIC_API_KEY=your-api-key

# Google
GOOGLE_API_KEY=your-api-key
```

## Usage

### Coding Agent

```bash
# Start chat mode
cargo run --bin pi chat

# Chat with UI
cargo run --bin pi chat --ui

# Stream responses
cargo run --bin pi chat --stream
```

### Web UI

```bash
# Start web server
cargo run --bin pi-web-ui

# Access at http://localhost:3000
```

### Slack Bot

```bash
# Start Slack bot
cargo run --bin pi-mom
```

## Documentation

- [API Reference](https://docs.rs/pi-mono-rust)
- [Architecture Guide](docs/architecture.md)
- [Tool Development](docs/tool-development.md)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a pull request

## License

MIT

## Acknowledgments

Based on the original [pi-mono](https://github.com/badlogic/pi-mono) project by badlogic.
