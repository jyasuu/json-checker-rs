FROM rust:1.85.1 AS builder

# Set the working directory
WORKDIR /usr/src/app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY ./src ./src

# Build the project in release mode
RUN cargo build --release

FROM alpine


# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/release/json-checker-rs /usr/local/bin/json-checker-rs


# Set the startup command
CMD ["json-checker-rs"]