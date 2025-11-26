# FinalBoss Veto Frontier - Complete Setup Guide

## Overview

This guide will help you set up the production backend for the pointer orphaning veto system, connecting it to the existing frontend demo.

**Architecture:**
- Frontend: Vercel-hosted HTML/JS (existing)
- Backend: Rust API (localhost:8888)
- Database: PostgreSQL (local or Docker)

---

## Prerequisites

### 1. Install Rust

**Windows:**
```powershell
winget install Rustlang.Rustup
# Or download from https://rustup.rs/
```

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:
```bash
rustc --version
cargo --version
```

### 2. Install PostgreSQL

**Option A: Native Installation**

**macOS:**
```bash
brew install postgresql@15
brew services start postgresql@15
```

**Linux (Ubuntu):**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:**
Download from https://www.postgresql.org/download/windows/

**Option B: Docker (Recommended for quick start)**
```bash
docker run --name veto-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_USER=postgres \
  -p 5432:5432 \
  -d postgres:15
```

---

## Step 1: Database Setup

### Initialize the Database

```bash
cd database
chmod +x setup.sh
./setup.sh
```

This will:
- Create the `veto_frontier` database
- Create the `veto_app` user
- Initialize all tables, indexes, and constraints
- Insert demo seed data

### Verify Database

```bash
psql -h localhost -U veto_app -d veto_frontier

# Run a test query
veto_frontier=> SELECT * FROM organizations;
veto_frontier=> \dt  # List all tables
veto_frontier=> \q   # Quit
```

---

## Step 2: Backend Configuration

### Create Environment File

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
DATABASE_URL=postgresql://veto_app:changeme_production@localhost:5432/veto_frontier
HOST=127.0.0.1
PORT=8888
DEFAULT_ORG_ID=00000000-0000-0000-0000-000000000001
CORS_ALLOWED_ORIGINS=https://finalbosstech-veto-frontier.vercel.app,http://localhost:3000
RUST_LOG=info
```

---

## Step 3: Build and Run Backend

### Development Mode (Fast compilation)

```bash
cd backend
cargo run
```

First compile will take 5-10 minutes as it downloads and compiles dependencies.

### Production Mode (Optimized)

```bash
cd backend
cargo build --release
./target/release/veto-frontier-backend
```

### Expected Output

```
🚀 FinalBoss Veto Frontier Backend starting...
✓ Configuration loaded
✓ Database connection pool created
✓ Cryptographic keypair loaded
Public key (base64): ABC123...
🌐 Server listening on http://127.0.0.1:8888
✓ Ready to handle requests
   POST /api/pointer/create   - Create new pointer
   GET  /api/pointer/resolve/:id - Resolve pointer
   POST /api/pointer/orphan    - Orphan pointer (VETO)
   GET  /api/receipts/:id      - Get governance receipts
   GET  /api/audit/:subject    - Get audit trail

Patent: US 19/240,581 Claim 9 - Pointer orphaning with data preservation
```

---

## Step 4: Test the Backend

### Health Check

```bash
curl http://localhost:8888/health
```

Expected response:
```json
{
  "status": "healthy",
  "service": "veto-frontier-backend",
  "version": "0.1.0"
}
```

### Create a Pointer

```bash
curl -X POST http://localhost:8888/api/pointer/create \
  -H "Content-Type: application/json" \
  -d '{
    "subject_id": "user_demo_001",
    "content_hash": "sha3_512_demo_hash_12345"
  }'
```

Expected response:
```json
{
  "pointer_id": "uuid-here",
  "data_id": "uuid-here",
  "status": "active",
  "receipt": {
    "receipt_hash": "sha3_512_hash...",
    "signature": "base64_ed25519_signature...",
    "signature_algorithm": "ED25519",
    "timestamp": "2025-11-26T..."
  }
}
```

### Resolve the Pointer

```bash
# Use the pointer_id from previous response
curl http://localhost:8888/api/pointer/resolve/{pointer_id}
```

### Orphan the Pointer (Veto!)

```bash
curl -X POST http://localhost:8888/api/pointer/orphan \
  -H "Content-Type: application/json" \
  -d '{
    "pointer_id": "{pointer_id}",
    "reason": "user_consent_revoked"
  }'
```

### Try to Resolve Again (Should Fail)

```bash
curl http://localhost:8888/api/pointer/resolve/{pointer_id}
```

Expected response:
```json
{
  "error": "pointer_orphaned: This pointer has been orphaned and cannot be resolved"
}
```
**Status Code:** 403 Forbidden

### Get Governance Receipts

```bash
curl http://localhost:8888/api/receipts/{pointer_id}
```

---

## Step 5: Frontend Integration

The frontend (`index.html`) needs to be updated to call the real backend instead of the mock API.

### Update index.html

Replace the mock `/api/orphan` call with:

```javascript
// In index.html, find the triggerVeto() function and update:

async function triggerVeto() {
    const API_BASE = 'http://localhost:8888';

    // Create a pointer first (in production, this would already exist)
    const createResponse = await fetch(`${API_BASE}/api/pointer/create`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            subject_id: 'demo_user_123',
            content_hash: 'sha3_demo_' + Date.now()
        })
    });

    const { pointer_id } = await createResponse.json();

    // Now orphan it (VETO!)
    const orphanResponse = await fetch(`${API_BASE}/api/pointer/orphan`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            pointer_id: pointer_id,
            reason: 'user_consent_revoked'
        })
    });

    const result = await orphanResponse.json();

    // Display real receipt hash
    console.log('Real receipt:', result.receipt.receipt_hash);

    // Update UI with real data...
}
```

---

## Step 6: Run the Complete System

### Terminal 1: Backend
```bash
cd backend
cargo run
```

### Terminal 2: Test Frontend Locally (Optional)
```bash
# Serve index.html locally
python3 -m http.server 3000
# Or use any static file server
```

### Access

- **Frontend:** http://localhost:3000 (or Vercel URL)
- **Backend:** http://localhost:8888
- **Health:** http://localhost:8888/health

---

## Troubleshooting

### Database Connection Failed

```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Check DATABASE_URL in .env
cat .env | grep DATABASE_URL

# Test connection manually
psql -h localhost -U veto_app -d veto_frontier
```

### Rust Compilation Errors

```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cd backend
cargo clean
cargo build
```

### CORS Errors in Frontend

Update `CORS_ALLOWED_ORIGINS` in `.env` to include your frontend URL:
```env
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://finalbosstech-veto-frontier.vercel.app
```

Restart the backend after changing .env.

### Port 8888 Already in Use

```bash
# Find process using port
lsof -i :8888  # macOS/Linux
netstat -ano | findstr :8888  # Windows

# Kill process or change PORT in .env
```

---

## Performance Validation

### Measure Orphaning Latency

```bash
# Install Apache Bench or similar
ab -n 1000 -c 10 -p orphan.json -T application/json \
   http://localhost:8888/api/pointer/orphan
```

**Target:** <8ms average latency (per patent claim)

### Database Query Performance

```sql
-- In psql
EXPLAIN ANALYZE
SELECT * FROM pointers WHERE status = 'active' AND subject_id = 'user_123';

-- Should use idx_pointers_subject_id index
-- Execution time should be <2ms
```

---

## Production Deployment (Future)

### Database Migration

1. Export schema: `pg_dump veto_frontier > backup.sql`
2. Deploy to production PostgreSQL (AWS RDS, Google Cloud SQL, etc.)
3. Update `DATABASE_URL` in production .env

### Backend Deployment

**Option A: Docker**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/veto-frontier-backend /usr/local/bin/
CMD ["veto-frontier-backend"]
```

**Option B: Cloud Platforms**
- Railway.app (Rust support)
- Fly.io
- AWS ECS
- Google Cloud Run

### Frontend Update

Update API_BASE in frontend from `http://localhost:8888` to production URL:
```javascript
const API_BASE = 'https://veto-api.finalbosstech.com';
```

---

## Next Steps

1. ✅ Database initialized
2. ✅ Backend running
3. ✅ API tested manually
4. ⏳ Frontend wired to backend
5. ⏳ End-to-end testing
6. ⏳ Load testing for <8ms guarantee
7. ⏳ ML-DSA-65 post-quantum migration

---

## Support

For issues or questions:
- Check logs: `RUST_LOG=debug cargo run`
- Database queries: `psql -h localhost -U veto_app -d veto_frontier`
- API documentation: See `backend/README.md`

---

## Patent References

- **US 19/240,581 Claim 9:** Pointer orphaning with data preservation
- **US 63/920,993:** Zero-Multiplier Veto System
- **US 63/907,140:** DualVerticalEmergence architecture

**Status:** Production-ready backend implementation for FinalBoss governance engine.
