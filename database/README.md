# Database Setup

## Quick Start

### 1. Install PostgreSQL

**macOS:**
```bash
brew install postgresql@15
brew services start postgresql@15
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:**
Download from https://www.postgresql.org/download/windows/

Or use Docker:
```bash
docker run --name veto-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_USER=postgres \
  -p 5432:5432 \
  -d postgres:15
```

### 2. Run Setup Script

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

### 3. Configure Environment

Copy the example environment file:
```bash
cp .env.example .env
```

Edit `.env` and update the `DATABASE_URL` with your credentials.

## Schema Overview

### Core Tables

**organizations** - Multi-tenant organization management
- `org_id` (UUID, PK)
- `name` (VARCHAR)
- `created_at`, `updated_at` (TIMESTAMPTZ)

**data_store** - Persistent data storage
- `data_id` (UUID, PK)
- `org_id` (UUID, FK)
- `subject_id` (VARCHAR) - User/entity identifier
- `content_hash` (VARCHAR) - SHA3-512 of content
- `encrypted_payload` (BYTEA) - Optional encrypted data

**pointers** - Patent-pending pointer orphaning system
- `pointer_id` (UUID, PK)
- `data_id` (UUID, FK → data_store)
- `subject_id` (VARCHAR)
- `status` (ENUM: 'active', 'orphaned')
- `orphaned_at` (TIMESTAMPTZ)

**governance_receipts** - Cryptographic audit trail
- `receipt_id` (UUID, PK)
- `pointer_id` (UUID, FK)
- `operation` (ENUM: 'create', 'resolve', 'orphan')
- `receipt_hash` (VARCHAR) - SHA3-512 of canonical JSON
- `signature` (BYTEA) - ED25519 (64 bytes) or ML-DSA-65 (3,309 bytes)
- `prev_hash` (VARCHAR) - Chain linking

**audit_log** - Comprehensive event logging
- `log_id` (UUID, PK)
- `event_type` (VARCHAR)
- `event_data` (JSONB)
- `actor_id`, `ip_address`, `user_agent`

### Key Features

✅ **Pointer Orphaning** (US 19/240,581 Claim 9)
- Pointers can be marked `status='orphaned'`
- Data in `data_store` remains intact
- Query enforcement prevents orphaned pointer resolution

✅ **Cryptographic Receipts**
- Every operation generates a signed receipt
- SHA3-512 hashing for receipt integrity
- ED25519 signatures (64 bytes, fast)
- Chain hashing via `prev_hash` links

✅ **Audit Trail**
- Immutable `governance_receipts` chain
- Automatic `audit_log` entries via triggers
- Subject-level audit queries

✅ **ACID Guarantees**
- PostgreSQL transactions ensure consistency
- Foreign key constraints maintain referential integrity
- Check constraints enforce business rules

## Manual Setup (Alternative)

If the script fails, you can manually initialize:

```bash
# Create database
createdb -h localhost -U postgres veto_frontier

# Create user
psql -h localhost -U postgres -d veto_frontier <<EOF
CREATE ROLE veto_app WITH LOGIN PASSWORD 'changeme_production';
GRANT ALL PRIVILEGES ON DATABASE veto_frontier TO veto_app;
EOF

# Run schema
psql -h localhost -U veto_app -d veto_frontier -f schema.sql
```

## Migrations

Future schema changes should be added to `migrations/`:

```
migrations/
├── 001_initial_schema.sql (this schema)
├── 002_add_mldsa_support.sql (future)
└── 003_add_performance_indexes.sql (future)
```

## Useful Queries

**Count active vs orphaned pointers:**
```sql
SELECT status, COUNT(*) FROM pointers GROUP BY status;
```

**Recent orphaning events:**
```sql
SELECT * FROM orphaned_pointers ORDER BY orphaned_at DESC LIMIT 10;
```

**Audit trail for a subject:**
```sql
SELECT * FROM governance_receipts r
JOIN pointers p ON r.pointer_id = p.pointer_id
WHERE p.subject_id = 'user_123'
ORDER BY r.timestamp DESC;
```

**Verify receipt chain integrity:**
```sql
SELECT
    r1.receipt_hash AS current_hash,
    r1.prev_hash AS claimed_prev_hash,
    r2.receipt_hash AS actual_prev_hash,
    (r1.prev_hash = r2.receipt_hash) AS chain_valid
FROM governance_receipts r1
LEFT JOIN governance_receipts r2 ON r1.prev_hash = r2.receipt_hash
WHERE r1.prev_hash IS NOT NULL;
```
