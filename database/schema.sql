-- FinalBoss Veto Frontier - Production Database Schema
-- Patent-referenced pointer orphaning system
-- US 19/240,581 Claim 9: "orphaning said pointer while preserving the underlying data object"
-- US 63/920,993: Zero-Multiplier Veto System

-- ============================================================================
-- EXTENSIONS
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- ENUMS
-- ============================================================================

CREATE TYPE pointer_status AS ENUM ('active', 'orphaned');
CREATE TYPE receipt_operation AS ENUM ('create', 'resolve', 'orphan');

-- ============================================================================
-- ORGANIZATIONS TABLE
-- ============================================================================

CREATE TABLE organizations (
    org_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT org_name_not_empty CHECK (length(trim(name)) > 0)
);

CREATE INDEX idx_organizations_created_at ON organizations(created_at DESC);

-- ============================================================================
-- DATA_STORE TABLE
-- ============================================================================
-- Stores the actual data objects that pointers reference
-- Data persists even when pointers are orphaned (per patent claim)

CREATE TABLE data_store (
    data_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    subject_id VARCHAR(255) NOT NULL,
    content_hash VARCHAR(128) NOT NULL, -- SHA3-512 hash of content
    encrypted_payload BYTEA, -- Actual encrypted data (optional)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT subject_id_not_empty CHECK (length(trim(subject_id)) > 0),
    CONSTRAINT content_hash_not_empty CHECK (length(trim(content_hash)) > 0)
);

CREATE INDEX idx_data_store_org_id ON data_store(org_id);
CREATE INDEX idx_data_store_subject_id ON data_store(subject_id);
CREATE INDEX idx_data_store_created_at ON data_store(created_at DESC);
CREATE INDEX idx_data_store_content_hash ON data_store(content_hash);

-- ============================================================================
-- POINTERS TABLE
-- ============================================================================
-- Core patent implementation: pointers can be orphaned without deleting data
-- When status='orphaned', pointer resolution is blocked by enforcement layer

CREATE TABLE pointers (
    pointer_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    data_id UUID NOT NULL REFERENCES data_store(data_id) ON DELETE CASCADE,
    subject_id VARCHAR(255) NOT NULL,
    status pointer_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    orphaned_at TIMESTAMPTZ,
    orphan_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT subject_id_not_empty CHECK (length(trim(subject_id)) > 0),
    CONSTRAINT orphaned_at_valid CHECK (
        (status = 'orphaned' AND orphaned_at IS NOT NULL) OR
        (status = 'active' AND orphaned_at IS NULL)
    )
);

CREATE INDEX idx_pointers_org_id ON pointers(org_id);
CREATE INDEX idx_pointers_data_id ON pointers(data_id);
CREATE INDEX idx_pointers_subject_id ON pointers(subject_id);
CREATE INDEX idx_pointers_status ON pointers(status);
CREATE INDEX idx_pointers_created_at ON pointers(created_at DESC);
CREATE INDEX idx_pointers_orphaned_at ON pointers(orphaned_at DESC) WHERE orphaned_at IS NOT NULL;

-- Composite index for common query pattern: org + subject + status
CREATE INDEX idx_pointers_org_subject_status ON pointers(org_id, subject_id, status);

-- ============================================================================
-- GOVERNANCE_RECEIPTS TABLE
-- ============================================================================
-- Cryptographically signed audit trail of all pointer operations
-- Implements chain hashing for tamper-evidence

CREATE TABLE governance_receipts (
    receipt_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pointer_id UUID NOT NULL REFERENCES pointers(pointer_id) ON DELETE CASCADE,
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    operation receipt_operation NOT NULL,

    -- Receipt content (canonical JSON serialized)
    receipt_json JSONB NOT NULL,
    receipt_hash VARCHAR(128) NOT NULL, -- SHA3-512 hash of canonical JSON

    -- Cryptographic signature (ED25519 initially, ML-DSA-65 later)
    signature BYTEA NOT NULL,
    signature_algorithm VARCHAR(50) NOT NULL DEFAULT 'ED25519',

    -- Chain linking for audit trail
    prev_hash VARCHAR(128), -- Links to previous receipt in chain

    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT receipt_hash_not_empty CHECK (length(trim(receipt_hash)) > 0),
    CONSTRAINT signature_not_empty CHECK (length(signature) > 0),
    CONSTRAINT signature_algorithm_valid CHECK (signature_algorithm IN ('ED25519', 'ML-DSA-65'))
);

CREATE INDEX idx_receipts_pointer_id ON governance_receipts(pointer_id);
CREATE INDEX idx_receipts_org_id ON governance_receipts(org_id);
CREATE INDEX idx_receipts_operation ON governance_receipts(operation);
CREATE INDEX idx_receipts_timestamp ON governance_receipts(timestamp DESC);
CREATE INDEX idx_receipts_receipt_hash ON governance_receipts(receipt_hash);
CREATE INDEX idx_receipts_prev_hash ON governance_receipts(prev_hash);

-- Composite index for audit trail queries
CREATE INDEX idx_receipts_org_timestamp ON governance_receipts(org_id, timestamp DESC);
CREATE INDEX idx_receipts_pointer_timestamp ON governance_receipts(pointer_id, timestamp DESC);

-- ============================================================================
-- AUDIT_LOG TABLE
-- ============================================================================
-- Comprehensive audit log for compliance and debugging

CREATE TABLE audit_log (
    log_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES organizations(org_id) ON DELETE CASCADE,
    pointer_id UUID REFERENCES pointers(pointer_id) ON DELETE SET NULL,
    receipt_id UUID REFERENCES governance_receipts(receipt_id) ON DELETE SET NULL,

    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    actor_id VARCHAR(255), -- User/service that triggered the event
    ip_address INET,
    user_agent TEXT,

    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT event_type_not_empty CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX idx_audit_log_org_id ON audit_log(org_id);
CREATE INDEX idx_audit_log_pointer_id ON audit_log(pointer_id);
CREATE INDEX idx_audit_log_receipt_id ON audit_log(receipt_id);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_log_actor_id ON audit_log(actor_id);

-- Composite index for org audit queries
CREATE INDEX idx_audit_log_org_timestamp ON audit_log(org_id, timestamp DESC);

-- ============================================================================
-- FUNCTIONS & TRIGGERS
-- ============================================================================

-- Update updated_at timestamp automatically
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Automatically log pointer status changes
CREATE OR REPLACE FUNCTION log_pointer_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status != NEW.status THEN
        INSERT INTO audit_log (
            org_id,
            pointer_id,
            event_type,
            event_data,
            actor_id
        ) VALUES (
            NEW.org_id,
            NEW.pointer_id,
            'pointer_status_change',
            jsonb_build_object(
                'old_status', OLD.status::text,
                'new_status', NEW.status::text,
                'orphan_reason', NEW.orphan_reason
            ),
            current_setting('app.current_user_id', true)
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_log_pointer_status_change
    AFTER UPDATE ON pointers
    FOR EACH ROW
    EXECUTE FUNCTION log_pointer_status_change();

-- ============================================================================
-- VIEWS
-- ============================================================================

-- Active pointers view (frequently used query)
CREATE VIEW active_pointers AS
SELECT
    p.pointer_id,
    p.org_id,
    p.data_id,
    p.subject_id,
    p.status,
    p.created_at,
    d.content_hash,
    o.name AS org_name
FROM pointers p
JOIN data_store d ON p.data_id = d.data_id
JOIN organizations o ON p.org_id = o.org_id
WHERE p.status = 'active';

-- Orphaned pointers view with revocation info
CREATE VIEW orphaned_pointers AS
SELECT
    p.pointer_id,
    p.org_id,
    p.data_id,
    p.subject_id,
    p.created_at,
    p.orphaned_at,
    p.orphan_reason,
    d.content_hash,
    o.name AS org_name,
    (p.orphaned_at - p.created_at) AS lifetime_duration
FROM pointers p
JOIN data_store d ON p.data_id = d.data_id
JOIN organizations o ON p.org_id = o.org_id
WHERE p.status = 'orphaned';

-- Receipt chain view for audit trail
CREATE VIEW receipt_chain AS
SELECT
    r.receipt_id,
    r.pointer_id,
    r.org_id,
    r.operation,
    r.receipt_hash,
    r.prev_hash,
    r.signature_algorithm,
    r.timestamp,
    p.subject_id,
    o.name AS org_name
FROM governance_receipts r
JOIN pointers p ON r.pointer_id = p.pointer_id
JOIN organizations o ON r.org_id = o.org_id
ORDER BY r.timestamp DESC;

-- ============================================================================
-- SEED DATA (DEVELOPMENT/DEMO)
-- ============================================================================

-- Insert demo organization
INSERT INTO organizations (org_id, name, metadata)
VALUES (
    '00000000-0000-0000-0000-000000000001'::uuid,
    'FinalBoss Demo Org',
    '{"environment": "demo", "type": "internal"}'::jsonb
);

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE pointers IS 'Core patent implementation: pointers reference data but can be orphaned independently';
COMMENT ON COLUMN pointers.status IS 'active = resolvable, orphaned = blocked by enforcement layer';
COMMENT ON TABLE data_store IS 'Persistent data storage - data survives pointer orphaning per US 19/240,581 Claim 9';
COMMENT ON TABLE governance_receipts IS 'Cryptographically signed audit trail with chain hashing';
COMMENT ON COLUMN governance_receipts.prev_hash IS 'Links to previous receipt hash for tamper-evident chain';
COMMENT ON COLUMN governance_receipts.signature_algorithm IS 'ED25519 (current) or ML-DSA-65 (future post-quantum)';

-- ============================================================================
-- GRANT PERMISSIONS (adjust for your security requirements)
-- ============================================================================

-- Example: Create application role with limited permissions
-- CREATE ROLE veto_app WITH LOGIN PASSWORD 'changeme';
-- GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO veto_app;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO veto_app;
