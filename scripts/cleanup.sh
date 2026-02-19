#!/bin/bash

# cleanup.sh - Run cargo clean for all subprojects

set -e

echo "Cleaning butt-head workspace..."

# Root project
echo "  [1/5] Root project"
cargo clean

# Examples
echo "  [2/5] examples/native"
(cd examples/native && cargo clean)

echo "  [3/5] examples/stm32f0"
(cd examples/stm32f0 && cargo clean)

echo "  [4/5] examples/stm32f0-embassy"
(cd examples/stm32f0-embassy && cargo clean)

# Remove tmp directory if it exists
if [ -d "tmp" ]; then
    echo "  [5/5] Removing tmp directory"
    rm -rf tmp
else
    echo "  [5/5] tmp directory (not present)"
fi

echo ""
echo "Cleanup complete!"
