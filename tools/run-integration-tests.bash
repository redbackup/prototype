#!/usr/bin/env bash
set -e

run_minimal () {
    set -ux
    
    # The tag of the images used for this run
    export TAG="$(git describe --tags)"

    # Start the nodes
    docker-compose -f docker/docker-compose.minimal.yml -p "minimal-$TAG" up -d nodeA nodeB

    # Wait a moment - hopefully until the nodes are ready
    sleep 5

    # Start a client
    # TODO: Actually do sometihg - e.g. make a backup
    docker-compose -f docker/docker-compose.minimal.yml -p "minimal-$TAG" up client1

    # TODO: Verify that replication is completed

    # Stop the nodes
    docker-compose -f docker/docker-compose.minimal.yml -p "minimal-$TAG" stop nodeA nodeB
}


run_medium () {
    set -ux
        
    # The tag of the images used for this run
    export TAG="$(git describe --tags)"

    # Start the nodes
    docker-compose -f docker/docker-compose.medium.yml -p "medium-$TAG" up -d nodeA nodeB nodeC

    # Wait a moment - hopefully until the nodes are ready
    sleep 5

    # Start the clients
    # TODO: Actually do sometihg - e.g. make a backup
    docker-compose -f docker/docker-compose.medium.yml -p "medium-$TAG" up client1 client2 client3

    # TODO: Verify that replication is completed

    # Stop the nodes
    docker-compose -f docker/docker-compose.medium.yml -p "medium-$TAG" stop nodeA nodeB nodeC
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




