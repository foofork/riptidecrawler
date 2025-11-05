#!/usr/bin/env bash
# Test script for riptide Python package

set -e

echo "Testing riptide Python package..."

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install package in development mode
echo "Installing package in development mode..."
pip install -e .

# Install test dependencies
echo "Installing test dependencies..."
pip install pytest pytest-asyncio pytest-benchmark

# Run tests
echo "Running Python tests..."
pytest tests/ -v

# Run basic smoke test
echo "Running smoke test..."
python3 -c "
import riptide
rt = riptide.RipTide()
print('✅ RipTide import successful')
print(f'✅ RipTide instance created: {rt}')
"

echo "All tests passed!"
