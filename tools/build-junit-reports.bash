#!/usr/bin/env bash
set -ux

REPORTS_DIR="target/reports/"

mkdir -p "$REPORTS_DIR"
cargo test --all ${@:1}

echo "Skipping creating of JUnit reports - since cargo test-junit cannot handle igored tests..."
exit 0

# Loop over all folders in the cwd
for d in */ ; do
    # Skip all directories that do not contain a crate...
    if [ "$d" == "tools/" ] || [ "$d" == "target/" ]; then
        continue
    fi

    # cd into the crate, built the report and move the result into the report directory
    cd "$d"
    cargo test-junit
    mv "${d%/}.xml" ../"$REPORTS_DIR"
    cd ../
done
