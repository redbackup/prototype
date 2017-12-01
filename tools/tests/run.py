"""
Integration tests for redbackup.
"""

from pyredbackup.utils import medium_configuration


def test_capturing_unicode(medium_configuration):
    # TODO: Build assert_tools
    # config.client_1.backup('dir/', node1)
    # config.client_1.list_backups(node1)
    # config.client_1.restore_backup('hash', 'target/', node1)
    print(medium_configuration)
    # node.wait_until_chunk_is_replicated('x')
    # node.export_cache_to('dir/')
    # print(config.client_1.container.logs())
