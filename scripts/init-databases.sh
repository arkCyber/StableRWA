#!/bin/bash

# =====================================================================================
# Database Initialization Script for RWA Platform
# Creates multiple databases for different environments
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

set -e

# Function to create database if it doesn't exist
create_database() {
    local database=$1
    echo "Creating database '$database' if it doesn't exist..."
    
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
        SELECT 'CREATE DATABASE $database'
        WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$database')\gexec
EOSQL
    
    echo "Database '$database' is ready."
}

# Function to create user if it doesn't exist
create_user() {
    local username=$1
    local password=$2
    echo "Creating user '$username' if it doesn't exist..."
    
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
        DO \$\$
        BEGIN
            IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = '$username') THEN
                CREATE USER $username WITH PASSWORD '$password';
            END IF;
        END
        \$\$;
EOSQL
    
    echo "User '$username' is ready."
}

# Function to grant privileges
grant_privileges() {
    local username=$1
    local database=$2
    echo "Granting privileges to '$username' on database '$database'..."
    
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$database" <<-EOSQL
        GRANT ALL PRIVILEGES ON DATABASE $database TO $username;
        GRANT ALL ON SCHEMA public TO $username;
        GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $username;
        GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO $username;
        ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO $username;
        ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO $username;
EOSQL
    
    echo "Privileges granted to '$username' on database '$database'."
}

# Function to create extensions
create_extensions() {
    local database=$1
    echo "Creating extensions for database '$database'..."
    
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$database" <<-EOSQL
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
        CREATE EXTENSION IF NOT EXISTS "pgcrypto";
        CREATE EXTENSION IF NOT EXISTS "pg_trgm";
        CREATE EXTENSION IF NOT EXISTS "btree_gin";
        CREATE EXTENSION IF NOT EXISTS "btree_gist";
EOSQL
    
    echo "Extensions created for database '$database'."
}

# Function to create initial schema
create_initial_schema() {
    local database=$1
    echo "Creating initial schema for database '$database'..."
    
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$database" <<-EOSQL
        -- Create audit log table
        CREATE TABLE IF NOT EXISTS audit_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            table_name VARCHAR(255) NOT NULL,
            operation VARCHAR(10) NOT NULL,
            old_values JSONB,
            new_values JSONB,
            user_id VARCHAR(255),
            timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Create audit trigger function
        CREATE OR REPLACE FUNCTION audit_trigger_function()
        RETURNS TRIGGER AS \$\$
        BEGIN
            IF TG_OP = 'DELETE' THEN
                INSERT INTO audit_log (table_name, operation, old_values, user_id)
                VALUES (TG_TABLE_NAME, TG_OP, row_to_json(OLD), current_setting('app.current_user_id', true));
                RETURN OLD;
            ELSIF TG_OP = 'UPDATE' THEN
                INSERT INTO audit_log (table_name, operation, old_values, new_values, user_id)
                VALUES (TG_TABLE_NAME, TG_OP, row_to_json(OLD), row_to_json(NEW), current_setting('app.current_user_id', true));
                RETURN NEW;
            ELSIF TG_OP = 'INSERT' THEN
                INSERT INTO audit_log (table_name, operation, new_values, user_id)
                VALUES (TG_TABLE_NAME, TG_OP, row_to_json(NEW), current_setting('app.current_user_id', true));
                RETURN NEW;
            END IF;
            RETURN NULL;
        END;
        \$\$ LANGUAGE plpgsql;

        -- Create migration tracking table
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Create application settings table
        CREATE TABLE IF NOT EXISTS app_settings (
            key VARCHAR(255) PRIMARY KEY,
            value JSONB NOT NULL,
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Insert default settings
        INSERT INTO app_settings (key, value, description) VALUES
        ('app_version', '"1.0.0"', 'Application version'),
        ('maintenance_mode', 'false', 'Maintenance mode flag'),
        ('max_file_upload_size', '10485760', 'Maximum file upload size in bytes (10MB)'),
        ('session_timeout', '3600', 'Session timeout in seconds (1 hour)')
        ON CONFLICT (key) DO NOTHING;

        -- Create indexes
        CREATE INDEX IF NOT EXISTS idx_audit_log_table_name ON audit_log(table_name);
        CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
        CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON audit_log(user_id);
EOSQL
    
    echo "Initial schema created for database '$database'."
}

# Main execution
echo "Starting database initialization..."

# Parse POSTGRES_MULTIPLE_DATABASES if set
if [ -n "${POSTGRES_MULTIPLE_DATABASES:-}" ]; then
    echo "Creating multiple databases: $POSTGRES_MULTIPLE_DATABASES"
    
    # Split the comma-separated list
    IFS=',' read -ra DATABASES <<< "$POSTGRES_MULTIPLE_DATABASES"
    
    for db in "${DATABASES[@]}"; do
        # Trim whitespace
        db=$(echo "$db" | xargs)
        
        if [ -n "$db" ]; then
            create_database "$db"
            create_extensions "$db"
            grant_privileges "$POSTGRES_USER" "$db"
            create_initial_schema "$db"
        fi
    done
else
    echo "No additional databases specified in POSTGRES_MULTIPLE_DATABASES"
fi

# Create application-specific user if different from POSTGRES_USER
if [ "${POSTGRES_USER}" != "rwa_user" ]; then
    create_user "rwa_user" "rwa_password"
    
    # Grant privileges to rwa_user on all databases
    if [ -n "${POSTGRES_MULTIPLE_DATABASES:-}" ]; then
        IFS=',' read -ra DATABASES <<< "$POSTGRES_MULTIPLE_DATABASES"
        for db in "${DATABASES[@]}"; do
            db=$(echo "$db" | xargs)
            if [ -n "$db" ]; then
                grant_privileges "rwa_user" "$db"
            fi
        done
    fi
    
    # Grant privileges on main database
    grant_privileges "rwa_user" "$POSTGRES_DB"
fi

# Create read-only user for reporting
create_user "rwa_readonly" "readonly_password"

# Grant read-only privileges
echo "Setting up read-only user..."
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    GRANT CONNECT ON DATABASE $POSTGRES_DB TO rwa_readonly;
    GRANT USAGE ON SCHEMA public TO rwa_readonly;
    GRANT SELECT ON ALL TABLES IN SCHEMA public TO rwa_readonly;
    ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO rwa_readonly;
EOSQL

# Apply read-only privileges to additional databases
if [ -n "${POSTGRES_MULTIPLE_DATABASES:-}" ]; then
    IFS=',' read -ra DATABASES <<< "$POSTGRES_MULTIPLE_DATABASES"
    for db in "${DATABASES[@]}"; do
        db=$(echo "$db" | xargs)
        if [ -n "$db" ]; then
            psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db" <<-EOSQL
                GRANT CONNECT ON DATABASE $db TO rwa_readonly;
                GRANT USAGE ON SCHEMA public TO rwa_readonly;
                GRANT SELECT ON ALL TABLES IN SCHEMA public TO rwa_readonly;
                ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO rwa_readonly;
EOSQL
        fi
    done
fi

# Create backup user
create_user "rwa_backup" "backup_password"

echo "Granting backup privileges..."
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    ALTER USER rwa_backup WITH REPLICATION;
    GRANT CONNECT ON DATABASE $POSTGRES_DB TO rwa_backup;
EOSQL

# Set up connection limits and other security settings
echo "Configuring security settings..."
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Set connection limits
    ALTER USER rwa_user CONNECTION LIMIT 50;
    ALTER USER rwa_readonly CONNECTION LIMIT 10;
    ALTER USER rwa_backup CONNECTION LIMIT 2;
    
    -- Set statement timeout (5 minutes)
    ALTER DATABASE $POSTGRES_DB SET statement_timeout = '300s';
    
    -- Set idle timeout (30 minutes)
    ALTER DATABASE $POSTGRES_DB SET idle_in_transaction_session_timeout = '1800s';
    
    -- Enable logging for slow queries
    ALTER DATABASE $POSTGRES_DB SET log_min_duration_statement = '1000';
EOSQL

echo "Database initialization completed successfully!"

# Display summary
echo ""
echo "=== Database Initialization Summary ==="
echo "Main database: $POSTGRES_DB"
if [ -n "${POSTGRES_MULTIPLE_DATABASES:-}" ]; then
    echo "Additional databases: $POSTGRES_MULTIPLE_DATABASES"
fi
echo "Users created:"
echo "  - $POSTGRES_USER (superuser)"
echo "  - rwa_user (application user)"
echo "  - rwa_readonly (read-only user)"
echo "  - rwa_backup (backup user)"
echo ""
echo "Extensions installed:"
echo "  - uuid-ossp (UUID generation)"
echo "  - pgcrypto (cryptographic functions)"
echo "  - pg_trgm (trigram matching)"
echo "  - btree_gin (GIN indexes)"
echo "  - btree_gist (GiST indexes)"
echo ""
echo "Initial schema includes:"
echo "  - audit_log table with trigger function"
echo "  - schema_migrations table"
echo "  - app_settings table with default values"
echo "=================================="
