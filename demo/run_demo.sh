#!/bin/bash
# Skelly-Jelly Demo Script
# This demonstrates the full system working together

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

echo "🦴 Skelly-Jelly Demo"
echo "=================="
echo ""
echo "This demo will show:"
echo "1. System startup and module initialization"
echo "2. Simulated ADHD state transitions"
echo "3. Intervention generation and delivery"
echo "4. Skeleton companion animations"
echo ""
echo "Press Enter to start..."
read

# Clean up any previous demo data
echo "🧹 Cleaning up previous demo data..."
rm -rf "$PROJECT_ROOT/data/demo"
mkdir -p "$PROJECT_ROOT/data/demo"

# Create demo configuration
echo "📝 Creating demo configuration..."
cat > "$PROJECT_ROOT/config/demo.toml" << 'EOF'
# Demo Configuration - Optimized for demonstration

[event_bus]
max_queue_size = 1000
message_timeout_ms = 2000
enable_persistence = false

[orchestrator]
health_check_interval_ms = 10000
startup_timeout_ms = 30000
shutdown_timeout_ms = 10000
enable_recovery = true
max_recovery_attempts = 3

[storage]
database_path = "./data/demo/skelly.db"
max_batch_size = 100
batch_timeout_ms = 5000
retention_days = 1
enable_compression = false

[data_capture]
# Demo mode with synthetic data
demo_mode = true
synthetic_event_rate_hz = 5
demo_scenarios = [
    "focused_work",
    "getting_distracted",
    "deep_flow",
    "need_break",
    "celebration"
]

[analysis_engine]
# Use lightweight model for demo
model_path = "./models/demo_classifier.onnx"
inference_threads = 2
window_size_seconds = 10
window_overlap_seconds = 2
confidence_threshold = 0.6
enable_gpu = false

[gamification]
[gamification.intervention]
min_cooldown_minutes = 1  # Quick interventions for demo
adaptive_cooldown = false
max_interventions_per_hour = 20
respect_flow_states = true
flow_state_threshold = 0.7

[gamification.rewards]
coins_per_focus_minute = 10  # Generous rewards for demo
bonus_multiplier = 2.0

[ai_integration]
[ai_integration.local_model]
# Use template responses for demo
use_templates = true
model_variant = "Demo"

[ai_integration.privacy]
default_privacy_level = "LocalOnly"

[cute_figurine]
window_width = 300
window_height = 300
enable_transparency = true
always_on_top = true
animation_fps = 30
demo_animations = true  # Show all animations in sequence
EOF

# Build the project (if needed)
echo "🔨 Building Skelly-Jelly..."
cd "$PROJECT_ROOT"
if command -v cargo &> /dev/null; then
    echo "Building Rust modules..."
    cargo build --release 2>&1 | grep -E "(error:|warning:|Compiling|Finished)" || true
    BINARY="$PROJECT_ROOT/target/release/skelly-jelly"
    
    # Note about TypeScript modules
    echo ""
    echo "ℹ️  Note: This demo shows the Rust module integration."
    echo "   TypeScript modules (Gamification, Cute Figurine) are simulated."
else
    echo "⚠️  Cargo not found, using pre-built binary..."
    BINARY="$PROJECT_ROOT/bin/skelly-jelly"
fi

# Start the demo
echo ""
echo "🚀 Starting Skelly-Jelly in demo mode..."
echo ""
echo "Watch for:"
echo "  💀 Skeleton companion appearing on screen"
echo "  📊 State transitions in the logs"
echo "  💬 Intervention messages"
echo "  🎉 Celebrations and rewards"
echo ""
echo "Press Ctrl+C to stop the demo"
echo ""

# Run the demo
if [ -f "$BINARY" ]; then
    "$BINARY"
else
    echo "⚠️  Binary not found at $BINARY"
    echo "💡 Trying to run with cargo..."
    cd "$PROJECT_ROOT"
    cargo run --release
fi

echo ""
echo "👋 Demo complete! Thanks for trying Skelly-Jelly!"