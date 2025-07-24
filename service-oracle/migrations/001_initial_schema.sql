-- =====================================================================================
-- RWA Tokenization Platform - Oracle Service Initial Schema
-- 
-- Author: arkSong (arksong2018@gmail.com)
-- =====================================================================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create asset_prices table for storing historical price data
CREATE TABLE IF NOT EXISTS asset_prices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_id VARCHAR(50) NOT NULL,
    price DECIMAL(20,8) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    confidence DECIMAL(3,2) NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    source VARCHAR(100) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT price_positive CHECK (price > 0),
    CONSTRAINT confidence_valid CHECK (confidence >= 0 AND confidence <= 1)
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_asset_prices_asset_timestamp 
    ON asset_prices(asset_id, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_asset_prices_timestamp 
    ON asset_prices(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_asset_prices_source 
    ON asset_prices(source);

CREATE INDEX IF NOT EXISTS idx_asset_prices_currency 
    ON asset_prices(currency);

CREATE INDEX IF NOT EXISTS idx_asset_prices_asset_currency 
    ON asset_prices(asset_id, currency);

-- Create composite index for common queries
CREATE INDEX IF NOT EXISTS idx_asset_prices_composite 
    ON asset_prices(asset_id, currency, timestamp DESC);

-- Create partial index for recent prices (last 7 days)
CREATE INDEX IF NOT EXISTS idx_asset_prices_recent 
    ON asset_prices(asset_id, currency, timestamp DESC) 
    WHERE timestamp > NOW() - INTERVAL '7 days';

-- Create providers table for tracking price data providers
CREATE TABLE IF NOT EXISTS providers (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    provider_type VARCHAR(50) NOT NULL,
    api_endpoint VARCHAR(500),
    weight DECIMAL(5,2) DEFAULT 1.00,
    timeout_seconds INTEGER DEFAULT 10,
    rate_limit_per_minute INTEGER DEFAULT 60,
    is_active BOOLEAN DEFAULT true,
    last_update TIMESTAMP WITH TIME ZONE,
    error_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT weight_positive CHECK (weight > 0),
    CONSTRAINT timeout_positive CHECK (timeout_seconds > 0),
    CONSTRAINT rate_limit_positive CHECK (rate_limit_per_minute > 0)
);

-- Create index on providers
CREATE INDEX IF NOT EXISTS idx_providers_active ON providers(is_active);
CREATE INDEX IF NOT EXISTS idx_providers_type ON providers(provider_type);

-- Create provider_health table for tracking provider health status
CREATE TABLE IF NOT EXISTS provider_health (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id VARCHAR(100) NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    is_healthy BOOLEAN NOT NULL,
    response_time_ms INTEGER,
    error_message TEXT,
    checked_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT response_time_positive CHECK (response_time_ms >= 0)
);

-- Create indexes for provider health
CREATE INDEX IF NOT EXISTS idx_provider_health_provider_time 
    ON provider_health(provider_id, checked_at DESC);

CREATE INDEX IF NOT EXISTS idx_provider_health_status 
    ON provider_health(is_healthy, checked_at DESC);

-- Create aggregation_logs table for tracking price aggregation events
CREATE TABLE IF NOT EXISTS aggregation_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_id VARCHAR(50) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    aggregation_method VARCHAR(50) NOT NULL,
    source_count INTEGER NOT NULL,
    final_price DECIMAL(20,8) NOT NULL,
    confidence DECIMAL(3,2) NOT NULL,
    deviation_percent DECIMAL(5,2),
    outliers_removed INTEGER DEFAULT 0,
    processing_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT source_count_positive CHECK (source_count > 0),
    CONSTRAINT final_price_positive CHECK (final_price > 0),
    CONSTRAINT confidence_valid_agg CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT processing_time_positive CHECK (processing_time_ms >= 0)
);

-- Create indexes for aggregation logs
CREATE INDEX IF NOT EXISTS idx_aggregation_logs_asset_time 
    ON aggregation_logs(asset_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_aggregation_logs_method 
    ON aggregation_logs(aggregation_method);

-- Create system_metrics table for storing system performance metrics
CREATE TABLE IF NOT EXISTS system_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(15,4) NOT NULL,
    metric_unit VARCHAR(20),
    tags JSONB,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT metric_name_not_empty CHECK (LENGTH(metric_name) > 0)
);

-- Create indexes for system metrics
CREATE INDEX IF NOT EXISTS idx_system_metrics_name_time 
    ON system_metrics(metric_name, recorded_at DESC);

CREATE INDEX IF NOT EXISTS idx_system_metrics_time 
    ON system_metrics(recorded_at DESC);

-- Create GIN index for JSONB tags
CREATE INDEX IF NOT EXISTS idx_system_metrics_tags 
    ON system_metrics USING GIN (tags);

-- Create alerts table for storing alert configurations and history
CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    alert_type VARCHAR(50) NOT NULL,
    asset_id VARCHAR(50),
    provider_id VARCHAR(100),
    threshold_value DECIMAL(20,8),
    condition_type VARCHAR(20) NOT NULL, -- 'above', 'below', 'equal', 'change'
    is_active BOOLEAN DEFAULT true,
    last_triggered TIMESTAMP WITH TIME ZONE,
    trigger_count INTEGER DEFAULT 0,
    recipient_email VARCHAR(255),
    webhook_url VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT condition_type_valid CHECK (condition_type IN ('above', 'below', 'equal', 'change')),
    CONSTRAINT trigger_count_non_negative CHECK (trigger_count >= 0)
);

-- Create indexes for alerts
CREATE INDEX IF NOT EXISTS idx_alerts_active ON alerts(is_active);
CREATE INDEX IF NOT EXISTS idx_alerts_asset ON alerts(asset_id);
CREATE INDEX IF NOT EXISTS idx_alerts_provider ON alerts(provider_id);
CREATE INDEX IF NOT EXISTS idx_alerts_type ON alerts(alert_type);

-- Create alert_history table for tracking alert triggers
CREATE TABLE IF NOT EXISTS alert_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    alert_id UUID NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
    triggered_value DECIMAL(20,8),
    message TEXT,
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for alert history
CREATE INDEX IF NOT EXISTS idx_alert_history_alert_time 
    ON alert_history(alert_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_alert_history_unresolved 
    ON alert_history(alert_id) WHERE resolved_at IS NULL;

-- Create cache_stats table for tracking cache performance
CREATE TABLE IF NOT EXISTS cache_stats (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cache_type VARCHAR(50) NOT NULL, -- 'price', 'feed', 'provider'
    hit_count INTEGER DEFAULT 0,
    miss_count INTEGER DEFAULT 0,
    eviction_count INTEGER DEFAULT 0,
    memory_usage_bytes BIGINT DEFAULT 0,
    key_count INTEGER DEFAULT 0,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT hit_count_non_negative CHECK (hit_count >= 0),
    CONSTRAINT miss_count_non_negative CHECK (miss_count >= 0),
    CONSTRAINT eviction_count_non_negative CHECK (eviction_count >= 0),
    CONSTRAINT memory_usage_non_negative CHECK (memory_usage_bytes >= 0),
    CONSTRAINT key_count_non_negative CHECK (key_count >= 0)
);

-- Create indexes for cache stats
CREATE INDEX IF NOT EXISTS idx_cache_stats_type_time 
    ON cache_stats(cache_type, recorded_at DESC);

-- Create functions for automatic timestamp updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for automatic timestamp updates
CREATE TRIGGER update_providers_updated_at 
    BEFORE UPDATE ON providers 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_alerts_updated_at 
    BEFORE UPDATE ON alerts 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to calculate cache hit rate
CREATE OR REPLACE FUNCTION calculate_cache_hit_rate(cache_type_param VARCHAR)
RETURNS DECIMAL(5,4) AS $$
DECLARE
    total_hits INTEGER;
    total_misses INTEGER;
    hit_rate DECIMAL(5,4);
BEGIN
    SELECT 
        COALESCE(SUM(hit_count), 0),
        COALESCE(SUM(miss_count), 0)
    INTO total_hits, total_misses
    FROM cache_stats 
    WHERE cache_type = cache_type_param 
    AND recorded_at > NOW() - INTERVAL '1 hour';
    
    IF (total_hits + total_misses) = 0 THEN
        RETURN 0.0000;
    END IF;
    
    hit_rate := total_hits::DECIMAL / (total_hits + total_misses)::DECIMAL;
    RETURN ROUND(hit_rate, 4);
END;
$$ LANGUAGE plpgsql;

-- Create function to clean up old data
CREATE OR REPLACE FUNCTION cleanup_old_data(retention_days INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
    cutoff_date TIMESTAMP WITH TIME ZONE;
BEGIN
    cutoff_date := NOW() - (retention_days || ' days')::INTERVAL;
    
    -- Clean up old asset prices (keep aggregated daily data)
    WITH deleted AS (
        DELETE FROM asset_prices 
        WHERE created_at < cutoff_date 
        AND id NOT IN (
            SELECT DISTINCT ON (asset_id, currency, DATE(timestamp)) id
            FROM asset_prices 
            WHERE created_at < cutoff_date
            ORDER BY asset_id, currency, DATE(timestamp), timestamp DESC
        )
        RETURNING id
    )
    SELECT COUNT(*) INTO deleted_count FROM deleted;
    
    -- Clean up old provider health records
    DELETE FROM provider_health WHERE checked_at < cutoff_date;
    
    -- Clean up old aggregation logs
    DELETE FROM aggregation_logs WHERE created_at < cutoff_date;
    
    -- Clean up old system metrics
    DELETE FROM system_metrics WHERE recorded_at < cutoff_date;
    
    -- Clean up old cache stats
    DELETE FROM cache_stats WHERE recorded_at < cutoff_date;
    
    -- Clean up resolved alert history older than retention period
    DELETE FROM alert_history 
    WHERE created_at < cutoff_date AND resolved_at IS NOT NULL;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Insert default providers
INSERT INTO providers (id, name, provider_type, api_endpoint, weight, rate_limit_per_minute) 
VALUES 
    ('coingecko', 'CoinGecko', 'market_data', 'https://api.coingecko.com/api/v3', 1.00, 50),
    ('binance', 'Binance', 'centralized_exchange', 'https://api.binance.com/api/v3', 1.10, 1200),
    ('coinmarketcap', 'CoinMarketCap', 'market_data', 'https://pro-api.coinmarketcap.com/v1', 1.20, 333)
ON CONFLICT (id) DO NOTHING;

-- Create view for recent price data (last 24 hours)
CREATE OR REPLACE VIEW recent_prices AS
SELECT 
    asset_id,
    currency,
    price,
    confidence,
    source,
    timestamp,
    metadata
FROM asset_prices 
WHERE timestamp > NOW() - INTERVAL '24 hours'
ORDER BY asset_id, currency, timestamp DESC;

-- Create view for provider performance summary
CREATE OR REPLACE VIEW provider_performance AS
SELECT 
    p.id,
    p.name,
    p.provider_type,
    p.is_active,
    p.success_count,
    p.error_count,
    CASE 
        WHEN (p.success_count + p.error_count) = 0 THEN 0
        ELSE ROUND(p.success_count::DECIMAL / (p.success_count + p.error_count)::DECIMAL * 100, 2)
    END as success_rate_percent,
    ph.is_healthy as last_health_status,
    ph.response_time_ms as last_response_time_ms,
    ph.checked_at as last_health_check
FROM providers p
LEFT JOIN LATERAL (
    SELECT is_healthy, response_time_ms, checked_at
    FROM provider_health 
    WHERE provider_id = p.id 
    ORDER BY checked_at DESC 
    LIMIT 1
) ph ON true;

-- Grant permissions (adjust as needed for your security model)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO oracle_service_user;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO oracle_service_user;
-- GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO oracle_service_user;
