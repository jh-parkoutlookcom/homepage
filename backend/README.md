# Homepage Backend

Rust-based backend service with REST (Axum) and gRPC (Tonic) APIs.

## Features

- 🚀 Axum REST API (HTTP/1.1)
- 🔌 gRPC API (HTTP/2)
- 🐳 Docker containerization
- ☸️ Kubernetes deployment
- 🔄 CI/CD with GitHub Actions

## Project Structure

```
homepage-backend/
├── .github/
│   └── workflows/
│       ├── deploy.yml           # Main deployment workflow
│       ├── ci.yml               # CI (test, lint, format check)
│       └── security.yml         # Security scanning (cargo audit)
│
├── .devcontainer/
│   ├── devcontainer.json        # Dev container configuration
│   └── Dockerfile               # Dev container Dockerfile
│
├── crates/
│   ├── grpc-api/
│   │   ├── Cargo.toml
│   │   ├── build.rs             # Protobuf compilation
│   │   ├── proto/
│   │   │   └── cvad.proto       # gRPC service definition
│   │   ├── src/
│   │   │   ├── lib.rs           # Library entry point
│   │   │   ├── auth.rs          # Token verification logic
│   │   │   ├── cvad_service.rs  # gRPC service implementation
│   │   │   ├── cvad_restapi.rs  # CVAD REST API client
│   │   │   └── generated/       # Auto-generated protobuf code
│   │   │       ├── cvad.v1.rs
│   │   │       └── descriptor.bin
│   │   ├── examples/
│   │   │   └── client_test.rs   # gRPC client example
│   │   └── tests/
│   │       └── integration_test.rs
│   │
│   ├── web-server/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # Axum + gRPC server entry point
│   │       ├── handlers/
│   │       │   ├── mod.rs
│   │       │   ├── health.rs    # Health check endpoint
│   │       │   └── api.rs       # REST API handlers
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs      # Authentication middleware
│   │       │   └── logging.rs   # Request logging
│   │       └── config.rs        # Configuration management
│   │
│   └── shared/                  # (Optional) Shared utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── error.rs         # Common error types
│           └── utils.rs         # Shared utilities
│
├── k8s/
│   ├── base/                    # Kustomize base
│   │   ├── kustomization.yaml
│   │   ├── namespace.yaml
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── ingress.yaml
│   │   └── configmap.yaml
│   │
│   ├── overlays/
│   │   ├── development/
│   │   │   ├── kustomization.yaml
│   │   │   ├── deployment-patch.yaml
│   │   │   └── ingress-patch.yaml
│   │   │
│   │   ├── staging/
│   │   │   ├── kustomization.yaml
│   │   │   ├── deployment-patch.yaml
│   │   │   └── ingress-patch.yaml
│   │   │
│   │   └── production/
│   │       ├── kustomization.yaml
│   │       ├── deployment-patch.yaml
│   │       ├── ingress-patch.yaml
│   │       └── hpa.yaml         # Horizontal Pod Autoscaler
│   │
│   └── secrets/                 # (gitignored)
│       └── .gitkeep
│
├── scripts/
│   ├── build.sh                 # Local build script
│   ├── deploy.sh                # Manual deployment script
│   ├── rollback.sh              # Rollback script
│   └── test-grpc.sh             # gRPC testing script
│
├── docs/
│   ├── API.md                   # API documentation
│   ├── DEPLOYMENT.md            # Deployment guide
│   ├── DEVELOPMENT.md           # Development guide
│   └── architecture.md          # Architecture overview
│
├── tests/
│   ├── integration/
│   │   └── grpc_test.rs         # Integration tests
│   └── e2e/
│       └── test_suite.rs        # End-to-end tests
│
├── .gitignore
├── .dockerignore
├── Cargo.toml                   # Workspace configuration
├── Cargo.lock
├── Dockerfile                   # Production Dockerfile
├── Dockerfile.dev               # Development Dockerfile
├── README.md
├── LICENSE
└── .env.example                 # Environment variables template
```

## Quick Start

### Development

```bash
# Run the server
cargo run -p web-server

# Run tests
cargo test --workspace

# Test gRPC
./scripts/test-grpc.sh
```

### Deployment

```bash
# Build Docker image
docker build -t web-server:latest .

# Deploy to Kubernetes
kubectl apply -k k8s/overlays/production
```

## Documentation

- [API Documentation](docs/API.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Development Guide](docs/DEVELOPMENT.md)

## License

MIT


