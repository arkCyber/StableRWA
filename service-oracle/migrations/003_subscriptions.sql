-- =====================================================================================
-- RWA Tokenization Platform - Oracle Service Subscriptions Schema
-- 
-- Author: arkSong (arksong2018@gmail.com)
-- =====================================================================================

-- Create subscriptions table for managing price feed subscriptions
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    feed_id UUID NOT NULL REFERENCES price_feeds(id) ON DELETE CASCADE,
    subscriber_id VARCHAR(200) NOT NULL,
    subscriber_type VARCHAR(50) NOT NULL DEFAULT 'external', -- 'external', 'internal', 'webhook'
    webhook_url VARCHAR(500),
    notification_method VARCHAR(50) NOT NULL DEFAULT 'webhook', -- 'webhook', 'websocket', 'sse', 'email'
    filters JSONB, -- price thresholds, change percentages, etc.
    retry_config JSONB, -- retry attempts, backoff strategy, etc.
    is_active BOOLEAN DEFAULT true,
    last_notification TIMESTAMP WITH TIME ZONE,
    notification_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT subscriber_id_not_empty CHECK (LENGTH(TRIM(subscriber_id)) > 0),
    CONSTRAINT notification_method_valid CHECK (
        notification_method IN ('webhook', 'websocket', 'sse', 'email', 'sms')
    ),
    CONSTRAINT subscriber_type_valid CHECK (
        subscriber_type IN ('external', 'internal', 'webhook', 'service')
    ),
    CONSTRAINT notification_count_non_negative CHECK (notification_count >= 0),
    CONSTRAINT failure_count_non_negative CHECK (failure_count >= 0),
    CONSTRAINT webhook_url_required CHECK (
        (notification_method != 'webhook') OR (webhook_url IS NOT NULL AND LENGTH(TRIM(webhook_url)) > 0)
    )
);

-- Create indexes for subscriptions
CREATE INDEX IF NOT EXISTS idx_subscriptions_feed ON subscriptions(feed_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_subscriber ON subscriptions(subscriber_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_active ON subscriptions(is_active);
CREATE INDEX IF NOT EXISTS idx_subscriptions_method ON subscriptions(notification_method);
CREATE INDEX IF NOT EXISTS idx_subscriptions_type ON subscriptions(subscriber_type);

-- Create GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_subscriptions_filters ON subscriptions USING GIN (filters);
CREATE INDEX IF NOT EXISTS idx_subscriptions_retry_config ON subscriptions USING GIN (retry_config);

-- Create composite index for active subscriptions by feed
CREATE INDEX IF NOT EXISTS idx_subscriptions_feed_active 
    ON subscriptions(feed_id, is_active) WHERE is_active = true;

-- Create notification_queue table for managing notification delivery
CREATE TABLE IF NOT EXISTS notification_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    feed_id UUID NOT NULL REFERENCES price_feeds(id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL, -- 'price_update', 'threshold_breach', 'feed_status'
    payload JSONB NOT NULL,
    priority INTEGER DEFAULT 5, -- 1 (highest) to 10 (lowest)
    max_retries INTEGER DEFAULT 3,
    retry_count INTEGER DEFAULT 0,
    retry_after TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'processing', 'sent', 'failed', 'cancelled'
    error_message TEXT,
    sent_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT priority_valid CHECK (priority >= 1 AND priority <= 10),
    CONSTRAINT max_retries_non_negative CHECK (max_retries >= 0),
    CONSTRAINT retry_count_non_negative CHECK (retry_count >= 0),
    CONSTRAINT retry_count_within_max CHECK (retry_count <= max_retries),
    CONSTRAINT status_valid_queue CHECK (
        status IN ('pending', 'processing', 'sent', 'failed', 'cancelled')
    ),
    CONSTRAINT notification_type_valid CHECK (
        notification_type IN ('price_update', 'threshold_breach', 'feed_status', 'system_alert')
    )
);

-- Create indexes for notification queue
CREATE INDEX IF NOT EXISTS idx_notification_queue_status 
    ON notification_queue(status, priority, created_at);

CREATE INDEX IF NOT EXISTS idx_notification_queue_subscription 
    ON notification_queue(subscription_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notification_queue_retry 
    ON notification_queue(retry_after) 
    WHERE status = 'pending' AND retry_after IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_notification_queue_processing 
    ON notification_queue(created_at) 
    WHERE status IN ('pending', 'processing');

-- Create notification_history table for tracking sent notifications
CREATE TABLE IF NOT EXISTS notification_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    notification_queue_id UUID REFERENCES notification_queue(id) ON DELETE SET NULL,
    notification_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    delivery_method VARCHAR(50) NOT NULL,
    delivery_endpoint VARCHAR(500),
    response_status INTEGER,
    response_body TEXT,
    response_time_ms INTEGER,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    sent_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT response_status_valid CHECK (response_status IS NULL OR (response_status >= 100 AND response_status < 600)),
    CONSTRAINT response_time_positive CHECK (response_time_ms IS NULL OR response_time_ms >= 0)
);

-- Create indexes for notification history
CREATE INDEX IF NOT EXISTS idx_notification_history_subscription_time 
    ON notification_history(subscription_id, sent_at DESC);

CREATE INDEX IF NOT EXISTS idx_notification_history_success 
    ON notification_history(success, sent_at DESC);

CREATE INDEX IF NOT EXISTS idx_notification_history_method 
    ON notification_history(delivery_method, sent_at DESC);

-- Create subscription_metrics table for tracking subscription performance
CREATE TABLE IF NOT EXISTS subscription_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    time_period TIMESTAMP WITH TIME ZONE NOT NULL, -- hourly buckets
    notifications_sent INTEGER DEFAULT 0,
    notifications_failed INTEGER DEFAULT 0,
    avg_response_time_ms INTEGER,
    success_rate DECIMAL(5,2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT notifications_sent_non_negative CHECK (notifications_sent >= 0),
    CONSTRAINT notifications_failed_non_negative CHECK (notifications_failed >= 0),
    CONSTRAINT avg_response_time_positive CHECK (avg_response_time_ms IS NULL OR avg_response_time_ms >= 0),
    CONSTRAINT success_rate_valid CHECK (success_rate IS NULL OR (success_rate >= 0 AND success_rate <= 100))
);

-- Create unique index for subscription metrics (one record per subscription per hour)
CREATE UNIQUE INDEX IF NOT EXISTS idx_subscription_metrics_unique 
    ON subscription_metrics(subscription_id, time_period);

-- Create webhook_endpoints table for managing webhook configurations
CREATE TABLE IF NOT EXISTS webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscriber_id VARCHAR(200) NOT NULL,
    url VARCHAR(500) NOT NULL,
    secret_key VARCHAR(255), -- for HMAC signature verification
    headers JSONB, -- custom headers to include
    timeout_seconds INTEGER DEFAULT 30,
    is_active BOOLEAN DEFAULT true,
    last_success TIMESTAMP WITH TIME ZONE,
    last_failure TIMESTAMP WITH TIME ZONE,
    consecutive_failures INTEGER DEFAULT 0,
    total_requests INTEGER DEFAULT 0,
    total_failures INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT url_not_empty CHECK (LENGTH(TRIM(url)) > 0),
    CONSTRAINT timeout_positive CHECK (timeout_seconds > 0),
    CONSTRAINT consecutive_failures_non_negative CHECK (consecutive_failures >= 0),
    CONSTRAINT total_requests_non_negative CHECK (total_requests >= 0),
    CONSTRAINT total_failures_non_negative CHECK (total_failures >= 0),
    CONSTRAINT total_failures_within_requests CHECK (total_failures <= total_requests)
);

-- Create indexes for webhook endpoints
CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_subscriber 
    ON webhook_endpoints(subscriber_id);

CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_active 
    ON webhook_endpoints(is_active);

CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_failures 
    ON webhook_endpoints(consecutive_failures DESC) 
    WHERE is_active = true;

-- Create triggers for automatic updated_at timestamps
CREATE TRIGGER update_subscriptions_updated_at 
    BEFORE UPDATE ON subscriptions 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_notification_queue_updated_at 
    BEFORE UPDATE ON notification_queue 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_webhook_endpoints_updated_at 
    BEFORE UPDATE ON webhook_endpoints 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to enqueue notification
CREATE OR REPLACE FUNCTION enqueue_notification(
    p_subscription_id UUID,
    p_feed_id UUID,
    p_notification_type VARCHAR,
    p_payload JSONB,
    p_priority INTEGER DEFAULT 5
) RETURNS UUID AS $$
DECLARE
    notification_id UUID;
BEGIN
    INSERT INTO notification_queue (
        subscription_id,
        feed_id,
        notification_type,
        payload,
        priority
    ) VALUES (
        p_subscription_id,
        p_feed_id,
        p_notification_type,
        p_payload,
        p_priority
    ) RETURNING id INTO notification_id;
    
    RETURN notification_id;
END;
$$ LANGUAGE plpgsql;

-- Create function to get next notifications to process
CREATE OR REPLACE FUNCTION get_next_notifications(limit_count INTEGER DEFAULT 10)
RETURNS TABLE(
    id UUID,
    subscription_id UUID,
    feed_id UUID,
    notification_type VARCHAR,
    payload JSONB,
    webhook_url VARCHAR,
    retry_count INTEGER,
    max_retries INTEGER
) AS $$
BEGIN
    RETURN QUERY
    UPDATE notification_queue nq
    SET 
        status = 'processing',
        updated_at = NOW()
    FROM (
        SELECT nq_inner.id
        FROM notification_queue nq_inner
        JOIN subscriptions s ON nq_inner.subscription_id = s.id
        WHERE nq_inner.status = 'pending'
        AND s.is_active = true
        AND (nq_inner.retry_after IS NULL OR nq_inner.retry_after <= NOW())
        ORDER BY nq_inner.priority ASC, nq_inner.created_at ASC
        LIMIT limit_count
        FOR UPDATE SKIP LOCKED
    ) selected ON nq.id = selected.id
    RETURNING 
        nq.id,
        nq.subscription_id,
        nq.feed_id,
        nq.notification_type,
        nq.payload,
        (SELECT webhook_url FROM subscriptions WHERE id = nq.subscription_id),
        nq.retry_count,
        nq.max_retries;
END;
$$ LANGUAGE plpgsql;

-- Create function to mark notification as sent or failed
CREATE OR REPLACE FUNCTION complete_notification(
    p_notification_id UUID,
    p_success BOOLEAN,
    p_response_status INTEGER DEFAULT NULL,
    p_response_body TEXT DEFAULT NULL,
    p_response_time_ms INTEGER DEFAULT NULL,
    p_error_message TEXT DEFAULT NULL
) RETURNS VOID AS $$
DECLARE
    notification_record RECORD;
    next_retry_time TIMESTAMP WITH TIME ZONE;
BEGIN
    -- Get notification details
    SELECT * INTO notification_record
    FROM notification_queue
    WHERE id = p_notification_id;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Notification not found: %', p_notification_id;
    END IF;
    
    -- Insert into history
    INSERT INTO notification_history (
        subscription_id,
        notification_queue_id,
        notification_type,
        payload,
        delivery_method,
        delivery_endpoint,
        response_status,
        response_body,
        response_time_ms,
        success,
        error_message
    ) VALUES (
        notification_record.subscription_id,
        p_notification_id,
        notification_record.notification_type,
        notification_record.payload,
        (SELECT notification_method FROM subscriptions WHERE id = notification_record.subscription_id),
        (SELECT webhook_url FROM subscriptions WHERE id = notification_record.subscription_id),
        p_response_status,
        p_response_body,
        p_response_time_ms,
        p_success,
        p_error_message
    );
    
    IF p_success THEN
        -- Mark as sent
        UPDATE notification_queue SET
            status = 'sent',
            sent_at = NOW(),
            updated_at = NOW()
        WHERE id = p_notification_id;
        
        -- Update subscription stats
        UPDATE subscriptions SET
            last_notification = NOW(),
            notification_count = notification_count + 1,
            failure_count = CASE WHEN failure_count > 0 THEN failure_count - 1 ELSE 0 END
        WHERE id = notification_record.subscription_id;
        
    ELSE
        -- Handle failure
        IF notification_record.retry_count < notification_record.max_retries THEN
            -- Schedule retry with exponential backoff
            next_retry_time := NOW() + (POWER(2, notification_record.retry_count + 1) || ' minutes')::INTERVAL;
            
            UPDATE notification_queue SET
                retry_count = retry_count + 1,
                retry_after = next_retry_time,
                status = 'pending',
                error_message = p_error_message,
                updated_at = NOW()
            WHERE id = p_notification_id;
        ELSE
            -- Max retries reached, mark as failed
            UPDATE notification_queue SET
                status = 'failed',
                error_message = p_error_message,
                updated_at = NOW()
            WHERE id = p_notification_id;
        END IF;
        
        -- Update subscription failure count
        UPDATE subscriptions SET
            failure_count = failure_count + 1
        WHERE id = notification_record.subscription_id;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Create function to update subscription metrics
CREATE OR REPLACE FUNCTION update_subscription_metrics()
RETURNS VOID AS $$
DECLARE
    current_hour TIMESTAMP WITH TIME ZONE;
    subscription_record RECORD;
BEGIN
    current_hour := DATE_TRUNC('hour', NOW());
    
    -- Update metrics for each subscription
    FOR subscription_record IN 
        SELECT DISTINCT subscription_id FROM notification_history 
        WHERE sent_at >= current_hour AND sent_at < current_hour + INTERVAL '1 hour'
    LOOP
        INSERT INTO subscription_metrics (
            subscription_id,
            time_period,
            notifications_sent,
            notifications_failed,
            avg_response_time_ms,
            success_rate
        )
        SELECT 
            subscription_record.subscription_id,
            current_hour,
            COUNT(*),
            COUNT(*) FILTER (WHERE NOT success),
            AVG(response_time_ms),
            (COUNT(*) FILTER (WHERE success)::DECIMAL / COUNT(*) * 100)
        FROM notification_history
        WHERE subscription_id = subscription_record.subscription_id
        AND sent_at >= current_hour
        AND sent_at < current_hour + INTERVAL '1 hour'
        ON CONFLICT (subscription_id, time_period) DO UPDATE SET
            notifications_sent = EXCLUDED.notifications_sent,
            notifications_failed = EXCLUDED.notifications_failed,
            avg_response_time_ms = EXCLUDED.avg_response_time_ms,
            success_rate = EXCLUDED.success_rate;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Create view for subscription status summary
CREATE OR REPLACE VIEW subscription_status_summary AS
SELECT 
    s.id,
    s.subscriber_id,
    s.notification_method,
    s.is_active,
    s.notification_count,
    s.failure_count,
    s.last_notification,
    pf.asset_id,
    pf.name as feed_name,
    CASE 
        WHEN s.failure_count = 0 THEN 'healthy'
        WHEN s.failure_count < 5 THEN 'warning'
        ELSE 'critical'
    END as health_status,
    COALESCE(sm.success_rate, 0) as recent_success_rate
FROM subscriptions s
JOIN price_feeds pf ON s.feed_id = pf.id
LEFT JOIN LATERAL (
    SELECT success_rate
    FROM subscription_metrics
    WHERE subscription_id = s.id
    ORDER BY time_period DESC
    LIMIT 1
) sm ON true;

-- Create view for notification queue status
CREATE OR REPLACE VIEW notification_queue_status AS
SELECT 
    status,
    COUNT(*) as count,
    MIN(created_at) as oldest_notification,
    MAX(created_at) as newest_notification,
    AVG(EXTRACT(EPOCH FROM (NOW() - created_at))) as avg_age_seconds
FROM notification_queue
GROUP BY status;

-- Create cleanup function for old notifications
CREATE OR REPLACE FUNCTION cleanup_old_notifications(retention_days INTEGER DEFAULT 7)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
    cutoff_date TIMESTAMP WITH TIME ZONE;
BEGIN
    cutoff_date := NOW() - (retention_days || ' days')::INTERVAL;
    
    -- Clean up old notification history
    WITH deleted AS (
        DELETE FROM notification_history 
        WHERE sent_at < cutoff_date
        RETURNING id
    )
    SELECT COUNT(*) INTO deleted_count FROM deleted;
    
    -- Clean up old completed/failed notifications from queue
    DELETE FROM notification_queue 
    WHERE status IN ('sent', 'failed', 'cancelled')
    AND updated_at < cutoff_date;
    
    -- Clean up old subscription metrics
    DELETE FROM subscription_metrics 
    WHERE time_period < cutoff_date;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Insert default webhook endpoint configurations
INSERT INTO webhook_endpoints (subscriber_id, url, timeout_seconds)
VALUES 
    ('system', 'http://localhost:8080/webhooks/system', 30),
    ('monitoring', 'http://localhost:9090/webhooks/alerts', 15)
ON CONFLICT DO NOTHING;
