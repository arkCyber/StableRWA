-- =====================================================================================
-- RWA Tokenization Platform - Oracle Service Price Feeds Schema
-- 
-- Author: arkSong (arksong2018@gmail.com)
-- =====================================================================================

-- Create price_feeds table for managing price feed configurations
CREATE TABLE IF NOT EXISTS price_feeds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_id VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    currency VARCHAR(10) NOT NULL,
    update_interval INTEGER NOT NULL, -- seconds
    providers JSONB NOT NULL, -- array of provider IDs
    aggregation_method VARCHAR(50) NOT NULL DEFAULT 'weighted_average',
    deviation_threshold DECIMAL(5,2) NOT NULL DEFAULT 10.00, -- percentage
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT update_interval_positive CHECK (update_interval > 0),
    CONSTRAINT deviation_threshold_positive CHECK (deviation_threshold > 0),
    CONSTRAINT name_not_empty CHECK (LENGTH(TRIM(name)) > 0),
    CONSTRAINT aggregation_method_valid CHECK (
        aggregation_method IN ('mean', 'median', 'weighted_average', 'volume_weighted')
    )
);

-- Create indexes for price feeds
CREATE INDEX IF NOT EXISTS idx_price_feeds_asset ON price_feeds(asset_id);
CREATE INDEX IF NOT EXISTS idx_price_feeds_active ON price_feeds(is_active);
CREATE INDEX IF NOT EXISTS idx_price_feeds_currency ON price_feeds(currency);
CREATE INDEX IF NOT EXISTS idx_price_feeds_asset_currency ON price_feeds(asset_id, currency);

-- Create GIN index for providers JSONB column
CREATE INDEX IF NOT EXISTS idx_price_feeds_providers ON price_feeds USING GIN (providers);

-- Create feed_updates table for tracking feed update history
CREATE TABLE IF NOT EXISTS feed_updates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    feed_id UUID NOT NULL REFERENCES price_feeds(id) ON DELETE CASCADE,
    price DECIMAL(20,8) NOT NULL,
    confidence DECIMAL(3,2) NOT NULL,
    source_count INTEGER NOT NULL,
    aggregation_method VARCHAR(50) NOT NULL,
    processing_time_ms INTEGER,
    deviation_percent DECIMAL(5,2),
    outliers_removed INTEGER DEFAULT 0,
    error_message TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'success', -- 'success', 'failed', 'warning'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT price_positive_feed CHECK (price > 0),
    CONSTRAINT confidence_valid_feed CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT source_count_positive_feed CHECK (source_count > 0),
    CONSTRAINT processing_time_positive_feed CHECK (processing_time_ms >= 0),
    CONSTRAINT outliers_non_negative CHECK (outliers_removed >= 0),
    CONSTRAINT status_valid CHECK (status IN ('success', 'failed', 'warning'))
);

-- Create indexes for feed updates
CREATE INDEX IF NOT EXISTS idx_feed_updates_feed_time 
    ON feed_updates(feed_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_feed_updates_status 
    ON feed_updates(status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_feed_updates_recent 
    ON feed_updates(created_at DESC) 
    WHERE created_at > NOW() - INTERVAL '24 hours';

-- Create feed_schedules table for managing feed update schedules
CREATE TABLE IF NOT EXISTS feed_schedules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    feed_id UUID NOT NULL REFERENCES price_feeds(id) ON DELETE CASCADE,
    next_update TIMESTAMP WITH TIME ZONE NOT NULL,
    last_update TIMESTAMP WITH TIME ZONE,
    update_count INTEGER DEFAULT 0,
    consecutive_failures INTEGER DEFAULT 0,
    is_paused BOOLEAN DEFAULT false,
    pause_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT update_count_non_negative CHECK (update_count >= 0),
    CONSTRAINT consecutive_failures_non_negative CHECK (consecutive_failures >= 0)
);

-- Create unique index to ensure one schedule per feed
CREATE UNIQUE INDEX IF NOT EXISTS idx_feed_schedules_feed_unique 
    ON feed_schedules(feed_id);

-- Create index for next update time
CREATE INDEX IF NOT EXISTS idx_feed_schedules_next_update 
    ON feed_schedules(next_update) WHERE is_paused = false;

-- Create feed_sources table for tracking individual source contributions
CREATE TABLE IF NOT EXISTS feed_sources (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    feed_update_id UUID NOT NULL REFERENCES feed_updates(id) ON DELETE CASCADE,
    provider_id VARCHAR(100) NOT NULL REFERENCES providers(id),
    price DECIMAL(20,8) NOT NULL,
    confidence DECIMAL(3,2) NOT NULL,
    response_time_ms INTEGER,
    weight DECIMAL(5,2) NOT NULL DEFAULT 1.00,
    is_outlier BOOLEAN DEFAULT false,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT price_positive_source CHECK (price > 0),
    CONSTRAINT confidence_valid_source CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT response_time_positive_source CHECK (response_time_ms >= 0),
    CONSTRAINT weight_positive_source CHECK (weight > 0)
);

-- Create indexes for feed sources
CREATE INDEX IF NOT EXISTS idx_feed_sources_update 
    ON feed_sources(feed_update_id);

CREATE INDEX IF NOT EXISTS idx_feed_sources_provider 
    ON feed_sources(provider_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_feed_sources_outlier 
    ON feed_sources(is_outlier, created_at DESC);

-- Create feed_quality_metrics table for tracking feed quality over time
CREATE TABLE IF NOT EXISTS feed_quality_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    feed_id UUID NOT NULL REFERENCES price_feeds(id) ON DELETE CASCADE,
    time_period TIMESTAMP WITH TIME ZONE NOT NULL, -- hourly buckets
    update_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    avg_confidence DECIMAL(3,2),
    avg_deviation DECIMAL(5,2),
    avg_processing_time_ms INTEGER,
    avg_source_count DECIMAL(4,1),
    outlier_count INTEGER DEFAULT 0,
    quality_score DECIMAL(3,2), -- 0.00 to 1.00
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT update_count_non_negative_quality CHECK (update_count >= 0),
    CONSTRAINT success_count_non_negative CHECK (success_count >= 0),
    CONSTRAINT failure_count_non_negative CHECK (failure_count >= 0),
    CONSTRAINT avg_confidence_valid CHECK (avg_confidence IS NULL OR (avg_confidence >= 0 AND avg_confidence <= 1)),
    CONSTRAINT avg_processing_time_positive CHECK (avg_processing_time_ms IS NULL OR avg_processing_time_ms >= 0),
    CONSTRAINT outlier_count_non_negative CHECK (outlier_count >= 0),
    CONSTRAINT quality_score_valid CHECK (quality_score IS NULL OR (quality_score >= 0 AND quality_score <= 1))
);

-- Create unique index for feed quality metrics (one record per feed per hour)
CREATE UNIQUE INDEX IF NOT EXISTS idx_feed_quality_metrics_unique 
    ON feed_quality_metrics(feed_id, time_period);

-- Create index for time-based queries
CREATE INDEX IF NOT EXISTS idx_feed_quality_metrics_time 
    ON feed_quality_metrics(time_period DESC);

-- Create trigger for automatic updated_at timestamp on price_feeds
CREATE TRIGGER update_price_feeds_updated_at 
    BEFORE UPDATE ON price_feeds 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create trigger for automatic updated_at timestamp on feed_schedules
CREATE TRIGGER update_feed_schedules_updated_at 
    BEFORE UPDATE ON feed_schedules 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to calculate feed quality score
CREATE OR REPLACE FUNCTION calculate_feed_quality_score(
    success_rate DECIMAL,
    avg_confidence DECIMAL,
    avg_deviation DECIMAL,
    source_reliability DECIMAL
) RETURNS DECIMAL(3,2) AS $$
DECLARE
    quality_score DECIMAL(3,2);
BEGIN
    -- Weighted quality score calculation
    -- 40% success rate, 30% confidence, 20% low deviation, 10% source reliability
    quality_score := (
        COALESCE(success_rate, 0) * 0.40 +
        COALESCE(avg_confidence, 0) * 0.30 +
        COALESCE((1 - LEAST(avg_deviation / 100.0, 1.0)), 0) * 0.20 +
        COALESCE(source_reliability, 0) * 0.10
    );
    
    RETURN ROUND(LEAST(quality_score, 1.00), 2);
END;
$$ LANGUAGE plpgsql;

-- Create function to update feed quality metrics
CREATE OR REPLACE FUNCTION update_feed_quality_metrics()
RETURNS VOID AS $$
DECLARE
    current_hour TIMESTAMP WITH TIME ZONE;
    feed_record RECORD;
BEGIN
    current_hour := DATE_TRUNC('hour', NOW());
    
    -- Update quality metrics for each active feed
    FOR feed_record IN 
        SELECT id FROM price_feeds WHERE is_active = true
    LOOP
        INSERT INTO feed_quality_metrics (
            feed_id,
            time_period,
            update_count,
            success_count,
            failure_count,
            avg_confidence,
            avg_deviation,
            avg_processing_time_ms,
            avg_source_count,
            outlier_count,
            quality_score
        )
        SELECT 
            feed_record.id,
            current_hour,
            COUNT(*),
            COUNT(*) FILTER (WHERE status = 'success'),
            COUNT(*) FILTER (WHERE status = 'failed'),
            AVG(confidence) FILTER (WHERE status = 'success'),
            AVG(deviation_percent) FILTER (WHERE status = 'success'),
            AVG(processing_time_ms) FILTER (WHERE status = 'success'),
            AVG(source_count) FILTER (WHERE status = 'success'),
            SUM(outliers_removed),
            calculate_feed_quality_score(
                COUNT(*) FILTER (WHERE status = 'success')::DECIMAL / GREATEST(COUNT(*), 1),
                AVG(confidence) FILTER (WHERE status = 'success'),
                AVG(deviation_percent) FILTER (WHERE status = 'success'),
                0.8 -- Default source reliability
            )
        FROM feed_updates
        WHERE feed_id = feed_record.id
        AND created_at >= current_hour
        AND created_at < current_hour + INTERVAL '1 hour'
        ON CONFLICT (feed_id, time_period) DO UPDATE SET
            update_count = EXCLUDED.update_count,
            success_count = EXCLUDED.success_count,
            failure_count = EXCLUDED.failure_count,
            avg_confidence = EXCLUDED.avg_confidence,
            avg_deviation = EXCLUDED.avg_deviation,
            avg_processing_time_ms = EXCLUDED.avg_processing_time_ms,
            avg_source_count = EXCLUDED.avg_source_count,
            outlier_count = EXCLUDED.outlier_count,
            quality_score = EXCLUDED.quality_score;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Create function to get next scheduled feeds
CREATE OR REPLACE FUNCTION get_next_scheduled_feeds(limit_count INTEGER DEFAULT 10)
RETURNS TABLE(
    feed_id UUID,
    asset_id VARCHAR,
    currency VARCHAR,
    update_interval INTEGER,
    providers JSONB,
    aggregation_method VARCHAR,
    next_update TIMESTAMP WITH TIME ZONE
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        pf.id,
        pf.asset_id,
        pf.currency,
        pf.update_interval,
        pf.providers,
        pf.aggregation_method,
        fs.next_update
    FROM price_feeds pf
    JOIN feed_schedules fs ON pf.id = fs.feed_id
    WHERE pf.is_active = true
    AND fs.is_paused = false
    AND fs.next_update <= NOW()
    ORDER BY fs.next_update ASC
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;

-- Create function to update feed schedule after processing
CREATE OR REPLACE FUNCTION update_feed_schedule(
    p_feed_id UUID,
    p_success BOOLEAN,
    p_next_update TIMESTAMP WITH TIME ZONE DEFAULT NULL
) RETURNS VOID AS $$
DECLARE
    current_failures INTEGER;
    new_next_update TIMESTAMP WITH TIME ZONE;
BEGIN
    -- Get current consecutive failures
    SELECT consecutive_failures INTO current_failures
    FROM feed_schedules
    WHERE feed_id = p_feed_id;
    
    -- Calculate next update time if not provided
    IF p_next_update IS NULL THEN
        SELECT 
            NOW() + (update_interval || ' seconds')::INTERVAL
        INTO new_next_update
        FROM price_feeds
        WHERE id = p_feed_id;
    ELSE
        new_next_update := p_next_update;
    END IF;
    
    -- Update schedule
    UPDATE feed_schedules SET
        last_update = NOW(),
        next_update = new_next_update,
        update_count = update_count + 1,
        consecutive_failures = CASE 
            WHEN p_success THEN 0 
            ELSE consecutive_failures + 1 
        END,
        is_paused = CASE 
            WHEN p_success THEN false
            WHEN consecutive_failures + 1 >= 5 THEN true -- Pause after 5 consecutive failures
            ELSE is_paused
        END,
        pause_reason = CASE 
            WHEN p_success THEN NULL
            WHEN consecutive_failures + 1 >= 5 THEN 'Too many consecutive failures'
            ELSE pause_reason
        END,
        updated_at = NOW()
    WHERE feed_id = p_feed_id;
END;
$$ LANGUAGE plpgsql;

-- Create view for active feed summary
CREATE OR REPLACE VIEW active_feeds_summary AS
SELECT 
    pf.id,
    pf.asset_id,
    pf.name,
    pf.currency,
    pf.update_interval,
    pf.aggregation_method,
    fs.next_update,
    fs.last_update,
    fs.consecutive_failures,
    fs.is_paused,
    COALESCE(fqm.quality_score, 0) as latest_quality_score,
    COALESCE(fu.price, 0) as latest_price,
    fu.created_at as latest_price_time
FROM price_feeds pf
LEFT JOIN feed_schedules fs ON pf.id = fs.feed_id
LEFT JOIN LATERAL (
    SELECT quality_score
    FROM feed_quality_metrics
    WHERE feed_id = pf.id
    ORDER BY time_period DESC
    LIMIT 1
) fqm ON true
LEFT JOIN LATERAL (
    SELECT price, created_at
    FROM feed_updates
    WHERE feed_id = pf.id AND status = 'success'
    ORDER BY created_at DESC
    LIMIT 1
) fu ON true
WHERE pf.is_active = true;

-- Initialize feed schedules for existing feeds
INSERT INTO feed_schedules (feed_id, next_update)
SELECT 
    id,
    NOW() + (update_interval || ' seconds')::INTERVAL
FROM price_feeds
WHERE is_active = true
ON CONFLICT (feed_id) DO NOTHING;
