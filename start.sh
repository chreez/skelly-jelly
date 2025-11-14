#!/bin/bash
# Skelly-Jelly Single Command Startup Script
# Usage: ./start.sh [mode]
# Modes: ts (default), full, demo, dev

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

MODE="${1:-ts}"

echo "🦴 Starting Skelly-Jelly in $MODE mode..."

case "$MODE" in
    "ts"|"typescript")
        echo "📦 Starting TypeScript modules only (Rust compilation issues)..."
        npm run start:ts-only
        ;;
    "full")
        echo "🔨 Building and starting full application with all services..."
        npm run start:full
        ;;
    "demo")
        echo "🎬 Starting demo mode with simulated components..."
        npm run start:demo
        ;;
    "dev")
        echo "🔧 Starting development mode with hot reload..."
        npm run dev
        ;;
    *)
        echo "❌ Unknown mode: $MODE"
        echo "Available modes: ts, full, demo, dev"
        echo "Usage: ./start.sh [ts|full|demo|dev]"
        echo ""
        echo "  ts   - TypeScript modules only (recommended)"
        echo "  full - Full application with Rust + TypeScript"
        echo "  demo - Demo mode with simulated components"
        echo "  dev  - Development mode with hot reload"
        exit 1
        ;;
esac