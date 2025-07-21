#!/bin/bash

# Kill processes on port 3000
echo "🔍 Checking for processes on port 3000..."

# Find and kill processes on port 3000
PIDS=$(lsof -ti:3000)

if [ -n "$PIDS" ]; then
    echo "📍 Found processes on port 3000: $PIDS"
    echo "🔪 Killing processes..."
    echo $PIDS | xargs kill -9
    echo "✅ Processes killed successfully"
else
    echo "✅ No processes found on port 3000"
fi

# Wait a moment for cleanup
sleep 2

echo "🚀 Port 3000 is now available"
