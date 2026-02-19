#!/bin/bash

# format.sh - Run cargo fmt for all subprojects

set -e

echo "Formatting butt-head workspace..."

# Root project
echo "  [1/4] Root project"
cargo fmt

# Examples
echo "  [2/4] examples/native"
(cd examples/native && cargo fmt)

echo "  [3/4] examples/stm32f0"
(cd examples/stm32f0 && cargo fmt)

echo "  [4/4] examples/stm32f0-embassy"
(cd examples/stm32f0-embassy && cargo fmt)

echo ""
echo "Format complete!"
