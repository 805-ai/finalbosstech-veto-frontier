#!/bin/bash

# FinalBoss Veto Frontier - Database Setup Script
# This script creates and initializes the PostgreSQL database

set -e

# Configuration
DB_NAME="${DB_NAME:-veto_frontier}"
DB_USER="${DB_USER:-veto_app}"
DB_PASSWORD="${DB_PASSWORD:-changeme_production}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"

echo "========================================="
echo "FinalBoss Veto Frontier Database Setup"
echo "========================================="
echo ""
echo "Database: $DB_NAME"
echo "User: $DB_USER"
echo "Host: $DB_HOST:$DB_PORT"
echo ""

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "ERROR: PostgreSQL is not installed or not in PATH"
    echo "Please install PostgreSQL first:"
    echo "  - macOS: brew install postgresql@15"
    echo "  - Linux: sudo apt install postgresql postgresql-contrib"
    echo "  - Windows: Download from https://www.postgresql.org/download/windows/"
    exit 1
fi

echo "✓ PostgreSQL found"

# Create database and user (requires superuser privileges)
echo ""
echo "Creating database and user..."
echo "(You may be prompted for PostgreSQL superuser password)"
echo ""

psql -h "$DB_HOST" -p "$DB_PORT" -U postgres <<-EOSQL
    -- Create user if not exists
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = '$DB_USER') THEN
            CREATE ROLE $DB_USER WITH LOGIN PASSWORD '$DB_PASSWORD';
        END IF;
    END
    \$\$;

    -- Create database if not exists
    SELECT 'CREATE DATABASE $DB_NAME OWNER $DB_USER'
    WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\gexec

    -- Grant privileges
    GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOSQL

echo "✓ Database and user created"

# Run schema initialization
echo ""
echo "Initializing database schema..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" < schema.sql

echo ""
echo "✓ Schema initialized successfully"

# Display connection info
echo ""
echo "========================================="
echo "Database ready!"
echo "========================================="
echo ""
echo "Connection URL:"
echo "postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
echo ""
echo "To connect:"
echo "  psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME"
echo ""
echo "Add to your .env file:"
echo "DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
echo ""
