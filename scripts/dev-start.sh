#!/bin/bash

# StableRWA Development Start Script
# This script starts the development servers

set -e

echo "🚀 Starting StableRWA Development Servers"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if already running
check_running() {
    if [ -f .dev_pids ]; then
        print_warning "Development servers may already be running"
        print_warning "Run './scripts/dev-stop.sh' first to stop them"
        
        read -p "Do you want to stop existing servers and restart? (y/N): " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            ./scripts/dev-stop.sh
        else
            exit 1
        fi
    fi
}

# Start database services
start_database() {
    print_step "Checking database services..."
    
    # Check if PostgreSQL is running
    if command -v pg_isready &> /dev/null; then
        if pg_isready -q; then
            print_status "PostgreSQL is running"
        else
            print_warning "PostgreSQL is not running. Please start it manually or use Docker."
        fi
    else
        print_warning "PostgreSQL not found. Using Docker or external database."
    fi
    
    # Check if Redis is running (optional)
    if command -v redis-cli &> /dev/null; then
        if redis-cli ping &> /dev/null; then
            print_status "Redis is running"
        else
            print_warning "Redis is not running. Some features may be limited."
        fi
    fi
}

# Start backend server
start_backend() {
    print_step "Starting backend server..."
    
    # Check if backend can be built
    if cargo check --quiet 2>/dev/null; then
        print_status "Starting Rust backend server..."
        cargo run --bin api-server &
        BACKEND_PID=$!
        echo $BACKEND_PID >> .dev_pids
        print_status "Backend server started (PID: $BACKEND_PID) on http://localhost:8080"
    else
        print_warning "Backend has compilation issues. Skipping backend startup."
        print_warning "Frontend will use mock API data."
    fi
}

# Start frontend server
start_frontend() {
    print_step "Starting frontend server..."
    
    if [ ! -d "webui/node_modules" ]; then
        print_status "Installing frontend dependencies..."
        cd webui && npm install && cd ..
    fi
    
    print_status "Starting Next.js development server..."
    cd webui
    npm run dev &
    FRONTEND_PID=$!
    cd ..
    
    echo $FRONTEND_PID >> .dev_pids
    print_status "Frontend server started (PID: $FRONTEND_PID) on http://localhost:3000"
}

# Wait for servers to be ready
wait_for_servers() {
    print_step "Waiting for servers to be ready..."
    
    # Wait for frontend
    print_status "Waiting for frontend server..."
    for i in {1..30}; do
        if curl -s http://localhost:3000 > /dev/null 2>&1; then
            print_status "Frontend server is ready!"
            break
        fi
        sleep 1
    done
    
    # Check backend if it was started
    if grep -q "api-server" .dev_pids 2>/dev/null; then
        print_status "Waiting for backend server..."
        for i in {1..30}; do
            if curl -s http://localhost:8080/health > /dev/null 2>&1; then
                print_status "Backend server is ready!"
                break
            fi
            sleep 1
        done
    fi
}

# Display server information
show_info() {
    echo ""
    echo "🎉 Development servers are running!"
    echo "=================================="
    echo ""
    echo "📱 Frontend Application:"
    echo "   URL: http://localhost:3000"
    echo "   Framework: Next.js 14"
    echo ""
    
    if grep -q "api-server" .dev_pids 2>/dev/null; then
        echo "🔧 Backend API:"
        echo "   URL: http://localhost:8080"
        echo "   Framework: Rust + Axum"
        echo ""
    else
        echo "🔧 Backend API:"
        echo "   Status: Not running (using mock data)"
        echo "   Note: Compilation issues being resolved"
        echo ""
    fi
    
    echo "📊 Available Pages:"
    echo "   • Dashboard: http://localhost:3000"
    echo "   • Assets: http://localhost:3000/assets"
    echo "   • Trading: http://localhost:3000/trading"
    echo "   • Analytics: http://localhost:3000/analytics"
    echo "   • Compliance: http://localhost:3000/compliance"
    echo ""
    
    echo "🛠️ Development Tools:"
    echo "   • Stop servers: ./scripts/dev-stop.sh"
    echo "   • View logs: tail -f .dev_pids"
    echo "   • Database: psql stablerwa_dev"
    echo ""
    
    echo "📚 Documentation:"
    echo "   • README: README.md (中文) / README_EN.md (English)"
    echo "   • Project Status: PROJECT_STATUS.md"
    echo "   • API Docs: http://localhost:3000/api (when backend is running)"
    echo ""
}

# Handle cleanup on exit
cleanup() {
    print_status "Shutting down development servers..."
    if [ -f .dev_pids ]; then
        while IFS= read -r pid; do
            if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
                kill "$pid"
            fi
        done < .dev_pids
        rm .dev_pids
    fi
}

# Set up signal handlers
trap cleanup EXIT INT TERM

# Main execution
main() {
    check_running
    start_database
    start_backend
    start_frontend
    wait_for_servers
    show_info
    
    print_status "Press Ctrl+C to stop all servers"
    
    # Keep script running
    while true; do
        sleep 1
    done
}

# Run main function
main "$@"
