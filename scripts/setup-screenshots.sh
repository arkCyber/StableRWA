#!/bin/bash

# StableRWA Screenshot Setup Script
# This script helps set up the correct screenshot files for the platform

set -e

echo "🖼️ StableRWA Screenshot Setup"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root directory
PROJECT_ROOT="/Users/arksong/rwa-platform"
SCREENSHOTS_DIR="$PROJECT_ROOT/assets/screenshots"

echo -e "${BLUE}📁 Checking project structure...${NC}"

# Ensure we're in the right directory
if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Not in StableRWA project directory${NC}"
    echo "Please run this script from the project root: $PROJECT_ROOT"
    exit 1
fi

# Create screenshots directory if it doesn't exist
if [ ! -d "$SCREENSHOTS_DIR" ]; then
    echo -e "${YELLOW}📁 Creating screenshots directory...${NC}"
    mkdir -p "$SCREENSHOTS_DIR"
    echo -e "${GREEN}✅ Created: $SCREENSHOTS_DIR${NC}"
fi

echo -e "${BLUE}🔍 Checking for screenshot files...${NC}"

# Check for required screenshots
DASHBOARD_FILE="$SCREENSHOTS_DIR/dashboard-main.png"
AI_SERVICES_FILE="$SCREENSHOTS_DIR/ai-services.png"

missing_files=0

if [ ! -f "$DASHBOARD_FILE" ]; then
    echo -e "${RED}❌ Missing: dashboard-main.png${NC}"
    missing_files=$((missing_files + 1))
else
    # Check if it's a real image file
    file_type=$(file "$DASHBOARD_FILE" 2>/dev/null | grep -o "PNG image data" || echo "")
    if [ -n "$file_type" ]; then
        echo -e "${GREEN}✅ Found valid PNG: dashboard-main.png${NC}"
    else
        echo -e "${YELLOW}⚠️  Found file but not a valid PNG: dashboard-main.png${NC}"
        missing_files=$((missing_files + 1))
    fi
fi

if [ ! -f "$AI_SERVICES_FILE" ]; then
    echo -e "${RED}❌ Missing: ai-services.png${NC}"
    missing_files=$((missing_files + 1))
else
    # Check if it's a real image file
    file_type=$(file "$AI_SERVICES_FILE" 2>/dev/null | grep -o "PNG image data" || echo "")
    if [ -n "$file_type" ]; then
        echo -e "${GREEN}✅ Found valid PNG: ai-services.png${NC}"
    else
        echo -e "${YELLOW}⚠️  Found file but not a valid PNG: ai-services.png${NC}"
        missing_files=$((missing_files + 1))
    fi
fi

if [ $missing_files -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}📋 SETUP INSTRUCTIONS${NC}"
    echo "================================"
    echo ""
    echo "You need to add the following screenshot files:"
    echo ""
    echo -e "${BLUE}1. Dashboard Screenshot${NC}"
    echo "   File: $DASHBOARD_FILE"
    echo "   Content: Main dashboard with asset metrics, charts, navigation"
    echo "   Requirements: PNG format, min 1920x1080, clear text"
    echo ""
    echo -e "${BLUE}2. AI Services Screenshot${NC}"
    echo "   File: $AI_SERVICES_FILE"
    echo "   Content: AI services page with service cards and metrics"
    echo "   Requirements: PNG format, min 1920x1080, clear text"
    echo ""
    echo -e "${YELLOW}📝 How to add screenshots:${NC}"
    echo ""
    echo "Option 1 - Copy from your files:"
    echo "  cp /path/to/your/dashboard-screenshot.png \"$DASHBOARD_FILE\""
    echo "  cp /path/to/your/ai-services-screenshot.png \"$AI_SERVICES_FILE\""
    echo ""
    echo "Option 2 - Use drag and drop:"
    echo "  1. Open Finder and navigate to: $SCREENSHOTS_DIR"
    echo "  2. Drag your screenshot files into the folder"
    echo "  3. Rename them to: dashboard-main.png and ai-services.png"
    echo ""
    echo -e "${BLUE}🔄 After adding files, run this script again to verify${NC}"
    echo ""
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 All screenshot files are ready!${NC}"
echo ""

# Check git status
echo -e "${BLUE}📊 Checking Git status...${NC}"
cd "$PROJECT_ROOT"

if git diff --quiet && git diff --staged --quiet; then
    echo -e "${GREEN}✅ No changes to commit${NC}"
else
    echo -e "${YELLOW}📝 Found changes to commit${NC}"
    echo ""
    echo "Would you like to commit and push the changes? (y/n)"
    read -r response
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}📤 Adding files to Git...${NC}"
        git add assets/screenshots/
        
        echo -e "${BLUE}💾 Committing changes...${NC}"
        git commit -m "feat: add platform screenshots

✨ Added screenshots:
- Dashboard main interface with asset metrics
- AI services interface with service cards
- Professional PNG format, high resolution
- Ready for GitHub display"
        
        echo -e "${BLUE}🚀 Pushing to GitHub...${NC}"
        git push origin main
        
        echo -e "${GREEN}🎉 Successfully pushed to GitHub!${NC}"
        echo ""
        echo "🔗 Check your repository: https://github.com/arkCyber/StableRWA"
    else
        echo -e "${YELLOW}⏸️  Skipped commit and push${NC}"
        echo "You can manually commit later with:"
        echo "  git add assets/screenshots/"
        echo "  git commit -m \"feat: add platform screenshots\""
        echo "  git push origin main"
    fi
fi

echo ""
echo -e "${GREEN}✅ Screenshot setup complete!${NC}"
echo ""
echo -e "${BLUE}📋 Summary:${NC}"
echo "  • Screenshots directory: $SCREENSHOTS_DIR"
echo "  • Dashboard screenshot: $([ -f "$DASHBOARD_FILE" ] && echo "✅ Ready" || echo "❌ Missing")"
echo "  • AI services screenshot: $([ -f "$AI_SERVICES_FILE" ] && echo "✅ Ready" || echo "❌ Missing")"
echo ""
echo -e "${BLUE}🔗 Next steps:${NC}"
echo "  1. Verify screenshots display correctly on GitHub"
echo "  2. Update README if needed"
echo "  3. Share the updated repository link"
echo ""
echo "🌟 StableRWA Platform is ready to showcase!"
