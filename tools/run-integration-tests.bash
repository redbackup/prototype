#!/usr/bin/env bash
set -e

run_minimal () {
    set -ux

    # Build everything
    ./buildimages.bash

    # Start the nodes
    docker-compose -f docker/docker-compose.minimal.yml up -d nodeA nodeB

    # Wait a moment - hopefully until the nodes are ready
    sleep 5

    # Start a client
    # TODO: Actually do sometihg - e.g. make a backup
    docker-compose -f docker/docker-compose.minimal.yml up client1

    # TODO: Verify that replication is completed

    # Stop the nodes
    docker-compose -f docker/docker-compose.minimal.yml stop nodeA nodeB
}


run_medium () {
    set -ux

    # Build everything
    ./buildimages.bash

    # Start the nodes
    docker-compose -f docker/docker-compose.medium.yml up -d nodeA nodeB nodeC

    # Wait a moment - hopefully until the nodes are ready
    sleep 5

    # Start the clients
    # TODO: Actually do sometihg - e.g. make a backup
    docker-compose -f docker/docker-compose.medium.yml up client1 client2 client3

    # TODO: Verify that replication is completed

    # Stop the nodes
    docker-compose -f docker/docker-compose.medium.yml stop nodeA nodeB nodeC
}

# cd into the directory, in which this script is stored
cd "$(dirname "$0")"

case $1 in
minimal)
    run_minimal
    ;;
medium)
    run_medium
    ;;
*)
    echo $"Usage: $0 {minimal|medium}"
    exit 1
esac




