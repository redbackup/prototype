# Integration Test

The integration tests are written in Python and run using docker.

## Requirements

- docker
- python >= 3.6 (including sources ,e.g. `python3-devel`)

## Getting started

```bash
cd tools/tests
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
py.test -s run.py # Run the tests
```
