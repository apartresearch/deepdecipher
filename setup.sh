#!/bin/bash

# Exit script if any command fails
set -e

# Setup variables
DATABASE_FILE="data.db"
MODEL_NAME="solu-1l"

echo "Checking Python version..."
PYTHON_VERSION=$(python -V 2>&1)
echo "Detected Python version: $PYTHON_VERSION"

if ! command -v cargo &> /dev/null; then
    echo "Rust toolchain not found. Please install Rust."
    exit 1
fi
echo "Rust toolchain detected."

# Assuming you're in the root of the cloned repo

echo "Setting up Python virtual environment..."
python -m venv .env

source .env/bin/activate

if [[ -z "$VIRTUAL_ENV" ]]; then
    echo "Virtual environment activation failed."
    exit 1
fi
echo "Virtual environment activated at $VIRTUAL_ENV"

echo "Installing maturin..."
python -m pip install maturin

echo "Building the DeepDecipher package with maturin..."
# Explicitly specifying the Python interpreter might help
maturin develop --release

echo "Verifying DeepDecipher installation..."
# Checking if the installation path includes DeepDecipher
python -c "import site; print(site.getsitepackages())"
python -c "import deepdecipher; print('DeepDecipher module found.')"

echo "Running the scrape Neuroscope script..."
python -m scripts.scrape_neuroscope $DATABASE_FILE $MODEL_NAME

echo "Starting the DeepDecipher server..."
python -m deepdecipher $DATABASE_FILE &

sleep 5

echo "Verifying server response..."
curl http://localhost:8080/api/$MODEL_NAME/neuroscope/0/9


# Check that npm is installed
if ! command -v npm &> /dev/null; then
    echo "npm not found. Please install npm."
    exit 1
fi
echo "npm detected."

# Run the frontend server on port
echo "Starting the frontend server..."
cd frontend
npm install
npm audit fix
npm run dev