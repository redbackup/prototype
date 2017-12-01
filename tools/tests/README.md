# Integration Test

The integration tests are written in Python and run using docker.

## Requirements

- docker
- python >= 3.6

## Getting started

```bash
cd toos/tests
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
py.test -s run.py # Run the tests
```