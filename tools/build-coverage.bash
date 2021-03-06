#!/usr/bin/env bash
set -eux
export LC_ALL=C.UTF-8
export LANG=C.UTF-8

cargo tarpaulin -v -o Xml
mkdir -p target/cov/
pycobertura show --format html --output target/cov/coverage.html cobertura.xml
mv cobertura.xml target/cov/