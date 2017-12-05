"""
Utils for integration testing.
"""
import subprocess

import pytest

from pyredbackup.configuration import MediumConfiguration, MinimalConfiguration


def current_version() -> str:
    """
    Returns the current git version.
    """
    result = subprocess.check_output('git describe --tags', shell=True)
    return result.decode().strip()


@pytest.fixture()
def medium_configuration():
    """
    Test fixture for py.test to simplify writing tests.
    """
    version = current_version()
    config = MediumConfiguration(version)
    config.start_nodes()
    try:
        yield config
    finally:
        config.stop_nodes()

    # Only clean up if everything was successful (for debugging)
    config.clean_up()


@pytest.fixture()
def minimal_configuration():
    """
    Test fixture for py.test to simplify writing tests.
    """
    version = current_version()
    config = MinimalConfiguration(version)
    config.start_nodes()
    try:
        yield config
    finally:
        config.stop_nodes()

    # Only clean up if everything was successful (for debugging)
    config.clean_up()
