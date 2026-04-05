# pliq-back

Rust backend API for the Pliq rental platform. Provides REST endpoints, WebSocket real-time notifications, and gRPC client for the AI service.

## Prerequisites

- [Rust](https://rustup.rs/) >= 1.94.1 (edition 2024)
- [Docker](https://www.docker.com/) (for containerized builds)
- [protoc](https://grpc.io/docs/protoc-installation/) (Protocol Buffers compiler)
- PostgreSQL 17 (via docker-compose or local install)

## Setup

```bash
# Clone and enter the repo
cd pliq-back

# Copy environment config
cp .env.example .env
# Edit .env with your values

# Build
cargo build

# Run
cargo run
```

The HTTP server starts on port 3001 by default.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `HTTP_HOST` | No | `0.0.0.0` | HTTP server bind address |
| `HTTP_PORT` | No | `3001` | HTTP server port |
| `GRPC_PORT` | No | `50051` | gRPC server port |
| `GRPC_AI_HOST` | Yes | — | pliq-ai gRPC address |
| `JWT_SECRET` | Yes | — | Secret for JWT signing |
| `JWT_EXPIRY_HOURS` | No | `24` | JWT token expiry |
| `WORLD_ID_APP_ID` | Yes | — | World ID application ID |
| `WORLD_ID_ACTION` | No | `pliq-verify` | World ID action string |
| `WORLD_ID_API_URL` | No | `https://developer.worldcoin.org/api/v2/verify` | World ID API endpoint |
| `CORS_ORIGINS` | No | `http://localhost:3000` | Allowed CORS origins |
| `RATE_LIMIT_RPS` | No | `100` | Rate limit per IP |
| `LOG_LEVEL` | No | `info` | Tracing log level |
| `LOG_FORMAT` | No | `pretty` | Log format (pretty/json) |
| `UNLINK_API_KEY` | No | — | Unlink SDK key (optional) |
| `UNLINK_ENGINE_URL` | No | `https://staging-api.unlink.xyz` | Unlink API endpoint |
| `CIRCLE_API_KEY` | No | — | Circle API key (optional) |
| `WS_HEARTBEAT_INTERVAL_SECS` | No | `30` | WebSocket ping interval |
| `WS_MAX_MESSAGE_SIZE` | No | `65536` | Max WebSocket message bytes |
| `STUN_SERVER` | No | `stun:stun.l.google.com:19302` | STUN server URL |
| `PLATFORM_FEE_BPS` | No | `100` | Platform fee (basis points) |

## Project Structure

```
src/
├── main.rs              # Entry point, server startup
├── config.rs            # Environment configuration (21 vars)
├── api/
│   ├── routes.rs        # Route definitions (40+ endpoints)
│   ├── errors.rs        # ApiError enum, IntoResponse
│   ├── handlers/        # HTTP request handlers
│   │   ├── auth.rs      # World ID verification
│   │   ├── users.rs     # User profile CRUD
│   │   ├── listings.rs  # Listing CRUD + fraud check
│   │   ├── applications.rs  # Application workflow
│   │   ├── leases.rs    # Lease lifecycle
│   │   ├── payments.rs  # Payment processing
│   │   ├── escrow.rs    # Escrow commitments
│   │   ├── reputation.rs  # Proof of Rent
│   │   └── webrtc.rs    # STUN/TURN config
│   ├── middleware/      # Request pipeline
│   │   ├── auth.rs      # JWT validation
│   │   ├── cors.rs      # CORS configuration
│   │   ├── request_id.rs  # X-Request-Id
│   │   ├── rate_limit.rs  # Per-IP rate limiting
│   │   └── tracing_mw.rs  # Request logging
│   └── extractors/      # Axum extractors
├── services/            # Business logic orchestration
│   ├── identity.rs      # World ID verification
│   ├── listings.rs      # Listing management
│   ├── applications.rs  # Application workflow
│   ├── leases.rs        # Lease lifecycle
│   ├── payments.rs      # Payment processing
│   ├── reputation.rs    # PoR scoring + Merkle tree
│   ├── escrow.rs        # Hash-based escrow
│   └── privacy.rs       # Unlink integration
├── crypto/              # Cryptographic modules
│   ├── merkle.rs        # SHA-3 Merkle tree
│   ├── commitments.rs   # Hash-based escrow commitments
│   └── hybrid.rs        # X25519 + AES-256-GCM
├── domain/              # Entity re-exports from pliq-back-db
├── grpc/                # gRPC server + AI client stubs
├── websocket/           # Real-time messaging
│   ├── handlers.rs      # WebSocket upgrade + dispatch
│   ├── messages.rs      # Message envelope types
│   ├── connection.rs    # Connection registry
│   ├── manager.rs       # Broadcast event manager
│   └── events.rs        # Event type definitions
├── application/         # Use case orchestration
└── infrastructure/      # External service adapters
proto/
├── ai_service.proto     # Fraud detection + matching
├── fraud_detection.proto  # User behavior analysis
├── matching.proto       # Tenant-listing compatibility
└── common.proto         # Shared message types
```

## Development

```bash
# Run tests
cargo test

# Run with hot reload (requires cargo-watch)
cargo watch -x run

# Check without building
cargo check

# Lint
cargo clippy

# Format
cargo fmt
```

## Docker

```bash
# Build (requires pliq-back-db as additional context)
docker build .

# Via docker-compose (from root project)
docker compose up back
```

The container exposes port 3001 for HTTP and 50051 for gRPC.

## Note on Cross-Crate Dependency

This service depends on `pliq-back-db` as a git dependency. For local development, `.cargo/config.toml` patches it to use the sibling directory. In CI/CD, cargo fetches from the git remote.
