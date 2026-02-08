#!/bin/bash

# PLONK Integration Tests
# This script runs integration tests for PLONK proof system

set -e

echo "=========================================="
echo "PLONK Integration Tests"
echo "=========================================="
echo

# Check if pytest is installed
if ! command -v pytest &> /dev/null; then
    echo "Error: pytest is not installed"
    echo "Install it with: pip install pytest"
    exit 1
fi

# Check if requests is installed
if ! python3 -c "import requests" &> /dev/null; then
    echo "Installing requests..."
    pip install requests
fi

# Check if web3 is installed
if ! python3 -c "import web3" &> /dev/null; then
    echo "Installing web3..."
    pip install web3
fi

# Check if relayer is running
if ! curl -s http://localhost:8080/api/v1/health &> /dev/null; then
    echo "Warning: Relayer is not running at http://localhost:8080"
    echo "Start the relayer with: cargo run -p relayer"
    echo "Some tests may fail"
    echo
fi

# Run PLONK-specific tests
echo "Running PLONK integration tests..."
echo
pytest tests/test_plonk.py -v --tb=short

echo
echo "=========================================="
echo "PLONK Integration Tests Complete"
echo "=========================================="
