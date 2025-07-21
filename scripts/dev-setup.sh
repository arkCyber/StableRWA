#!/bin/bash

# StableRWA Development Setup Script
# This script sets up the development environment for StableRWA

set -e

echo "🚀 StableRWA Development Setup"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if required tools are installed
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    else
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        print_status "Rust $RUST_VERSION is installed"
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed. Please install Node.js from https://nodejs.org/"
        exit 1
    else
        NODE_VERSION=$(node --version)
        print_status "Node.js $NODE_VERSION is installed"
    fi
    
    # Check PostgreSQL
    if ! command -v psql &> /dev/null; then
        print_warning "PostgreSQL is not installed. You may need to install it for full functionality."
        print_warning "Install from: https://www.postgresql.org/download/"
    else
        print_status "PostgreSQL is installed"
    fi
    
    # Check Docker (optional)
    if command -v docker &> /dev/null; then
        print_status "Docker is available"
    else
        print_warning "Docker is not installed. Docker setup will be skipped."
    fi
}

# Setup environment variables
setup_environment() {
    print_step "Setting up environment variables..."
    
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            cp .env.example .env
            print_status "Created .env file from .env.example"
        else
            # Create a basic .env file
            cat > .env << EOF
# Database Configuration
DATABASE_URL=postgresql://stablerwa:password@localhost:5432/stablerwa_dev
REDIS_URL=redis://localhost:6379

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION=24h

# API Configuration
API_HOST=0.0.0.0
API_PORT=8080

# Frontend Configuration
NEXT_PUBLIC_API_URL=http://localhost:8080

# AI Configuration (Optional)
OPENAI_API_KEY=your-openai-api-key-here

# Payment Configuration (Optional)
STRIPE_SECRET_KEY=your-stripe-secret-key
STRIPE_PUBLISHABLE_KEY=your-stripe-publishable-key

# Environment
ENVIRONMENT=development
LOG_LEVEL=debug
EOF
            print_status "Created basic .env file"
            print_warning "Please update the .env file with your actual configuration values"
        fi
    else
        print_status ".env file already exists"
    fi
}

# Setup database
setup_database() {
    print_step "Setting up database..."
    
    if command -v psql &> /dev/null; then
        # Check if database exists
        if psql -lqt | cut -d \| -f 1 | grep -qw stablerwa_dev; then
            print_status "Database 'stablerwa_dev' already exists"
        else
            print_status "Creating database 'stablerwa_dev'..."
            createdb stablerwa_dev || print_warning "Failed to create database. You may need to create it manually."
        fi
    else
        print_warning "PostgreSQL not available. Skipping database setup."
        print_warning "You can use Docker to run PostgreSQL: docker run -d -p 5432:5432 -e POSTGRES_DB=stablerwa_dev -e POSTGRES_USER=stablerwa -e POSTGRES_PASSWORD=password postgres:14"
    fi
}

# Build Rust backend
build_backend() {
    print_step "Building Rust backend..."
    
    print_status "Installing Rust dependencies..."
    if cargo build; then
        print_status "Backend build successful"
    else
        print_warning "Backend build failed. Some modules may have compilation issues."
        print_warning "This is expected in the current development phase."
    fi
}

# Setup frontend
setup_frontend() {
    print_step "Setting up frontend..."
    
    cd webui
    
    print_status "Installing Node.js dependencies..."
    if npm install; then
        print_status "Frontend dependencies installed successfully"
    else
        print_error "Failed to install frontend dependencies"
        exit 1
    fi
    
    cd ..
}

# Start development servers
start_dev_servers() {
    print_step "Starting development servers..."
    
    # Start frontend in background
    print_status "Starting frontend development server..."
    cd webui
    npm run dev &
    FRONTEND_PID=$!
    cd ..
    
    print_status "Frontend server started (PID: $FRONTEND_PID)"
    print_status "Frontend available at: http://localhost:3000"
    
    # Note about backend
    print_warning "Backend server is not started due to compilation issues."
    print_warning "Working on resolving these issues in the next update."
    
    # Save PIDs for cleanup
    echo $FRONTEND_PID > .dev_pids
    
    print_status "Development environment is ready!"
    echo ""
    echo "📱 Frontend: http://localhost:3000"
    echo "🔧 Backend: Not running (compilation issues)"
    echo ""
    echo "To stop the development servers, run: ./scripts/dev-stop.sh"
}

# Docker setup (optional)
setup_docker() {
    if command -v docker &> /dev/null && command -v docker-compose &> /dev/null; then
        print_step "Docker setup available..."
        echo "To use Docker instead of local setup:"
        echo "  docker-compose up --build"
        echo ""
    fi
}

# Main execution
main() {
    echo "Starting StableRWA development setup..."
    echo ""
    
    check_prerequisites
    setup_environment
    setup_database
    build_backend
    setup_frontend
    setup_docker
    
    echo ""
    read -p "Do you want to start the development servers now? (y/N): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        start_dev_servers
    else
        print_status "Setup complete! Run './scripts/dev-start.sh' to start the servers."
    fi
}

# Run main function
main "$@"
