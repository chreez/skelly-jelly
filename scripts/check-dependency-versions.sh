#!/bin/bash
# Check dependency version consistency across all modules

set -e

echo "🔍 Checking dependency version consistency..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track inconsistencies
INCONSISTENCIES=0

# Common dependencies to check
DEPS=(
    "tokio"
    "serde"
    "thiserror"
    "dashmap"
    "chrono"
    "uuid"
    "anyhow"
    "tracing"
    "parking_lot"
    "crossbeam-channel"
)

# Function to extract version from Cargo.toml
get_version() {
    local file=$1
    local dep=$2
    grep -E "^$dep\s*=.*version\s*=" "$file" | sed -E 's/.*version\s*=\s*"([^"]+)".*/\1/' | head -1
}

# Check each dependency
for dep in "${DEPS[@]}"; do
    echo -n "Checking $dep... "
    
    # Find all Cargo.toml files
    versions=()
    files=()
    
    while IFS= read -r file; do
        version=$(get_version "$file" "$dep")
        if [ -n "$version" ]; then
            versions+=("$version")
            files+=("$file")
        fi
    done < <(find modules -name "Cargo.toml" -type f | grep -v target)
    
    # Check if all versions are the same
    if [ ${#versions[@]} -gt 0 ]; then
        first_version="${versions[0]}"
        all_same=true
        
        for i in "${!versions[@]}"; do
            if [ "${versions[$i]}" != "$first_version" ]; then
                all_same=false
                break
            fi
        done
        
        if $all_same; then
            echo -e "${GREEN}✓${NC} (${first_version})"
        else
            echo -e "${RED}✗${NC}"
            echo -e "${YELLOW}  Inconsistent versions found:${NC}"
            for i in "${!versions[@]}"; do
                echo "    ${files[$i]}: ${versions[$i]}"
            done
            ((INCONSISTENCIES++))
        fi
    else
        echo "Not found in any module"
    fi
done

# Check workspace dependencies catalog exists
if [ -f "workspace-dependencies.toml" ]; then
    echo -e "\n${GREEN}✓${NC} Workspace dependencies catalog found"
else
    echo -e "\n${RED}✗${NC} Workspace dependencies catalog not found"
    ((INCONSISTENCIES++))
fi

# Summary
echo -e "\n📊 Summary:"
if [ $INCONSISTENCIES -eq 0 ]; then
    echo -e "${GREEN}All dependency versions are consistent!${NC}"
    exit 0
else
    echo -e "${RED}Found $INCONSISTENCIES inconsistencies${NC}"
    exit 1
fi