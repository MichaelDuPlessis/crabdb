# Use the official Rust image as the base
FROM rust:1.88 as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy all the workspace members
COPY concurrent-map/ ./concurrent-map/
COPY engine/ ./engine/
COPY logging/ ./logging/
COPY object/ ./object/
COPY server/ ./server/
COPY storage/ ./storage/
COPY threadpool/ ./threadpool/

# Build the project in release mode
RUN cargo build --release --bin engine

# Use a minimal runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false crabdb

# Create data directory and set permissions
RUN mkdir -p /app/data && chown crabdb:crabdb /app/data

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/engine /usr/local/bin/crabdb

# Set the user
USER crabdb

# Set the working directory
WORKDIR /app

# Expose the port
EXPOSE 7227

# Set the command to run the application
CMD ["crabdb"]
