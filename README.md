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

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `BACKEND_HOST` | No | Bind host (default: `0.0.0.0`) |
| `BACKEND_PORT` | No | HTTP port (default: `8080`) |
| `GRPC_AI_URL` | No | AI service gRPC endpoint |
| `JWT_SECRET` | Yes | Secret for JWT token signing |
| `WORLD_ID_APP_ID` | No | World ID application ID |
| `WORLD_ID_ACTION_ID` | No | World ID action identifier |
| `WORLD_ID_API_URL` | No | World ID API base URL |
| `RUST_LOG` | No | Log level (default: `info`) |

## Project Structure

```
src/
├── main.rs              # Entry point, server startup
├── config.rs            # Environment configuration
├── api/                 # HTTP handlers, routes, middleware
├── application/         # Use cases and business workflows
├── domain/              # Core entities and business rules
├── infrastructure/      # External service adapters (AI, blockchain)
├── crypto/              # PQC: commitments, Merkle tree
├── grpc/                # gRPC client for pliq-ai
├── websocket/           # Real-time event broadcasting
└── services/            # Shared orchestration logic
proto/
└── ai_service.proto     # gRPC service definitions
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

## Note on Cross-Crate Dependency

This service depends on `pliq-back-db` as a git dependency. For local development, `.cargo/config.toml` patches it to use the sibling directory. In CI/CD, cargo fetches from the git remote.
