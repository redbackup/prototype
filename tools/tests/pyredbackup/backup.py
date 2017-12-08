"""
This module contains an abstraction of a "Backup" that is stored on a node
a.k.a. a Root handle
"""


class Backup:
    """
    abstraction of a "Backup" that is stored on a node a.k.a. a Root handle
    """

    def __init__(self, backup_id: str, expiration_date: str) -> None:
        self.backup_id = backup_id
        self.expiration_date = expiration_date
