# Getting Started with Stheno

Welcome to Stheno, the high-performance, modular commerce engine built in Rust.

---

## 1. Quick Start

### Prerequisites
- Rust 1.80+
- PostgreSQL 15+
- Redis 7+

### Running Locally
```bash
# Clone and setup environment
cp .env.example .env

# Run database migrations
cargo run -p stheno-cli -- migrate

# Start the server
cargo run -p stheno-server
```

The API will be available at `http://localhost:9000`.
- Storefront API: `http://localhost:9000/store`
- Admin Dashboard API: `http://localhost:9000/admin`
- Health check: `http://localhost:9000/health`
