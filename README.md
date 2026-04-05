# pliq-back

Rust backend API for the Pliq rental platform. Provides HTTP REST endpoints via Axum, WebSocket real-time notifications, gRPC client for the AI fraud-detection service, on-chain event indexing via Alloy, and cryptographic modules for Merkle-based Proof of Rent.

## Prerequisites

- [Rust](https://rustup.rs/) >= 1.94.1 (edition 2024)
- [protoc](https://grpc.io/docs/protoc-installation/) (Protocol Buffers compiler)
- PostgreSQL 17 (via docker-compose or local install)
- [Docker](https://www.docker.com/) (for containerized builds)

## Setup

```bash
cd pliq-back

# Copy environment config and fill in required values
cp .env.example .env

# Build
cargo build

# Run
cargo run
```

The HTTP server starts on `0.0.0.0:3001` by default and the gRPC server on port `50051`.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | -- | PostgreSQL connection string |
| `GRPC_AI_HOST` | Yes | -- | pliq-ai gRPC address (e.g. `http://localhost:50052`) |
| `JWT_SECRET` | Yes | -- | Secret key for JWT signing |
| `WORLD_ID_APP_ID` | Yes | -- | World ID application ID |
| `HTTP_HOST` | No | `0.0.0.0` | HTTP server bind address |
| `HTTP_PORT` | No | `3001` | HTTP server port |
| `GRPC_PORT` | No | `50051` | gRPC server port |
| `JWT_EXPIRY_HOURS` | No | `24` | JWT token lifetime in hours |
| `WORLD_ID_ACTION` | No | `pliq-verify` | World ID action string |
| `WORLD_ID_API_URL` | No | `https://developer.worldcoin.org/api/v2/verify` | World ID verification endpoint |
| `CORS_ORIGINS` | No | `http://localhost:3000` | Comma-separated allowed CORS origins |
| `RATE_LIMIT_RPS` | No | `100` | Max requests per second per IP |
| `LOG_LEVEL` | No | `info` | Tracing filter level |
| `LOG_FORMAT` | No | `pretty` | Log output format (`pretty` or `json`) |
| `UNLINK_API_KEY` | No | -- | Unlink SDK API key (optional integration) |
| `UNLINK_ENGINE_URL` | No | `https://staging-api.unlink.xyz` | Unlink API endpoint |
| `CIRCLE_API_KEY` | No | -- | Circle API key (optional integration) |
| `WS_HEARTBEAT_INTERVAL_SECS` | No | `30` | WebSocket ping interval in seconds |
| `WS_MAX_MESSAGE_SIZE` | No | `65536` | Maximum WebSocket message size in bytes |
| `STUN_SERVER` | No | `stun:stun.l.google.com:19302` | STUN server URL for WebRTC |
| `PLATFORM_FEE_BPS` | No | `100` | Platform fee in basis points (100 = 1%) |

## Project Structure

```
src/
  main.rs              Entry point, server startup, graceful shutdown
  config.rs            Environment configuration (Config::from_env)
  api/
    routes.rs          Route definitions
    errors.rs          ApiError enum, IntoResponse impl
    response.rs        Response envelope helpers
    handlers/          HTTP request handlers (auth, users, listings, ...)
    middleware/         Request pipeline (JWT, CORS, rate limiting, tracing)
    extractors/        Axum extractors
  services/            Business logic orchestration
  chain/               On-chain integration (Alloy event indexer)
  crypto/              Cryptographic modules (Merkle tree, escrow commitments, X25519+AES-GCM)
  domain/              Entity re-exports from pliq-back-db
  grpc/                gRPC server and AI client stubs
  websocket/           Real-time messaging (connection registry, broadcast manager)
  application/         Use case orchestration
  infrastructure/      External service adapters
proto/
  ai_service.proto     Fraud detection + matching service definitions
  fraud_detection.proto
  matching.proto
  common.proto         Shared message types
```

## Development

```bash
# Run tests
cargo test

# Run with hot reload (requires cargo-watch)
cargo watch -x run

# Type check without building
cargo check

# Lint
cargo clippy

# Format
cargo fmt
```

## Docker

The Dockerfile is a multi-stage build (builder + slim runtime). It requires `pliq-back-db` as an additional build context because the crate depends on it at compile time.

```bash
# Standalone build (needs pliq-back-db sibling directory)
docker build \
  --build-context pliq-back-db=../pliq-back-db \
  .

# Via docker-compose (from root project, contexts configured automatically)
docker compose up back
```

The container exposes port `3001` (HTTP) and `50051` (gRPC) and runs as a non-root `pliq` user.

## Note on Cross-Crate Dependency

This service depends on `pliq-back-db` for database models, queries, connection pooling, and migrations. The dependency is declared as a git URL in `Cargo.toml`:

```toml
pliq-back-db = { git = "ssh://git@github.com/pliq-tech/pliq-back-db", branch = "main" }
```

For local development, `.cargo/config.toml` patches the git URL to resolve from the sibling directory (`../pliq-back-db`), so changes to the database crate are picked up immediately without pushing to the remote. In CI/CD (or when the sibling directory is absent), cargo fetches from the git remote instead.
