"""
This module contains an implementation to controll a redbackup client in a
docker container.
"""
import logging

from docker.client import DockerClient
from docker.models.networks import Network

LOG = logging.getLogger(__name__)


class Client:
    """
    Abstraction to controll a redbackup client in a docker container.
    """

    IMAGE = 'redbackup/client'

    def __init__(self, name: str, version: str,
                 network: Network, docker: DockerClient) -> None:
        self.name = name
        self.network = network
        self.docker = docker
        self.image = f'{Client.IMAGE}:{version}'
        LOG.debug(f"Setting up client {self.name}")
        # TODO: create containers ad_hoc
        # commit after every command and use that image as base for
        # the next execution
        # TODO: `put_archive(path, data)` (also: commit!)

    def _run_sync(self):
        # TODO: Debug...
        container = self.docker.containers.create(self.image)
        self.network.connect(container)
        container.start()
        container.wait()
        container.logs()
        self.image = container.commit().short_id()
