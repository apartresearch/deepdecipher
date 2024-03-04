#!/bin/bash

# Exit script if any command fails
set -e

# Check Python version (assuming Python 3 is installed as 'python')
PYTHON_VERSION=$(python -V)
echo "Detected Python version: $PYTHON_VERSION"

# Check Rust toolchain
if ! command -v cargo &> /dev/null
then
    echo "Rust toolchain not found. Please install Rust."
    exit 1
fi
echo "Rust toolchain detected."

# -- Assuming you're already in the root folder --

# Ensure a Python virtual environment is activated
# This part depends on whether you're using conda or venv.
# The script will proceed with venv for this example.

# Check if a virtual environment is already active
if [[ -z "$VIRTUAL_ENV" ]]; then
    echo "No virtual environment detected. Creating one..."
    python -m venv .env
    source .env/bin/activate
else
    echo "Virtual environment detected."
fi

# Install maturin
echo "Installing maturin..."
python -m pip install maturin

# Validate Python interpreter type
PYTHON_INTERPRETER_TYPE=$(python -c "import platform; print(platform.python_implementation())")
if [ "$PYTHON_INTERPRETER_TYPE" != "CPython" ]; then
    echo "Unsupported Python interpreter for cross-compilation: $PYTHON_INTERPRETER_TYPE. Please ensure you're using CPython."
    exit 1
fi

# Proceed with maturin development build
echo "Building the DeepDecipher package with maturin..."
maturin develop --release

# Run the scrape Neuroscope script
echo "Running the scrape Neuroscope script..."
python -m scripts.scrape_neuroscope

# Start the DeepDecipher server
echo "Starting the DeepDecipher server..."
python -m DeepDecipher data.db &

# Give the server a moment to start
sleep 5

# Check if the server is running by accessing the API
echo "Verifying server response..."
curl http://localhost:8080/api/solu-1l/neuroscope/0/9

echo "Setup completed. Visit http://localhost:8080/viz/solu-1l/all/0/9 in your browser to see visualizations."
