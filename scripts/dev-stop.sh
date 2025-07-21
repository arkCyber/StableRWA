#!/bin/bash

# StableRWA Development Stop Script
# This script stops the development servers

set -e

echo "🛑 Stopping StableRWA Development Servers"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Stop development servers
stop_servers() {
    # Check if PID file exists
    if [ -f .dev_pids ]; then
        print_status "Found running development servers..."
        
        # Read PIDs and stop processes
        while IFS= read -r pid; do
            if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
                print_status "Stopping process $pid..."
                kill "$pid"
            fi
        done < .dev_pids
        
        # Remove PID file
        rm .dev_pids
        print_status "Development servers stopped"
    else
        print_warning "No PID file found. Attempting to stop common development processes..."
        
        # Try to stop common Node.js development processes
        pkill -f "next dev" 2>/dev/null && print_status "Stopped Next.js development server" || true
        pkill -f "npm run dev" 2>/dev/null && print_status "Stopped npm development server" || true
        
        # Try to stop Rust development processes
        pkill -f "cargo run" 2>/dev/null && print_status "Stopped Rust development server" || true
        pkill -f "api-server" 2>/dev/null && print_status "Stopped API server" || true
    fi
}

# Stop Docker containers if running
stop_docker() {
    if command -v docker-compose &> /dev/null; then
        if docker-compose ps -q | grep -q .; then
            print_status "Stopping Docker containers..."
            docker-compose down
        fi
    fi
}

# Clean up temporary files
cleanup() {
    print_status "Cleaning up temporary files..."
    
    # Remove Next.js build cache if it exists
    if [ -d "webui/.next" ]; then
        print_status "Removing Next.js cache..."
        rm -rf webui/.next
    fi
    
    # Remove any lock files that might be stuck
    find . -name "*.lock" -type f -delete 2>/dev/null || true
    
    print_status "Cleanup complete"
}

# Main execution
main() {
    stop_servers
    stop_docker
    
    echo ""
    read -p "Do you want to clean up temporary files? (y/N): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cleanup
    fi
    
    print_status "All development servers have been stopped"
}

# Run main function
main "$@"
