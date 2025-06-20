# Use the official Rust image
FROM rust:1.82

# Set the working directory
WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Expose port 3000
EXPOSE 3000

# Set default environment variable
ENV UPSTREAM_BASE_URL=https://gustavskitchen.se

# Run the binary
CMD ["./target/release/pixel-proxy"]