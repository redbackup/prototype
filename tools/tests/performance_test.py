"""
Performance tests for redbackup.
"""

import logging
import random
import tempfile
import os
from typing import List

import pytest

from pyredbackup.client import Client
from pyredbackup.configuration import BaseConfiguration
from pyredbackup.helpers import check_log_for_errors
from pyredbackup.node import Node
from pyredbackup.utils import current_version
import docker
import iso8601
LOG = logging.getLogger(__name__)


class PerfTestConfiguration(BaseConfiguration):
    """
    A Performance test configuration for performance testing
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
        clients = []
        for idx in range(1, 5):
            clients.append(Client(f'client{idx}', self.version,
                                  self.network, self.docker))

        return clients


@pytest.fixture()
def performance_configuration():
    """
    Test fixture for py.test to simplify writing tests.
    """
    version = current_version()
    config = PerfTestConfiguration(version)
    config.start_nodes()
    try:
        yield config
    finally:
        LOG.info("Cleaning up nodes...")
        config.stop_nodes()
        config.clean_up()


def generate_random_files_contents(target_directory: str) -> int:
    """
    Generates 1-100 random files in the target directory
    each containing random data of 1MB-100MB.
    Note that this method might override existing files in
    the given directory (if they start with the prefix `file_`)

    Returns the total size of the created files in MB
    """
    total_size_in_mb = 0
    for idx in range(1, 1 + int(random.random() * 100)):
        fname = f'file_{idx}'
        with open(os.path.join(target_directory, fname), 'wb') as hand:
            file_size_in_mb = int(random.random() * 100)
            for _ in range(1, file_size_in_mb):
                hand.write(os.urandom(1024 * 1024))
                hand.flush()
            LOG.info(f'Generated random file {fname} ({file_size_in_mb} MB)')
    return total_size_in_mb


def test_concurrent_backup_performance(
        performance_configuration: PerfTestConfiguration):
    """
    Context: Performance Test Setup (3 Nodes, lots of client)
    Scenarion:
    - all clients create a backup on a random host
    - wait for completion and verify that all clients succeeded
    """

    handles = {}
    for client in performance_configuration.clients:
        # pick a random node
        node = random.choice(performance_configuration.nodes)
        with tempfile.TemporaryDirectory() as tmpdirname:
            LOG.info(f'Generating random backup data for '
                     f'{client.name} in {tmpdirname}')
            generate_random_files_contents(tmpdirname)
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
