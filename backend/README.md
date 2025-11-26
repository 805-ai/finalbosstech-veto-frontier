# Veto Frontier Backend (Rust)

High-performance backend implementation for the patent-pending pointer orphaning system.

## Installation

### 1. Install Rust

**Windows:**
```bash
# Download and run rustup-init.exe
# https://rustup.rs/
winget install Rustlang.Rustup
```

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

### 2. Install PostgreSQL (if not already installed)

See `../database/README.md` for PostgreSQL setup instructions.

### 3. Initialize Database

```bash
cd ../database
chmod +x setup.sh
./setup.sh
```

### 4. Configure Environment

```bash
cd ../backend
cp ../.env.example ../.env
# Edit ../.env with your database credentials
```

### 5. Build and Run

```bash
# Development mode (fast compilation)
cargo run

# Production mode (optimized for sub-8ms performance)
cargo build --release
./target/release/veto-frontier-backend
```

The server will start on `http://localhost:8888`

## API Endpoints

### Health Check
```bash
GET /health
```

### Create Pointer
```bash
POST /api/pointer/create
Content-Type: application/json

{
  "org_id": "00000000-0000-0000-0000-000000000001",
  "subject_id": "user_123",
  "content_hash": "sha3_512_hash_here",
  "encrypted_payload": "base64_encoded_optional"
}

Response: 201 Created
{
  "pointer_id": "uuid",
  "data_id": "uuid",
  "receipt": {
    "receipt_hash": "sha3_512...",
    "signature": "ed25519_signature_base64",
    "timestamp": "2025-11-26T..."
  }
}
```

### Resolve Pointer
```bash
GET /api/pointer/resolve/{pointer_id}

Response: 200 OK (if active)
{
  "pointer_id": "uuid",
  "data_id": "uuid",
  "subject_id": "user_123",
  "content_hash": "sha3_512...",
  "status": "active",
  "receipt": {...}
}

Response: 403 Forbidden (if orphaned)
{
  "error": "pointer_orphaned",
  "message": "This pointer has been orphaned and cannot be resolved",
  "orphaned_at": "2025-11-26T..."
}
```

### Orphan Pointer (Veto)
```bash
POST /api/pointer/orphan
Content-Type: application/json

{
  "pointer_id": "uuid",
  "reason": "user_consent_revoked"
}

Response: 200 OK
{
  "pointer_id": "uuid",
  "status": "orphaned",
  "orphaned_at": "2025-11-26T...",
  "receipt": {
    "receipt_hash": "sha3_512...",
    "signature": "ed25519_signature_base64",
    "prev_hash": "chain_link_to_previous_receipt"
  }
}
```

### Get Receipts
```bash
GET /api/receipts/{pointer_id}

Response: 200 OK
{
  "pointer_id": "uuid",
  "receipts": [
    {
      "operation": "create",
      "receipt_hash": "...",
      "signature": "...",
      "timestamp": "..."
    },
    {
      "operation": "orphan",
      "receipt_hash": "...",
      "signature": "...",
      "prev_hash": "...",
      "timestamp": "..."
    }
  ]
}
```

### Audit Trail
```bash
GET /api/audit/{subject_id}

Response: 200 OK
{
  "subject_id": "user_123",
  "total_pointers": 5,
  "active_pointers": 2,
  "orphaned_pointers": 3,
  "audit_trail": [...]
}
```

## Architecture

```
backend/
├── Cargo.toml                 # Dependencies and build config
├── src/
│   ├── main.rs                # Entry point, server initialization
│   ├── config.rs              # Configuration and environment
│   ├── crypto/
│   │   ├── mod.rs             # Crypto module exports
│   │   ├── ed25519.rs         # ED25519 signing
│   │   ├── hashing.rs         # SHA3-512 hashing
│   │   └── receipts.rs        # Receipt generation logic
│   ├── db/
│   │   ├── mod.rs             # Database module exports
│   │   ├── connection.rs      # SQLx connection pool
│   │   ├── models.rs          # Database models
│   │   └── queries.rs         # SQL queries
│   ├── api/
│   │   ├── mod.rs             # API module exports
│   │   ├── handlers.rs        # Request handlers
│   │   ├── routes.rs          # Route definitions
│   │   └── errors.rs          # Error types and responses
│   └── enforcement/
│       ├── mod.rs             # Enforcement module exports
│       └── pointer_guard.rs   # Orphaned pointer enforcement
```

## Performance Targets

- **Orphaning Latency:** <8ms end-to-end (per patent claim)
- **Throughput:** 10,000 req/s sustained
- **Database Queries:** <2ms average
- **Signature Generation:** <1ms (ED25519)

## Development

### Run Tests
```bash
cargo test
```

### Run with Debug Logging
```bash
RUST_LOG=debug cargo run
```

### Format Code
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

### Benchmark
```bash
cargo bench
```

## Deployment

### Build Optimized Binary
```bash
cargo build --release
```

The optimized binary will be at: `target/release/veto-frontier-backend`

### Docker (TODO)
```bash
docker build -t veto-frontier-backend .
docker run -p 8888:8888 --env-file .env veto-frontier-backend
```

## Future: ML-DSA-65 Migration

When ready to migrate to post-quantum signatures:

1. Add dependency: `pqcrypto-dilithium = "0.5"`
2. Update `src/crypto/mldsa.rs` with ML-DSA-65 implementation
3. Change `signature_algorithm` in receipts from 'ED25519' to 'ML-DSA-65'
4. Signature size will increase: 64 bytes → 3,309 bytes
5. Performance impact: ~2-3ms additional latency

## Patent References

- **US 19/240,581 Claim 9:** "orphaning said pointer while preserving the underlying data object"
- **US 63/920,993:** Zero-Multiplier Veto System
- **US 63/907,140:** DualVerticalEmergence (DVE) architecture
