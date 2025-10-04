#!/bin/bash

# The command `set -e` makes the script exit immediately if any command exits with a non-zero status.
# The command `set -x` makes the script print each command before executing it.
set -e
set -x

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

OPENSSL_DIR=$(brew --prefix openssl) cargo build --target=x86_64-unknown-linux-gnu --release
strip target/x86_64-unknown-linux-gnu/release/ledgerbot
scp target/x86_64-unknown-linux-gnu/release/ledgerbot ${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/ledgerbot
