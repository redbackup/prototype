"""
This module contains an implementation to controll a redbackup node in a
docker container.
"""
import logging
from typing import List

from docker.client import DockerClient
from docker.models.containers import Container
from docker.models.networks import Network

from pyredbackup.helpers import check_log_for_errors

LOG = logging.getLogger(__name__)


class Node:
    """
    Abstraction to controll a redbackup node in a docker container.
    """

    IMAGE = 'redbackup/node'
    PORT = '8080'
    SLEEP_BEFORE_LAUNCH = 5

    def __init__(self, name: str, version: str, network: Network,
                 docker: DockerClient) -> None:
        self.name = name
        self.network = network
        self.docker = docker
        self.version = version
        self._known_nodes = []  # type: List['Node']
        self.container = None  # type: Container
        LOG.debug(f"Setting up node {self.name}")
        # TODO: allow `put_archive(path, data)` and `get_archive(path)`

    def clean_up(self) -> None:
        """
        Cleans up this container
        """
        LOG.debug(f"Cleaning up node {self.name}")
        self.container.remove()

    def known_nodes(self, *args: 'Node') -> None:
        """
        Adds the given node to its known nodes.
        """
        for node in args:
            self._known_nodes.append(node)

    def start(self) -> None:
        """
        Starts this node and blocks until its port is accessible.
        """
        LOG.debug(f"Starting node {self.name}")
        self.container.start()

    def create(self) -> None:
        """
        Creates a container for this node, linked to all other known nodes
        """
        self.container = self.docker.containers.create(
            f'{Node.IMAGE}:{self.version}',
            entrypoint=self._generate_start_command(),
            name=self.name,
            hostname=self.name,
            environment=["RUST_BACKTRACE=1", "RUST_LOG=redbackup=info"])
        self.network.connect(self.container)
        LOG.debug(f"{self.name} is now container {self.container.name}")

    def _generate_start_command(self) -> str:
        """
        This method generates the actual CLI-Call. It also adds a short
        timeout because we would run in network problems otherwise (trying to
        resolve hosts that are not yet up).
        """
        command = f'bash -c "sleep {Node.SLEEP_BEFORE_LAUNCH}; '\
            '/usr/local/bin/redbackup-node'
        for node in self._known_nodes:
            command += f' -k {node.name}'
        command = command + '"'
        LOG.debug(f'command for {self.name} is {command}')
        return command

    def stop(self) -> None:
        """
        (forcefully) stops this node
        """
        LOG.debug(f"Stopping node {self.name}")
        self.container.stop(timeout=2)
        check_log_for_errors(self.container)
