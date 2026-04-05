# Stage 1: Builder
FROM rust:1.94-slim AS builder
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Local dev: docker-compose provides pliq-back-db via additional_contexts
# CI/CD: cargo fetches via git dependency
COPY --from=pliq-back-db . /pliq-back-db/

# Copy cargo config (contains [patch] for local path override)
COPY .cargo .cargo

# Cache dependency layer
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy source and build
COPY src/ src/
COPY proto/ proto/
COPY build.rs ./
RUN cargo build --release

# Stage 2: Runtime
FROM debian:trixie-slim AS runner
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 1001 pliq && \
    useradd --system --uid 1001 --gid pliq pliq

COPY --from=builder /app/target/release/pliq-back ./pliq-back

USER pliq
EXPOSE 3001 50051

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3001/api/v1/health || exit 1

CMD ["./pliq-back"]
