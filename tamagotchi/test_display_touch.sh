#!/bin/bash
# Test script for display and touch functionality

echo "Running tamagotchi with full logging..."
echo "Output will be saved to test_output.log"

cargo run --release 2>&1 | tee test_output.log

echo ""
echo "=== Test Results ==="
echo ""
echo "Checking for touch controller..."
grep -E "(FT3168|touch|0x38)" test_output.log | head -20
echo ""
echo "Checking for display..."
grep -E "(display|SH8601|Red screen|SPI)" test_output.log | head -20
echo ""
echo "Full log saved to test_output.log"
