#!/bin/bash
# Run performance benchmarks and generate reports

set -e

echo "🚀 Running Skelly-Jelly Performance Benchmarks..."
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Create results directory
RESULTS_DIR="benchmark_results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

# Run benchmarks
echo "📊 Running performance baselines..."
cargo bench --bench performance_baselines -- --save-baseline current

# Copy results
cp -r target/criterion/* "$RESULTS_DIR/" 2>/dev/null || true

# Analyze results against HLD targets
echo
echo "📈 Analyzing results against HLD targets..."
echo

# Expected performance targets from HLD
declare -A CPU_TARGETS=(
    ["event_bus"]=2
    ["orchestrator"]=1
    ["data_capture"]=5
    ["storage"]=10
    ["analysis_engine"]=20
)

declare -A MEMORY_TARGETS=(
    ["event_bus"]=100
    ["orchestrator"]=50
    ["data_capture"]=50
    ["storage"]=200
    ["analysis_engine"]=500
)

# Check event bus throughput
if [ -f "target/criterion/event_bus_throughput/1000/base/estimates.json" ]; then
    throughput=$(cat target/criterion/event_bus_throughput/1000/base/estimates.json | grep -o '"point_estimate":[0-9.]*' | cut -d: -f2)
    messages_per_sec=$(echo "scale=0; 1000000000 / $throughput" | bc 2>/dev/null || echo "0")
    
    echo -n "Event Bus Throughput: "
    if [ "$messages_per_sec" -gt 1000 ]; then
        echo -e "${GREEN}✓ $messages_per_sec msg/sec (target: >1000)${NC}"
    else
        echo -e "${RED}✗ $messages_per_sec msg/sec (target: >1000)${NC}"
    fi
fi

# Check event bus latency
if [ -f "target/criterion/event_bus_latency/base/estimates.json" ]; then
    latency=$(cat target/criterion/event_bus_latency/base/estimates.json | grep -o '"point_estimate":[0-9.]*' | cut -d: -f2)
    latency_ms=$(echo "scale=2; $latency / 1000000" | bc 2>/dev/null || echo "0")
    
    echo -n "Event Bus Latency: "
    if (( $(echo "$latency_ms < 1" | bc -l) )); then
        echo -e "${GREEN}✓ ${latency_ms}ms (target: <1ms)${NC}"
    else
        echo -e "${RED}✗ ${latency_ms}ms (target: <1ms)${NC}"
    fi
fi

# Generate summary report
echo
echo "📝 Generating summary report..."

cat > "$RESULTS_DIR/summary.md" << EOF
# Skelly-Jelly Performance Benchmark Results

Date: $(date)

## Summary

### Event Bus Performance
- Throughput: ${messages_per_sec:-N/A} messages/second (target: >1000)
- Latency: ${latency_ms:-N/A}ms (target: <1ms)

### Module Resource Usage vs Targets

| Module | CPU Target | Memory Target |
|--------|------------|---------------|
| Event Bus | 2% | 100MB |
| Orchestrator | 1% | 50MB |
| Data Capture | 5% | 50MB |
| Storage | 10% | 200MB |
| Analysis Engine | 20% | 500MB |

## Recommendations

1. Continue monitoring performance as modules are implemented
2. Set up CI/CD to run benchmarks on each commit
3. Create alerts for performance regressions
4. Profile actual resource usage in production

## Raw Results

See \`target/criterion/\` for detailed benchmark results and graphs.
EOF

echo -e "${GREEN}✓ Results saved to $RESULTS_DIR${NC}"
echo
echo "View detailed results:"
echo "  open target/criterion/report/index.html"
echo
echo "View summary:"
echo "  cat $RESULTS_DIR/summary.md"