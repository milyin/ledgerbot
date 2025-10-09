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
CARGO_BUILD_FLAGS=--release ./build.sh

echo "Deploying ledgerbot on remote host ${DEPLOY_HOST}..."
echo "----------------------------------------"

# stop the ledgerbot service before copying
echo "Stopping ledgerbot service..."
ssh -t ${DEPLOY_USER}@${DEPLOY_HOST} "sudo systemctl stop ledgerbot"
echo "Service stopped"

scp target/x86_64-unknown-linux-gnu/release/ledgerbot ${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH_RELEASE}/ledgerbot

# make ls command to show the deployed binary details
ssh ${DEPLOY_USER}@${DEPLOY_HOST} "ls -lh ${DEPLOY_PATH_RELEASE}/ledgerbot"

# If in release mode, start the ledgerbot service after copying
echo "Starting ledgerbot service..."
ssh -t ${DEPLOY_USER}@${DEPLOY_HOST} "sudo systemctl start ledgerbot"
echo "Service started successfully"
