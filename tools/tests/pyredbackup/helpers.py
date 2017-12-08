"""
Shared helper methods to repvent duplicated code.
"""
from docker.models.containers import Container


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
