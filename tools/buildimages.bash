#!/usr/bin/env bash
set -eux

# cd into the project root
cd "$(dirname "$0")/.."

# build everything
cargo build --all --release

# Evaluate the current version
VERSION="$(git describe --tags)"

# build docker images
docker build --no-cache -f tools/docker/client.dockerfile -t "redbackup/client:$VERSION" .
docker build --no-cache -f tools/docker/node.dockerfile -t "redbackup/node:$VERSION" .

# Tag this release as "latest"
docker image tag "redbackup/client:$VERSION" redbackup/client:latest
docker image tag "redbackup/node:$VERSION" redbackup/node:latest
