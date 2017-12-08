#!/usr/bin/env bash
set -ex

# cd into the directory, in which this script is stored
cd "$(dirname "$0")"
cd tests/
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
py.test run.py # Run the tests

