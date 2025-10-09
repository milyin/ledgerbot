#!/bin/bash

# The command `set -e` makes the script exit immediately if any command exits with a non-zero status.
set -e

# Source environment variables from .env file if it exists
if [ -f ".env" ]; then
  export $(grep -v '^#' .env | xargs)
fi

# Determine the build directory based on CARGO_BUILD_FLAGS
if [[ "$CARGO_BUILD_FLAGS" == *"--release"* ]]; then
  BUILD_DIR="release"
else
  BUILD_DIR="debug"
fi

# Check if DEPLOY_USER, DEPLOY_HOST, and DEPLOY_PATH are set
if [ -z "$DEPLOY_USER" ] || [ -z "$DEPLOY_HOST" ] || [ -z "$DEPLOY_PATH" ]; then
  echo "Error: DEPLOY_USER, DEPLOY_HOST, and DEPLOY_PATH environment variables must be set."
  echo "Please copy .env.example to .env and fill in your deployment configuration."
  exit 1
fi

echo "Building and deploying ledgerbot..."
# Run the build script
./build.sh

echo "Starting ledgerbot on remote host ${DEPLOY_HOST}..."
echo "Press Ctrl+C to stop the remote process and exit"
echo "----------------------------------------"

scp target/x86_64-unknown-linux-gnu/$BUILD_DIR/ledgerbot ${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/ledgerbot

# make ls command to show the deployed binary details
ssh ${DEPLOY_USER}@${DEPLOY_HOST} "ls -lh ${DEPLOY_PATH}/ledgerbot"

# If in release mode, start the ledgerbot service after copying
if [[ "$BUILD_DIR" == "release" ]]; then
  echo "Starting ledgerbot service..."
  ssh -t ${DEPLOY_USER}@${DEPLOY_HOST} "sudo systemctl start ledgerbot"
  echo "Service started successfully"
fi

# SSH to the remote host and run ledgerbot, showing output locally
# Use -tt to force TTY allocation and ensure output is shown
# The -o options help with proper output handling
ssh -tt -o LogLevel=ERROR ${DEPLOY_USER}@${DEPLOY_HOST} "cd ${DEPLOY_PATH} && RUST_LOG=debug ./ledgerbot 2>&1"