#!/usr/bin/env bash
set -eux

# cd into the project root
cd "$(dirname "$0")/.."

# Evaluate the current version
VERSION="$(git describe --tags)"

# build docker images
docker build --no-cache -f tools/docker/client.dockerfile -t "redbackup/client:$VERSION" .
docker build --no-cache -f tools/docker/node.dockerfile -t "redbackup/node:$VERSION" .
