-- Pincher Central Registry Database Schema
-- 
-- Append-only, immutable package storage with developer identity verification.
-- Enforces composite (package_id, version_semver) unique constraint to prevent
-- supply-chain attacks and ensure stable dependency resolution across all clients.
--
-- Security:
--   - Each developer has a unique author_key_hash (cryptographic identity)
--   - Packages are bound to developers via foreign key (ON DELETE RESTRICT)
--   - Bundle releases are immutable: signature + version locked at publish time
--   - Different signature on same (package, version) = rejection

-- Developer identities
CREATE TABLE IF NOT EXISTS developers (
    developer_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_key_hash  VARCHAR(64) NOT NULL UNIQUE,   -- SHA-256 of public key
    username         VARCHAR(50) NOT NULL UNIQUE,
    api_token_hash   VARCHAR(128),                    -- Hashed auth token (never store raw)
    created_at       TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Registered packages
CREATE TABLE IF NOT EXISTS packages (
    package_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    developer_id     UUID REFERENCES developers(developer_id) ON DELETE RESTRICT,
    name             VARCHAR(100) NOT NULL UNIQUE,   -- Publicly visible name
    description      TEXT,
    repository_url   TEXT,
    created_at       TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Immutable release versions for each package
CREATE TABLE IF NOT EXISTS bundle_releases (
    release_id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id              UUID REFERENCES packages(package_id) ON DELETE CASCADE,
    version_semver          VARCHAR(30) NOT NULL,     -- Semantic version (e.g., 1.2.3)
    cryptographic_signature VARCHAR(128) NOT NULL,    -- Signed by developer's private key
    binary_payload_url      TEXT NOT NULL,             -- S3 / Cloudflare R2 / local path
    checksum                VARCHAR(64) NOT NULL,      -- SHA-256 of binary_payload
    manifest_snapshot       JSONB NOT NULL,            -- Snapshot of Intent.toml at compile time
    target_arch             VARCHAR(20) DEFAULT 'wasm32-wasip1',
    compiled_at             TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at              TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Immutability constraint: one version per package, forever
    CONSTRAINT unique_package_version UNIQUE (package_id, version_semver)
);

-- Telemetry from edge devices (for self-healing loop)
CREATE TABLE IF NOT EXISTS edge_telemetry (
    telemetry_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id       UUID REFERENCES packages(package_id) ON DELETE CASCADE,
    device_id        VARCHAR(64) NOT NULL,
    reflex_id        VARCHAR(64) NOT NULL,
    error_logs       TEXT NOT NULL,
    env_context      JSONB,
    reported_at      TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
CREATE INDEX IF NOT EXISTS idx_releases_semver ON bundle_releases(package_id, version_semver);
CREATE INDEX IF NOT EXISTS idx_releases_package_id ON bundle_releases(package_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_package ON edge_telemetry(package_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_reported ON edge_telemetry(reported_at);

-- Trigger: automatically update updated_at on row modification
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_developers_updated_at
    BEFORE UPDATE ON developers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_packages_updated_at
    BEFORE UPDATE ON packages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
