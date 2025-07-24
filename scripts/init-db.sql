-- =====================================================================================
-- File: scripts/init-db.sql
-- Description: Enterprise database initialization for StableRWA Platform
-- Author: arkSong (arksong2018@gmail.com)
-- Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
-- =====================================================================================

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS assets;
CREATE SCHEMA IF NOT EXISTS users;
CREATE SCHEMA IF NOT EXISTS transactions;
CREATE SCHEMA IF NOT EXISTS compliance;
CREATE SCHEMA IF NOT EXISTS audit;

-- Users table
CREATE TABLE IF NOT EXISTS users.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    phone VARCHAR(20),
    country_code VARCHAR(3),
    kyc_status VARCHAR(20) DEFAULT 'pending',
    kyc_level INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Assets table
CREATE TABLE IF NOT EXISTS assets.assets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    description TEXT,
    asset_type VARCHAR(50) NOT NULL,
    total_supply DECIMAL(36, 18) DEFAULT 0,
    circulating_supply DECIMAL(36, 18) DEFAULT 0,
    underlying_asset_value DECIMAL(20, 2),
    currency VARCHAR(3) DEFAULT 'USD',
    blockchain VARCHAR(20) NOT NULL,
    contract_address VARCHAR(42),
    owner_id UUID REFERENCES users.users(id),
    status VARCHAR(20) DEFAULT 'draft',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions.transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_hash VARCHAR(66) UNIQUE,
    from_address VARCHAR(42),
    to_address VARCHAR(42),
    asset_id UUID REFERENCES assets.assets(id),
    amount DECIMAL(36, 18) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    blockchain VARCHAR(20) NOT NULL,
    block_number BIGINT,
    gas_used BIGINT,
    gas_price DECIMAL(36, 18),
    fee DECIMAL(36, 18),
    user_id UUID REFERENCES users.users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    confirmed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Compliance records
CREATE TABLE IF NOT EXISTS compliance.kyc_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users.users(id),
    document_type VARCHAR(50) NOT NULL,
    document_number VARCHAR(100),
    issuing_country VARCHAR(3),
    expiry_date DATE,
    verification_status VARCHAR(20) DEFAULT 'pending',
    verification_date TIMESTAMP WITH TIME ZONE,
    verifier_id VARCHAR(100),
    risk_score INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Audit logs
CREATE TABLE IF NOT EXISTS audit.audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id UUID,
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    session_id VARCHAR(255),
    status VARCHAR(20),
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Price feeds table
CREATE TABLE IF NOT EXISTS assets.price_feeds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_symbol VARCHAR(10) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    volume_24h DECIMAL(20, 2),
    change_24h DECIMAL(10, 4),
    market_cap DECIMAL(20, 2),
    source VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Wallets table
CREATE TABLE IF NOT EXISTS users.wallets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users.users(id),
    address VARCHAR(42) UNIQUE NOT NULL,
    blockchain VARCHAR(20) NOT NULL,
    wallet_type VARCHAR(20) DEFAULT 'hot',
    is_primary BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_used TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Asset holdings
CREATE TABLE IF NOT EXISTS assets.holdings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users.users(id),
    asset_id UUID REFERENCES assets.assets(id),
    wallet_id UUID REFERENCES users.wallets(id),
    balance DECIMAL(36, 18) DEFAULT 0,
    locked_balance DECIMAL(36, 18) DEFAULT 0,
    average_cost DECIMAL(20, 8),
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    UNIQUE(user_id, asset_id, wallet_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users.users(email);
CREATE INDEX IF NOT EXISTS idx_users_kyc_status ON users.users(kyc_status);
CREATE INDEX IF NOT EXISTS idx_assets_symbol ON assets.assets(symbol);
CREATE INDEX IF NOT EXISTS idx_assets_type ON assets.assets(asset_type);
CREATE INDEX IF NOT EXISTS idx_assets_blockchain ON assets.assets(blockchain);
CREATE INDEX IF NOT EXISTS idx_transactions_hash ON transactions.transactions(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_user ON transactions.transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_asset ON transactions.transactions(asset_id);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions.transactions(status);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit.audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit.audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit.audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_price_feeds_symbol ON assets.price_feeds(asset_symbol);
CREATE INDEX IF NOT EXISTS idx_price_feeds_timestamp ON assets.price_feeds(timestamp);
CREATE INDEX IF NOT EXISTS idx_wallets_user ON users.wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_address ON users.wallets(address);
CREATE INDEX IF NOT EXISTS idx_holdings_user ON assets.holdings(user_id);
CREATE INDEX IF NOT EXISTS idx_holdings_asset ON assets.holdings(asset_id);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply updated_at triggers
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users.users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_assets_updated_at BEFORE UPDATE ON assets.assets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kyc_records_updated_at BEFORE UPDATE ON compliance.kyc_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert sample data for testing
INSERT INTO users.users (email, password_hash, first_name, last_name, kyc_status, is_verified) VALUES
('admin@stablerwa.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uO.G', 'Admin', 'User', 'approved', true),
('test@stablerwa.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uO.G', 'Test', 'User', 'pending', false)
ON CONFLICT (email) DO NOTHING;

-- Insert sample assets
INSERT INTO assets.assets (name, symbol, description, asset_type, blockchain, status) VALUES
('StableRWA Real Estate Token', 'SRWA-RE', 'Tokenized real estate portfolio', 'real_estate', 'ethereum', 'active'),
('StableRWA Gold Token', 'SRWA-GOLD', 'Gold-backed digital asset', 'commodity', 'ethereum', 'active'),
('StableRWA Art Token', 'SRWA-ART', 'Fine art tokenization', 'art', 'ethereum', 'draft')
ON CONFLICT DO NOTHING;

-- Insert sample price feeds
INSERT INTO assets.price_feeds (asset_symbol, price, volume_24h, change_24h, source) VALUES
('SRWA-RE', 100.50, 1000000.00, 2.5, 'internal'),
('SRWA-GOLD', 1950.75, 500000.00, -0.8, 'internal'),
('BTC', 45000.00, 25000000000.00, 3.2, 'binance'),
('ETH', 2800.00, 15000000000.00, 1.8, 'binance')
ON CONFLICT DO NOTHING;

-- Grant permissions
GRANT USAGE ON SCHEMA assets TO stablerwa;
GRANT USAGE ON SCHEMA users TO stablerwa;
GRANT USAGE ON SCHEMA transactions TO stablerwa;
GRANT USAGE ON SCHEMA compliance TO stablerwa;
GRANT USAGE ON SCHEMA audit TO stablerwa;

GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA assets TO stablerwa;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA users TO stablerwa;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA transactions TO stablerwa;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA compliance TO stablerwa;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA audit TO stablerwa;

GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA assets TO stablerwa;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA users TO stablerwa;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA transactions TO stablerwa;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA compliance TO stablerwa;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA audit TO stablerwa;
