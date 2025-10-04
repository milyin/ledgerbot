#!/bin/bash

# The command `set -e` makes the script exit immediately if any command exits with a non-zero status.
set -e

# Source environment variables from .env file if it exists
if [ -f ".env" ]; then
  export $(grep -v '^#' .env | xargs)
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

# SSH to the remote host and run ledgerbot, showing output locally
# Use -tt to force TTY allocation and ensure output is shown
# The -o options help with proper output handling
ssh -tt -o LogLevel=ERROR ${DEPLOY_USER}@${DEPLOY_HOST} "cd ${DEPLOY_PATH} && RUST_LOG=debug ./ledgerbot 2>&1"