"""
This module contains an implementation to controll a redbackup client in a
docker container.
"""
import filecmp
import logging
import os
import tarfile
import tempfile
from contextlib import contextmanager
from io import BytesIO
from typing import List

from docker.client import DockerClient
from docker.models.containers import Container
from docker.models.networks import Network

from pyredbackup.backup import Backup
from pyredbackup.node import Node

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
        self.version = version
        LOG.debug(f"Client {self.name} initialized")

    def backup(self, directory_to_send: str, expiration_date: str, node: Node):
        """
        Backs up the given (local) directory onto the given node with the
        provided expiration_date.
        This method asserts for errors during the backup proces.
        """
        command = f'/usr/local/bin/redbackup-client -h {node.name} '\
            f'create {expiration_date} /{directory_to_send}'

        with self._run_sync(command, local_path=directory_to_send,
                            container_path='/') as (container, exit_code):
            assert exit_code == 0
            Client.check_log_for_errors(container)

    @staticmethod
    def check_log_for_errors(container: Container):
        """
        Checks all lines in the given containers log for errors.
        If one is found, an exception is thrown.
        """
        problems = []
        for line in container.logs().decode().split('\n'):
            if line.startswith('ERROR') or line.startswith('WARN'):
                problems.append(line)

        if len(problems) != 0:
            print("Problems found during execution:")
            for problem in problems:
                print(problem)
                raise Exception("Problems found during execution!")

    def restore(self, backup_id: str, expected_directory: str, node: Node):
        """
        Restores the backup with the given hash from the provided node
        and compares it with the given expected_directory.
        """
        restore_to = '/restore-dir'
        command = f'bash -c "mkdir {restore_to};'\
            f'/usr/local/bin/redbackup-client '\
            f'-h {node.name} restore {backup_id} {restore_to}"'

        with self._run_sync(command) as (container, exit_code):
            assert exit_code == 0
            Client.check_log_for_errors(container)
            with Client._copy_from_container(container, restore_to) as d:
                restored = os.path.join(
                    d, restore_to[1:], os.path.split(expected_directory)[1])
                LOG.info('Comparing directory contents of '
                         f'{restored} and {expected_directory}')
                assert are_dir_trees_equal(restored, expected_directory), \
                    'Not all files were restored properly'

    def list_backups(self, node: Node) -> List[Backup]:
        """
        Returns a unordered list of Backups present on the given node.
        and compares it with the given expected_directory.
        """
        command = f'/usr/local/bin/redbackup-client -h {node.name} list'

        backups = []

        with self._run_sync(command, env=[]) as (container, exit_code):
            output = container.logs(stdout=True, stderr=False,
                                    stream=False, timestamps=False).decode()
            assert exit_code == 0
            for line in output.split('\n')[1:]:
                line = line.strip()
                if line != '':
                    (backup_id, expiration_date) = line.split(' ', 1)
                    backups.append(Backup(backup_id, expiration_date))

            self.check_log_for_errors(container)

        return backups

    @contextmanager
    def _run_sync(self, command: str, env=None, container_path=None,
                  local_path=None):
        """
        Runs the given command on the client and blocks until completion.

        env is a list of environment variables, e.g. ["X=Y", "ZY=Q"]
        """

        env = env or ["RUST_BACKTRACE=1", "RUST_LOG=redbackup=debug"]
        LOG.debug(f'Running command {command} on client {self.name}')
        container = self.docker.containers.create(
            self.image,
            entrypoint=command,
            name=self.name,
            hostname=self.name,
            environment=env)
        try:
            self.network.connect(container)
            Client._copy_into_container(container, container_path, local_path)
            LOG.debug(f'Starting client container {self.name}...')
            container.start()
            LOG.debug(
                f'Waiting for completion of client container {self.name}...')
            exit_code = container.wait()
            LOG.debug(
                f'Client container {self.name} has completed with '
                f'exit-code {exit_code}...')
            LOG.debug(f'Tagging container {self.name} ...')
            self.image = container.commit()  # .short_id()
            yield (container, exit_code)
        finally:
            container.remove()

    @staticmethod
    def _copy_into_container(container, container_dir=None, local_dir=None):
        """
        Copies the given local directory into the given container
        at container_dir.
        """
        if not container_dir or not local_dir:
            return

        LOG.debug(
            f"copy local {local_dir} into container to {container_dir}")
        tarstream = BytesIO()
        tar = tarfile.open(fileobj=tarstream, mode='w')
        tar.add(local_dir)
        tar.close()
        tarstream.seek(os.SEEK_SET)
        container.put_archive(container_dir, tarstream)

    @staticmethod
    @contextmanager
    def _copy_from_container(container, container_dir):
        """
        Copies files provided in data from the given container into a
        local directory - yielding it when using it wiht `with`
        """
        data = container.get_archive(container_dir)[0]
        with tempfile.TemporaryDirectory() as local_dir:
            LOG.debug(
                f'copy from container {container_dir} to '
                f'local directory: {local_dir}')
            with tempfile.TemporaryFile() as tmp:
                for chunck in data.stream():
                    tmp.write(chunck)
                tmp.seek(os.SEEK_SET)
                tar = tarfile.open(fileobj=tmp, mode='r')
                tar.extractall(path=local_dir)
                yield local_dir


def are_dir_trees_equal(dir1, dir2):
    """
    Compare two directories recursively. Files in each directory are
    assumed to be equal if their names and contents are equal.

    @param dir1: First directory path
    @param dir2: Second directory path

    @return: True if the directory trees are the same and 
        there were no errors while accessing the directories or files, 
        False otherwise.
    Source: https://stackoverflow.com/questions/4187564/recursive-dircmp-compare-two-directories-to-ensure-they-have-the-same-files-and
   """

    dirs_cmp = filecmp.dircmp(dir1, dir2)
    if len(dirs_cmp.left_only) > 0 or len(dirs_cmp.right_only) > 0 or \
            len(dirs_cmp.funny_files) > 0:
        LOG.error(f'Left only: {dirs_cmp.left_only}')
        LOG.error(f'Left only: {dirs_cmp.right_only}')
        LOG.error(f'Funny Files: {dirs_cmp.funny_files}')
        return False
    (_, mismatch, errors) = filecmp.cmpfiles(
        dir1, dir2, dirs_cmp.common_files, shallow=False)
    if len(mismatch) > 0 or len(errors) > 0:
        LOG.error(f'mismatch: {mismatch}')
        LOG.error(f'errors: {errors}')
        return False
    for common_dir in dirs_cmp.common_dirs:
        new_dir1 = os.path.join(dir1, common_dir)
        new_dir2 = os.path.join(dir2, common_dir)
        LOG.info(f'Calling comparison recursively: "{new_dir1}" "{new_dir2}"')
        if not are_dir_trees_equal(new_dir1, new_dir2):
            return False
    return True
