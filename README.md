# Pixel Proxy

A fast and lightweight image proxy service built with Rust. Stream images from remote servers through your own endpoint.

## What it does

Pixel Proxy allows you to serve images from external sources through your own domain. When a client requests `/images/photo.jpg`, the service fetches the image from a configured upstream server and streams it directly to the client.

## Features

- **Memory efficient**: Streams images without loading them entirely into memory
- **Fast**: Built with Rust and Axum for high performance
- **Simple**: Minimal configuration and setup
- **Scalable**: Handles multiple concurrent requests efficiently

## Quick Start

1. **Clone and build**

   ```bash
   git clone <your-repo>
   cd pixel-proxy
   cargo run
   ```

2. **Test it out**
   ```bash
   curl http://localhost:3000/images/grill.png
   ```
   This will fetch and return the image from `https://gustavskitchen.se/images/grill.png`

## Configuration

Set the upstream server via environment variable:

```bash
UPSTREAM_BASE_URL=https://your-image-server.com cargo run
```

Default upstream: `https://gustavskitchen.se`

## Usage

```
GET /images/{filename}
```

Examples:

- `/images/photo.jpg` → `{UPSTREAM_BASE_URL}/images/photo.jpg`
- `/images/icons/logo.png` → `{UPSTREAM_BASE_URL}/images/icons/logo.png`

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Build for production
cargo build --release
```

## License

MIT
