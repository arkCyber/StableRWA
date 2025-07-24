#!/bin/sh

# =====================================================================================
# File: webui/docker-entrypoint.sh
# Description: Docker entrypoint script for StableRWA WebUI
# Author: arkSong (arksong2018@gmail.com)
# Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
# =====================================================================================

set -e

echo "🚀 Starting StableRWA WebUI..."

# Function to wait for backend services
wait_for_services() {
    echo "⏳ Waiting for backend services to be ready..."
    
    # Extract host and port from URLs
    GATEWAY_HOST=$(echo "${NEXT_PUBLIC_GATEWAY_URL:-http://localhost:8080}" | sed 's|http://||' | cut -d: -f1)
    GATEWAY_PORT=$(echo "${NEXT_PUBLIC_GATEWAY_URL:-http://localhost:8080}" | sed 's|http://||' | cut -d: -f2 | cut -d/ -f1)
    
    # Wait for gateway service
    timeout=60
    while [ $timeout -gt 0 ]; do
        if nc -z "$GATEWAY_HOST" "$GATEWAY_PORT" 2>/dev/null; then
            echo "✅ Gateway service is ready"
            break
        fi
        echo "⏳ Waiting for gateway service... ($timeout seconds remaining)"
        sleep 2
        timeout=$((timeout - 2))
    done
    
    if [ $timeout -le 0 ]; then
        echo "⚠️ Gateway service not ready, continuing anyway..."
    fi
}

# Function to create runtime configuration
create_runtime_config() {
    echo "📝 Creating runtime configuration..."
    
    cat > /app/public/config.js << EOF
window.RUNTIME_CONFIG = {
    API_BASE_URL: '${NEXT_PUBLIC_API_BASE_URL:-http://localhost:8080}',
    GATEWAY_URL: '${NEXT_PUBLIC_GATEWAY_URL:-http://localhost:8080}',
    ASSETS_URL: '${NEXT_PUBLIC_ASSETS_URL:-http://localhost:8081}',
    ORACLE_URL: '${NEXT_PUBLIC_ORACLE_URL:-http://localhost:8082}',
    AI_URL: '${NEXT_PUBLIC_AI_URL:-http://localhost:8083}',
    WS_URL: '${NEXT_PUBLIC_WS_URL:-ws://localhost:8080/ws}',
    CHAIN_ID: '${NEXT_PUBLIC_CHAIN_ID:-1337}',
    RPC_URL: '${NEXT_PUBLIC_RPC_URL:-http://localhost:8545}',
    DOCKER_MODE: '${NEXT_PUBLIC_DOCKER_MODE:-true}',
    THEME: '${NEXT_PUBLIC_THEME:-dark}',
    LANGUAGE: '${NEXT_PUBLIC_LANGUAGE:-en}',
    CURRENCY: '${NEXT_PUBLIC_CURRENCY:-USD}',
    ENABLE_AI_FEATURES: '${NEXT_PUBLIC_ENABLE_AI_FEATURES:-true}',
    ENABLE_REAL_TIME_UPDATES: '${NEXT_PUBLIC_ENABLE_REAL_TIME_UPDATES:-true}',
    ENABLE_NOTIFICATIONS: '${NEXT_PUBLIC_ENABLE_NOTIFICATIONS:-true}',
    LOG_LEVEL: '${NEXT_PUBLIC_LOG_LEVEL:-info}',
    VERSION: '${NEXT_PUBLIC_APP_VERSION:-1.0.0}',
    BUILD_DATE: '${NEXT_PUBLIC_BUILD_DATE:-$(date -Iseconds)}'
};
EOF
}

# Function to display startup information
display_startup_info() {
    echo "================================================="
    echo "🚀 StableRWA WebUI Starting"
    echo "================================================="
    echo "Environment: ${NODE_ENV:-production}"
    echo "Version: ${NEXT_PUBLIC_APP_VERSION:-1.0.0}"
    echo "Build Date: ${NEXT_PUBLIC_BUILD_DATE:-unknown}"
    echo "Docker Mode: ${NEXT_PUBLIC_DOCKER_MODE:-true}"
    echo "Gateway URL: ${NEXT_PUBLIC_GATEWAY_URL:-http://localhost:8080}"
    echo "Assets URL: ${NEXT_PUBLIC_ASSETS_URL:-http://localhost:8081}"
    echo "Oracle URL: ${NEXT_PUBLIC_ORACLE_URL:-http://localhost:8082}"
    echo "AI URL: ${NEXT_PUBLIC_AI_URL:-http://localhost:8083}"
    echo "WebSocket URL: ${NEXT_PUBLIC_WS_URL:-ws://localhost:8080/ws}"
    echo "Theme: ${NEXT_PUBLIC_THEME:-dark}"
    echo "Language: ${NEXT_PUBLIC_LANGUAGE:-en}"
    echo "================================================="
}

# Function to setup health check endpoint
setup_health_check() {
    echo "🔍 Setting up health check endpoint..."
    
    # Create health check API route if it doesn't exist
    mkdir -p /app/pages/api
    
    cat > /app/pages/api/health.js << 'EOF'
export default function handler(req, res) {
  if (req.method === 'GET') {
    res.status(200).json({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      version: process.env.NEXT_PUBLIC_APP_VERSION || '1.0.0',
      environment: process.env.NODE_ENV || 'production'
    });
  } else {
    res.setHeader('Allow', ['GET']);
    res.status(405).end(`Method ${req.method} Not Allowed`);
  }
}
EOF
}

# Function to check if netcat is available
check_netcat() {
    if ! command -v nc >/dev/null 2>&1; then
        echo "⚠️ netcat not available, installing..."
        apk add --no-cache netcat-openbsd
    fi
}

# Main execution
main() {
    display_startup_info
    
    # Check for required tools
    check_netcat
    
    # Create runtime configuration
    create_runtime_config
    
    # Setup health check
    setup_health_check
    
    # Only wait for services in Docker mode
    if [ "${NEXT_PUBLIC_DOCKER_MODE:-true}" = "true" ]; then
        wait_for_services
    fi
    
    echo "✅ WebUI initialization complete"
    echo "🌐 Starting Next.js server on port ${PORT:-3000}..."
    
    # Execute the main command
    exec "$@"
}

# Run main function
main "$@"
