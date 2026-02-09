-- Initial schema for ZKP Privacy Airdrop
-- Migration: 20240101000001_initial_schema

-- Create claims table to track all claim attempts
CREATE TABLE IF NOT EXISTS claims (
    id BIGSERIAL PRIMARY KEY,
    nullifier VARCHAR(66) NOT NULL UNIQUE,
    recipient VARCHAR(42) NOT NULL,
    proof_type VARCHAR(20) NOT NULL,
    tx_hash VARCHAR(66),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    gas_used BIGINT,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index on nullifier for fast lookups
CREATE INDEX IF NOT EXISTS idx_claims_nullifier ON claims(nullifier);
CREATE INDEX IF NOT EXISTS idx_claims_status ON claims(status);
CREATE INDEX IF NOT EXISTS idx_claims_created_at ON claims(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_claims_recipient ON claims(recipient);

-- Create stats table for aggregated statistics
CREATE TABLE IF NOT EXISTS stats (
    id BIGSERIAL PRIMARY KEY,
    stat_key VARCHAR(100) NOT NULL UNIQUE,
    stat_value BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insert initial stats
INSERT INTO stats (stat_key, stat_value) VALUES
    ('total_claims', 0),
    ('successful_claims', 0),
    ('failed_claims', 0),
    ('pending_claims', 0),
    ('total_tokens_claimed', 0)
ON CONFLICT (stat_key) DO NOTHING;

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_claims_updated_at BEFORE UPDATE ON claims
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create index for rate limiting by IP
CREATE INDEX IF NOT EXISTS idx_claims_ip_address ON claims(ip_address);
