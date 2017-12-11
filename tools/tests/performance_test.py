"""
Performance tests for redbackup.
"""

import logging
import os
import random
import tempfile
import time
from typing import List

import docker
import iso8601
import pytest

from pyredbackup.client import Client
from pyredbackup.configuration import BaseConfiguration
from pyredbackup.helpers import check_log_for_errors
from pyredbackup.node import Node
from pyredbackup.utils import current_version, minimal_configuration

LOG = logging.getLogger(__name__)


class PerfTestConfiguration(BaseConfiguration):
    """
    A Performance test configuration for performance testing
    """

    def __init__(self, version: str, number_of_clients: int,
                 number_of_nodes: int) -> None:
        self.number_of_clients = number_of_clients
        self.number_of_nodes = number_of_nodes
        super().__init__(version)

    def _init_nodes(self) -> List[Node]:
        LOG.info(f"Initializing nodes...")
        nodes = []
        for idx in range(1, self.number_of_nodes + 1):
            nodes.append(Node(f'Node{idx}', self.version,
                              self.network, self.docker))

        for node in nodes:
            node.known_nodes(*[x for x in nodes if node.name != x.name])

        return nodes

    def _init_clients(self) -> List[Client]:
        LOG.info(f"Initializing clients...")
        clients = []
        for idx in range(1, self.number_of_clients + 1):
            clients.append(Client(f'Client{idx}', self.version,
                                  self.network, self.docker))

        return clients


@pytest.fixture()
def many_clients_config():
    """
    Test fixture for py.test to simplify writing tests.
    """
    version = current_version()
    config = PerfTestConfiguration(version, 5, 1)
    config.start_nodes()
    try:
        yield config
    finally:
        LOG.info("Cleaning up nodes...")
        config.stop_nodes()
        config.clean_up()


@pytest.fixture()
def many_nodes_config():
    """
    Test fixture for py.test to simplify writing tests.
    """
    _tmp = Node.SLEEP_BEFORE_LAUNCH
    Node.SLEEP_BEFORE_LAUNCH = 30
    version = current_version()
    config = PerfTestConfiguration(version, 1, 10)
    config.start_nodes()
    try:
        yield config
    finally:
        Node.SLEEP_BEFORE_LAUNCH = _tmp
        LOG.info("Cleaning up nodes...")
        config.stop_nodes()
        config.clean_up()


def generate_random_files_contents(target_directory: str,
                                   number_of_files: int,
                                   file_size_min: int,
                                   file_size_max: int) -> int:
    """
    Generates `number_of_files` random files in the target directory
    each containing random data between `file_size_min` and
    `file_size_max` MB.

    Note that this method might override existing files in
    the given directory (if they start with the prefix `file_`)

    Returns the total size of the created files in MB
    """
    total_size_in_mb = 0
    for idx in range(1, 1 + number_of_files):
        fname = f'file_{idx}'
        with open(os.path.join(target_directory, fname), 'wb') as hand:
            file_size_in_mb = random.randint(file_size_min, file_size_max)
            total_size_in_mb += file_size_in_mb
            for _ in range(1, file_size_in_mb):
                hand.write(os.urandom(1024 * 1024))
                hand.flush()
            LOG.info(f'Generated random file {fname} ({file_size_in_mb} MB)')
    return total_size_in_mb


def test_concurrent_backup_performance(
        many_clients_config: PerfTestConfiguration):
    """
    Context: Performance Test Setup (3 Nodes, few clients)
    Scenarion:
    - all clients create a backup on a random host
    - wait for completion and verify that all clients succeeded
    """

    handles = {}
    for client in many_clients_config.clients:
        node = random.choice(many_clients_config.nodes)
        with tempfile.TemporaryDirectory() as tmpdirname:
            LOG.info(f'Generating random backup data for '
                     f'{client.name} in {tmpdirname}')
            backup_size = generate_random_files_contents(
                tmpdirname, random.randint(1, 10), 1, 100)
            print(f'Size of {client.name} backup-size: {backup_size}')
            handles[client] = client.prepare_backup_detached(
                tmpdirname, '2099-04-12T17:49', node)

    for (client, container) in handles.items():
        LOG.debug(f'Starting client container {client.name}...')
        container.start()

    try:
        api_client = docker.APIClient()
        print('')
        for (client, container) in handles.items():
            exit_code = container.wait(timeout=3 * 60)
            check_log_for_errors(container)
            assert exit_code == 0
            details = api_client.inspect_container(container.id)
            start = iso8601.parse_date(details['State']['StartedAt'])
            finish = iso8601.parse_date(details['State']['FinishedAt'])
            duration = str(finish - start)
            print(f'Container {client.name} took {duration}')
    finally:
        LOG.info("Cleaning up clients...")
        for (_, container) in handles.items():
            container.remove()

