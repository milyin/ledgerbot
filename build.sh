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

# download openssl source code from 
# https://github.com/openssl/openssl/releases/download/openssl-3.6.0/openssl-3.6.0.tar.gz 
# into target/openssl if it does not exist there yet
if [ ! -d "target/openssl-3.6.0" ]; then
  mkdir -p target
  cd target
  curl -LO https://github.com/openssl/openssl/releases/download/openssl-3.6.0/openssl-3.6.0.tar.gz
  tar -xzf openssl-3.6.0.tar.gz
  cd openssl-3.6.0
  CC=x86_64-linux-gnu-gcc LD=x86_64-linux-gnu-ld \
  ./config linux-x86_64 --prefix=$(pwd)/install no-asm no-engine no-shared
  make
  make install
  cd ../../
fi

OPENSSL_DIR=$PWD/target/openssl-3.6.0/install cargo build --target=x86_64-unknown-linux-gnu $CARGO_BUILD_FLAGS
strip target/x86_64-unknown-linux-gnu/release/ledgerbot
scp target/x86_64-unknown-linux-gnu/release/ledgerbot ${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/ledgerbot
