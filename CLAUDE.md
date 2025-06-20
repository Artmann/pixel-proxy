# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

```bash
# Run the development server
cargo run

# Run tests
cargo test

# Build for production
cargo build --release

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture

This is a Rust-based image proxy service built with:
- **Tokio**: Async runtime for handling concurrent requests
- **Tower/Tower-HTTP**: Middleware and HTTP utilities
- **Tokio-util**: Additional utilities for streaming

The service acts as a reverse proxy for images, fetching them from an upstream server and streaming them directly to clients without loading entire images into memory.

## Configuration

- **UPSTREAM_BASE_URL**: Environment variable to set the upstream image server (defaults to `https://gustavskitchen.se`)
- The service listens on port 3000 by default
- API endpoint: `GET /images/{filename}` proxies to `{UPSTREAM_BASE_URL}/images/{filename}`

## Project Structure

- `src/main.rs`: Entry point (currently minimal)
- `src/handlers/`: HTTP request handlers (empty modules ready for implementation)
- `src/services/`: Business logic services (empty modules ready for implementation)

## Development Tips

- Use `cargo build` to verify that your changes work

## Project Conventions

- This project uses semver and conventional commits