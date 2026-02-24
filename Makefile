.PHONY: all build clean test fmt clippy doc install help

all: build

build:
	cargo build --release

clean:
	cargo clean

test:
	cargo test --workspace

test-coverage:
	cargo tarpaulin --workspace --out Html

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace -- -D warnings

doc:
	cargo doc --workspace --no-deps --open

install:
	cargo install --path crates/pi-coding-agent --force
	cargo install --path crates/pi-pods --force
	cargo install --path crates/pi-mom --force

run-coding-agent:
	cargo run --release -p pi-coding-agent -- chat --ui

run-pods:
	cargo run --release -p pi-pods -- list

run-mom:
	cargo run --release -p pi-mom

check: fmt-check clippy test

help:
	@echo "Available targets:"
	@echo "  all              - Build all crates"
	@echo "  build            - Build all crates in release mode"
	@echo "  clean            - Clean build artifacts"
	@echo "  test             - Run all tests"
	@echo "  test-coverage    - Run tests with coverage report"
	@echo "  fmt              - Format all code"
	@echo "  fmt-check        - Check code formatting"
	@echo "  clippy           - Run clippy linter"
	@echo "  doc              - Build and open documentation"
	@echo "  install          - Install all binaries"
	@echo "  run-coding-agent - Run coding agent CLI"
	@echo "  run-pods         - Run pods manager"
	@echo "  run-mom          - Run Slack bot"
	@echo "  check            - Run fmt-check, clippy, and test"
	@echo "  help             - Show this help message"
