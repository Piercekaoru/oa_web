# ── Stage 1: Build ──
FROM rust:1.88 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Create a dummy main.rs for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Copy real source code and rebuild
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/oa-backend /usr/local/bin/oa-backend

ENV RUST_LOG=info

EXPOSE 8080
CMD ["oa-backend"]
