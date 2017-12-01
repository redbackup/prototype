"""
This module contains different configurations for possible integration tests
"""
import logging
from abc import ABC, abstractmethod

import docker
from docker.errors import ImageNotFound
from docker.models.networks import Network
from typing import List

from pyredbackup import Client, Node

LOG = logging.getLogger(__name__)


class BaseConfiguration(ABC):
    """
    Generic abstraction that can be subclassed for concrete
    configs.
    """

    def __init__(self, version: str) -> None:
        self.docker = docker.from_env()
        self.version = version
        self._check_for_images()
        self.network = self._init_network()
        self.clients = self._init_clients()
        self.nodes = self._init_nodes()

    def _init_network(self) -> Network:
        # TODO: generate based on the name...
        return self.docker.networks.create(f"redbackup_medium_{self.version}")

    @abstractmethod
    def _init_nodes(self) -> List[Node]:
        pass

    @abstractmethod
    def _init_clients(self) -> List[Client]:
        pass

    def _check_for_images(self) -> None:
        # TODO: build images here - including `cargo build --release`
        try:
            node = self.docker.images.get(f'{Node.IMAGE}:{self.version}')
            LOG.debug(f"Node image {node.tags} exists...")
            client = self.docker.images.get(f'{Client.IMAGE}:{self.version}')
            LOG.debug(f"Client image {client.tags} exists...")
        except ImageNotFound as err:
            LOG.error(
                "Please build all images before running the integration tests")
            raise err

    def clean_up(self) -> None:
        """
        Cleans up clients and nodes and all other related data
        (e.g. docker networks)
        """
        LOG.info(f"Cleaning up...")

        for node in self.nodes:
            node.clean_up()

        self.network.remove()

    def start_nodes(self) -> None:
        """
        This is just a convenience method to call `create` and
        then `start` on all nodes.
        """
        LOG.info(f"Starting nodes...")
        for node in self.nodes:
            node.create()

        for node in self.nodes:
            node.start()

    def stop_nodes(self) -> None:
        """
        This is just a convenience method to call `stop` on all nodes.
        """
        LOG.info(f"Stopping nodes...")
        for node in self.nodes:
            node.stop()


class MediumConfiguration(BaseConfiguration):
    """
    Medium as described in
    the test section of the SA documentation.
    """

    def _init_nodes(self) -> List[Node]:
        LOG.info(f"Initializing nodes...")
        self.node_a = Node('NodeA', self.version, self.network, self.docker)
        self.node_b = Node('NodeB', self.version, self.network, self.docker)
        self.node_c = Node('NodeC', self.version, self.network, self.docker)
        self.node_a.known_nodes(self.node_b, self.node_c)
        self.node_b.known_nodes(self.node_a, self.node_c)
        self.node_c.known_nodes(self.node_a, self.node_b)

        return [self.node_a, self.node_b, self.node_c]

    def _init_clients(self) -> List[Client]:
        LOG.info(f"Initializing clients...")
        self.client_1 = Client('client1', self.version,
                               self.network, self.docker)
        self.client_2 = Client('client2', self.version,
                               self.network, self.docker)
        self.client_3 = Client('client3', self.version,
                               self.network, self.docker)

        return self.client_1, self.client_2, self.client_3
