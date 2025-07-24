-- =====================================================================================
-- RWA Tokenization Platform - Oracle Service Database Initialization
-- 
-- Author: arkSong (arksong2018@gmail.com)
-- =====================================================================================

-- Create database if it doesn't exist (this runs as postgres user)
SELECT 'CREATE DATABASE oracle_service'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'oracle_service')\gexec

-- Connect to the oracle_service database
\c oracle_service;

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create application user with limited privileges
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'oracle_app') THEN
        CREATE USER oracle_app WITH PASSWORD 'oracle_app_password';
    END IF;
END
$$;

-- Grant necessary privileges to application user
GRANT CONNECT ON DATABASE oracle_service TO oracle_app;
GRANT USAGE ON SCHEMA public TO oracle_app;
GRANT CREATE ON SCHEMA public TO oracle_app;

-- Grant privileges on future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO oracle_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO oracle_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT EXECUTE ON FUNCTIONS TO oracle_app;

-- Create monitoring user for metrics collection
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'oracle_monitor') THEN
        CREATE USER oracle_monitor WITH PASSWORD 'oracle_monitor_password';
    END IF;
END
$$;

-- Grant read-only access to monitoring user
GRANT CONNECT ON DATABASE oracle_service TO oracle_monitor;
GRANT USAGE ON SCHEMA public TO oracle_monitor;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO oracle_monitor;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO oracle_monitor;

-- Create backup user
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'oracle_backup') THEN
        CREATE USER oracle_backup WITH PASSWORD 'oracle_backup_password';
    END IF;
END
$$;

-- Grant backup privileges
GRANT CONNECT ON DATABASE oracle_service TO oracle_backup;
GRANT USAGE ON SCHEMA public TO oracle_backup;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO oracle_backup;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO oracle_backup;

-- Configure database settings for optimal performance
ALTER SYSTEM SET shared_preload_libraries = 'pg_stat_statements';
ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;

-- Configure logging for monitoring
ALTER SYSTEM SET log_destination = 'stderr';
ALTER SYSTEM SET logging_collector = on;
ALTER SYSTEM SET log_directory = 'pg_log';
ALTER SYSTEM SET log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log';
ALTER SYSTEM SET log_rotation_age = '1d';
ALTER SYSTEM SET log_rotation_size = '100MB';
ALTER SYSTEM SET log_min_duration_statement = '1000ms';
ALTER SYSTEM SET log_checkpoints = on;
ALTER SYSTEM SET log_connections = on;
ALTER SYSTEM SET log_disconnections = on;
ALTER SYSTEM SET log_lock_waits = on;
ALTER SYSTEM SET log_temp_files = 0;

-- Configure statement statistics
ALTER SYSTEM SET pg_stat_statements.max = 10000;
ALTER SYSTEM SET pg_stat_statements.track = all;

-- Create tablespaces for better I/O distribution (optional)
-- CREATE TABLESPACE oracle_data LOCATION '/var/lib/postgresql/data/oracle_data';
-- CREATE TABLESPACE oracle_index LOCATION '/var/lib/postgresql/data/oracle_index';

-- Create schema for partitioning (if needed for large datasets)
CREATE SCHEMA IF NOT EXISTS partitions;
GRANT USAGE ON SCHEMA partitions TO oracle_app;
GRANT CREATE ON SCHEMA partitions TO oracle_app;

-- Create function to automatically create monthly partitions for asset_prices
CREATE OR REPLACE FUNCTION create_monthly_partition(table_name TEXT, start_date DATE)
RETURNS VOID AS $$
DECLARE
    partition_name TEXT;
    end_date DATE;
BEGIN
    partition_name := table_name || '_' || to_char(start_date, 'YYYY_MM');
    end_date := start_date + INTERVAL '1 month';
    
    EXECUTE format('CREATE TABLE IF NOT EXISTS partitions.%I PARTITION OF %I 
                    FOR VALUES FROM (%L) TO (%L)',
                   partition_name, table_name, start_date, end_date);
    
    -- Create indexes on partition
    EXECUTE format('CREATE INDEX IF NOT EXISTS %I ON partitions.%I (asset_id, timestamp DESC)',
                   partition_name || '_asset_timestamp_idx', partition_name);
END;
$$ LANGUAGE plpgsql;

-- Create function for automatic partition maintenance
CREATE OR REPLACE FUNCTION maintain_partitions()
RETURNS VOID AS $$
DECLARE
    current_month DATE;
    next_month DATE;
BEGIN
    current_month := date_trunc('month', CURRENT_DATE);
    next_month := current_month + INTERVAL '1 month';
    
    -- Create partition for current month if it doesn't exist
    PERFORM create_monthly_partition('asset_prices', current_month);
    
    -- Create partition for next month
    PERFORM create_monthly_partition('asset_prices', next_month);
    
    -- Drop partitions older than 12 months
    PERFORM drop_old_partitions('asset_prices', 12);
END;
$$ LANGUAGE plpgsql;

-- Create function to drop old partitions
CREATE OR REPLACE FUNCTION drop_old_partitions(table_name TEXT, months_to_keep INTEGER)
RETURNS VOID AS $$
DECLARE
    cutoff_date DATE;
    partition_record RECORD;
BEGIN
    cutoff_date := date_trunc('month', CURRENT_DATE) - (months_to_keep || ' months')::INTERVAL;
    
    FOR partition_record IN
        SELECT schemaname, tablename
        FROM pg_tables
        WHERE schemaname = 'partitions'
        AND tablename LIKE table_name || '_%'
        AND to_date(substring(tablename from '(\d{4}_\d{2})$'), 'YYYY_MM') < cutoff_date
    LOOP
        EXECUTE format('DROP TABLE IF EXISTS %I.%I', partition_record.schemaname, partition_record.tablename);
        RAISE NOTICE 'Dropped partition: %.%', partition_record.schemaname, partition_record.tablename;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Create monitoring views
CREATE OR REPLACE VIEW database_stats AS
SELECT 
    datname as database_name,
    numbackends as active_connections,
    xact_commit as transactions_committed,
    xact_rollback as transactions_rolled_back,
    blks_read as blocks_read,
    blks_hit as blocks_hit,
    ROUND(blks_hit::numeric / NULLIF(blks_hit + blks_read, 0) * 100, 2) as cache_hit_ratio,
    tup_returned as tuples_returned,
    tup_fetched as tuples_fetched,
    tup_inserted as tuples_inserted,
    tup_updated as tuples_updated,
    tup_deleted as tuples_deleted,
    stats_reset
FROM pg_stat_database
WHERE datname = 'oracle_service';

CREATE OR REPLACE VIEW table_stats AS
SELECT 
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_tuples,
    n_dead_tup as dead_tuples,
    last_vacuum,
    last_autovacuum,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC;

CREATE OR REPLACE VIEW index_usage AS
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_tup_read as index_tuples_read,
    idx_tup_fetch as index_tuples_fetched,
    idx_scan as index_scans
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Create function for database maintenance
CREATE OR REPLACE FUNCTION perform_maintenance()
RETURNS TEXT AS $$
DECLARE
    result TEXT := '';
BEGIN
    -- Update table statistics
    ANALYZE;
    result := result || 'Statistics updated. ';
    
    -- Maintain partitions
    PERFORM maintain_partitions();
    result := result || 'Partitions maintained. ';
    
    -- Vacuum if needed (this will be done automatically by autovacuum)
    -- VACUUM ANALYZE;
    
    RETURN result || 'Maintenance completed at ' || NOW();
END;
$$ LANGUAGE plpgsql;

-- Grant execute permission on maintenance functions
GRANT EXECUTE ON FUNCTION perform_maintenance() TO oracle_app;
GRANT EXECUTE ON FUNCTION maintain_partitions() TO oracle_app;

-- Create initial partitions for current and next month
SELECT maintain_partitions();

-- Insert initial configuration data
INSERT INTO pg_settings (name, setting) VALUES ('application_name', 'oracle_service') ON CONFLICT DO NOTHING;

-- Log successful initialization
DO $$
BEGIN
    RAISE NOTICE 'Oracle Service database initialization completed successfully at %', NOW();
END
$$;
