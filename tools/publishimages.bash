#!/usr/bin/env bash
set -eu

# cd into the directory, in which this script is stored
cd "$(dirname "$0")"

# Evaluate the current version
VERSION="$(git describe --tags)"

# Login to the docker registry
docker login -u ${bamboo.dockercloud.username} -p ${bamboo.dockercloud.password}

# Push all tags
docker push "redbackup/client:$VERSION"
docker push "redbackup/node:$VERSION"

# Logount from the docker registry
docker logout