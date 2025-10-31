#!/usr/bin/env bash
# Test script to run all examples and verify they start without crashing
# This catches runtime issues that cargo build misses
#
# Note: Many examples require an audio device. Without one, they will fail with
# "Device not configured" errors. This is expected in headless environments.
#
# Usage:
#   ./scripts/test-examples.sh [--duration SECONDS]
#
# Options:
#   --duration SECONDS    How long to run each example (default: 2)

set -e

DURATION=2
ALL_FEATURES=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --all-features)
            ALL_FEATURES="--all-features"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--duration SECONDS] [--all-features]"
            exit 1
            ;;
    esac
done

# Get list of all examples from Cargo.toml
echo "Discovering examples..."
EXAMPLES=$(cargo metadata --format-version=1 --no-deps | \
    jq -r '.packages[] | select(.name == "earworm") | .targets[] | select(.kind[] == "example") | .name')

if [ -z "$EXAMPLES" ]; then
    echo "No examples found!"
    exit 1
fi

echo "Found $(echo "$EXAMPLES" | wc -l) examples"
echo "Running each for ${DURATION}s ${ALL_FEATURES}"
echo ""

FAILED=()
PASSED=()

for example in $EXAMPLES; do
    echo -n "Testing $example... "

    # Run the example in background with timeout
    if timeout ${DURATION}s cargo run --example "$example" $ALL_FEATURES > /dev/null 2>&1; then
        # Exit code 0 means it ran successfully for the duration
        echo "✓ passed"
        PASSED+=("$example")
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ]; then
            # Exit code 124 from timeout means it was killed after duration (success)
            echo "✓ passed"
            PASSED+=("$example")
        else
            # Any other exit code means it crashed
            echo "✗ FAILED (exit code: $EXIT_CODE)"
            FAILED+=("$example")
        fi
    fi
done

echo ""
echo "================================"
echo "Results: ${#PASSED[@]} passed, ${#FAILED[@]} failed"

if [ ${#FAILED[@]} -gt 0 ]; then
    echo ""
    echo "Failed examples:"
    for example in "${FAILED[@]}"; do
        echo "  - $example"
    done
    exit 1
fi

echo "All examples ran successfully!"
