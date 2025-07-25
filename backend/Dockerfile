# Use the official Rust nightly image as a base
FROM rustlang/rust:nightly-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/app

# Copy the manifests
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove the dummy main.rs and copy the real source code
RUN rm src/main.rs
COPY src ./src

# Build the application
RUN cargo build --release

# Install sqlx-cli in the builder stage
RUN cargo install sqlx-cli --no-default-features --features postgres

# Create a new stage with a minimal runtime image
FROM rustlang/rust:nightly-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false app

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/

# Copy sqlx-cli from builder stage
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/

# Copy the startup script
COPY start.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/start.sh

# Change ownership
RUN chown app:app /usr/local/bin/backend /usr/local/bin/sqlx /usr/local/bin/start.sh

# Switch to the app user
USER app

# Expose the port the app runs on
EXPOSE 3000

# Run the startup script
CMD ["/usr/local/bin/start.sh"] 