#!/bin/bash

# =====================================================================================
# File: scripts/prepare-git-push.sh
# Description: Prepare StableRWA platform for Git push to GitHub
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

set -e

echo "🚀 Preparing StableRWA platform for Git push..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

print_info "Cleaning up temporary and build files..."

# Remove build artifacts
if [ -d "target" ]; then
    print_info "Removing Rust build artifacts..."
    rm -rf target/debug target/release target/doc target/tmp target/rust-analyzer
    # Keep target/CACHEDIR.TAG for faster rebuilds
fi

# Remove Node.js artifacts
if [ -d "node_modules" ]; then
    print_info "Removing Node.js modules..."
    rm -rf node_modules
fi

if [ -f "package-lock.json" ]; then
    rm -f package-lock.json
fi

if [ -d "webui/node_modules" ]; then
    print_info "Removing WebUI Node.js modules..."
    rm -rf webui/node_modules
fi

# Remove coverage reports
if [ -d "coverage" ]; then
    print_info "Removing coverage reports..."
    rm -rf coverage
fi

# Remove temporary test files
find . -name "*.tmp" -type f -delete 2>/dev/null || true
find . -name "*.bak" -type f -delete 2>/dev/null || true
find . -name "*~" -type f -delete 2>/dev/null || true

# Remove OS-specific files
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
find . -name "Thumbs.db" -type f -delete 2>/dev/null || true

print_status "Cleaned up temporary files"

# Verify essential files exist
print_info "Verifying essential files..."

essential_files=(
    "README.md"
    "README_CN.md" 
    "README_EN.md"
    "Cargo.toml"
    "LICENSE"
    "CONTRIBUTING.md"
    "SECURITY.md"
    ".gitignore"
    "docker-compose.yml"
)

for file in "${essential_files[@]}"; do
    if [ -f "$file" ]; then
        print_status "Found $file"
    else
        print_warning "Missing $file"
    fi
done

# Check core modules
print_info "Verifying core modules..."

core_modules=(
    "core-utils"
    "core-security"
    "core-blockchain"
    "core-defi"
    "core-layer2"
    "core-oracle"
    "core-wallet"
    "core-nft"
    "core-smart-contract"
    "core-regtech"
    "core-ai-risk"
    "core-monitoring"
    "core-privacy"
    "core-api-gateway"
)

for module in "${core_modules[@]}"; do
    if [ -d "$module" ] && [ -f "$module/Cargo.toml" ]; then
        print_status "Core module: $module"
    else
        print_warning "Missing core module: $module"
    fi
done

# Validate Cargo.toml
print_info "Validating Cargo workspace..."
if cargo check --workspace --quiet 2>/dev/null; then
    print_status "Cargo workspace is valid"
else
    print_warning "Cargo workspace has issues - please check manually"
fi

# Check for large files
print_info "Checking for large files (>10MB)..."
large_files=$(find . -type f -size +10M 2>/dev/null | grep -v target | grep -v node_modules | head -10)
if [ -n "$large_files" ]; then
    print_warning "Found large files that should be excluded:"
    echo "$large_files"
else
    print_status "No large files found"
fi

# Generate project statistics
print_info "Generating project statistics..."

total_rust_files=$(find . -name "*.rs" -not -path "./target/*" -not -path "./node_modules/*" | wc -l)
total_lines=$(find . -name "*.rs" -not -path "./target/*" -not -path "./node_modules/*" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")
total_modules=$(find . -name "Cargo.toml" -not -path "./target/*" | wc -l)

echo ""
echo "📊 Project Statistics:"
echo "   • Rust files: $total_rust_files"
echo "   • Lines of code: $total_lines"
echo "   • Cargo modules: $total_modules"
echo "   • Core modules: ${#core_modules[@]}"

# Check Git status
print_info "Checking Git status..."

if [ -d ".git" ]; then
    # Check if there are uncommitted changes
    if [ -n "$(git status --porcelain)" ]; then
        print_warning "There are uncommitted changes"
        echo "Files to be committed:"
        git status --short
    else
        print_status "Working directory is clean"
    fi
    
    # Show current branch
    current_branch=$(git branch --show-current)
    print_info "Current branch: $current_branch"
else
    print_warning "Not a Git repository - run 'git init' first"
fi

echo ""
print_status "✨ StableRWA platform is ready for Git push!"
echo ""
print_info "Next steps:"
echo "   1. Review the files to be committed"
echo "   2. Add your actual dashboard screenshot to assets/screenshots/dashboard.png"
echo "   3. Run: git add ."
echo "   4. Run: git commit -m 'feat: complete enterprise Web3 RWA platform'"
echo "   5. Run: git remote add origin https://github.com/arkCyber/StableRWA.git"
echo "   6. Run: git push -u origin main"
echo ""
print_info "🎉 Your world-class Web3 RWA platform is ready to share with the world!"
