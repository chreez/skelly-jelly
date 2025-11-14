# Skelly-Jelly Makefile
# Simple commands to run the ADHD companion app

.PHONY: run start full demo dev build test clean help

# Default target
run: start

# Main startup targets  
start: ts

ts:
	@echo "📦 Starting Skelly-Jelly (TypeScript modules)..."
	@./start.sh ts

full:
	@echo "🦴 Starting Skelly-Jelly (full mode)..."
	@./start.sh full

demo:
	@echo "🎬 Starting Skelly-Jelly (demo mode)..."
	@./start.sh demo

dev:
	@echo "🔧 Starting Skelly-Jelly (development mode)..."
	@./start.sh dev

# Build targets
build:
	@echo "🔨 Building Skelly-Jelly..."
	@npm run build

build-rust:
	@echo "🦀 Building Rust modules..."
	@npm run build:rust

build-ts:
	@echo "📦 Building TypeScript modules..."
	@npm run build:ts

# Test targets
test:
	@echo "🧪 Running all tests..."
	@npm run test

test-rust:
	@echo "🦀 Running Rust tests..."
	@npm run test:rust

test-ts:
	@echo "📦 Running TypeScript tests..."
	@npm run test:ts

# Maintenance targets
clean:
	@echo "🧹 Cleaning build artifacts..."
	@npm run clean

# Help target
help:
	@echo "🦴 Skelly-Jelly - Your ADHD Companion"
	@echo ""
	@echo "Usage:"
	@echo "  make run     - Start TypeScript modules (default)"
	@echo "  make start   - Same as 'run'"
	@echo "  make ts      - Start TypeScript modules only"
	@echo "  make full    - Start with all services (TypeScript + Rust)"
	@echo "  make demo    - Start in demo mode (simulated components)"
	@echo "  make dev     - Start in development mode (hot reload)"
	@echo ""
	@echo "Build commands:"
	@echo "  make build      - Build all components"
	@echo "  make build-rust - Build only Rust modules"
	@echo "  make build-ts   - Build only TypeScript modules"
	@echo ""
	@echo "Test commands:"
	@echo "  make test      - Run all tests"
	@echo "  make test-rust - Run Rust tests only"
	@echo "  make test-ts   - Run TypeScript tests only"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean  - Clean build artifacts"
	@echo "  make help   - Show this help message"