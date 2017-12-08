"""
Integration tests for redbackup.
"""

from pyredbackup.utils import minimal_configuration, medium_configuration
import time


def test_backup_and_restore(minimal_configuration):
    """
    Context: Minimal Test Setup (2 Nodes, 1 client)
    Scenarion:
    - create a backup
    - store it on node a
    - restore it from node a
    """
    client = minimal_configuration.client_1
    node = minimal_configuration.node_a
    client.backup('redbackup-test-data', '2099-04-12T17:49', node)
    backups = client.list_backups(node)
    assert len(backups) == 1
    assert backups[0].expiration_date == '2099-04-12 17:49:00 UTC'
    client.restore(backups[0].backup_id, 'redbackup-test-data', node)


def test_replicated_backup_and_restore(medium_configuration):
    """
    Context: Medium Test Setup (3 Nodes, 3 client)
    Scenarion:
    - create a backup from client 1
    - store it on node A
    - restore it from node B on client 2
    """
    client = medium_configuration.client_1
    node = medium_configuration.node_a
    client.backup('redbackup-test-data', '2099-04-12T17:49', node)

    # As for the study project, 5 chunks are replicated
    # every 30 seconds. If we therefore wait for this amount of time (plus
    # a little bit of extra for the transmission) and the number
    # of files on our backup is less than 5 we must be able to restore
    # the files from another node afterwards
    #
    # it also does not matter which client we use...
    print("Waiting for replication...")
    time.sleep(35)

    client = medium_configuration.client_2
    node = medium_configuration.node_b

    backups = client.list_backups(node)
    assert len(backups) == 1
    assert backups[0].expiration_date == '2099-04-12 17:49:00 UTC'
    client.restore(backups[0].backup_id, 'redbackup-test-data', node)

    # Same thing using client 3 and node c
    client = medium_configuration.client_3
    node = medium_configuration.node_c

    backups = client.list_backups(node)
    assert len(backups) == 1
    assert backups[0].expiration_date == '2099-04-12 17:49:00 UTC'
    client.restore(backups[0].backup_id, 'redbackup-test-data', node)
