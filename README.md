# Redbackup Prototype

This is a prototype of the [redbackup system](https://www.redbackup.org/) written in [Rust](http://rust-lang.org/).

## License

This is free software, published under the [AGPL-License](https://www.gnu.org/licenses/agpl-3.0.en.html).

## Trying it out

We build docker images on every push. This allows you to test the prototype without a complex setup. All you need is [docker](https://www.docker.com/).

Please note that some basic understanding of docker is required to proceed with the commands below.

```bash
# Get the latest git tag version from https://git.redbackup.org/projects/RED/repos/prototype
VERSION=0.11.0

# Firstly, we create a dedicated Network
docker network create redbackup-demo

# Launch a node in the background
docker run --name Node --network redbackup-demo --rm -d -e RUST_LOG=redbackup=debug redbackup/node:$VERSION

# Copy some sample data into a temporary directory
cp -r /some/data/to/backup back-me-up/

# Lets back it up
docker run --name Client --network redbackup-demo --rm -v "$(pwd)/back-me-up":/data:z -e RUST_LOG=redbackup=warn redbackup/client:$VERSION -h Node create 2018-04-12T17:49 /data/

# And lets restore it to another place...
# Fist, we need to list all available backups:
docker run --name Client --network redbackup-demo --rm -e RUST_LOG=redbackup=warn redbackup/client:$VERSION -h Node list

# Next, pick the first (and only) Backup-ID from the output and replace BACKUP_ID in the next command with its value
docker run --name Client --network redbackup-demo --rm -v "$(pwd)/restore-me":/data:z -e RUST_LOG=redbackup=warn redbackup/client:$VERSION -h Node restore BACKUP_ID /data/

# All data is now restored in the directory "./restore-me"

# Shut it down...
docker kill Node
```

## Building and testing

Firstly, [install latest stable rust](https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html)

Make sure you also have `libsqlite3` and `libsqlite3-dev` (or similar) installed.
If you want to run the integration tests, also install `python3` and `python3-dev`.

```bash
$ git clone ssh://git@git.redbackup.org:40001/red/prototype.git
$ cd prototype/
$ cargo build --all
```

The project ist structured in the form of a [cargo workspace](https://github.com/rust-lang/rfcs/blob/master/text/1525-cargo-workspace.md), which means that you can build and tests all modules at once in the project root using the `--all` parameter.

## Installation

You can deploy either a docker image in a docker infrastructure or install the binaries directly.
Note that we currently build images for Linux 64 bit only. If you need another plattform, you have to build
the project manually (see section above).

Make sure you also have `libsqlite3` installed (e.g. `apt-get install libsqlite3`)

You can download the following precompiled binaries (called `binary-release-linux-x86_64`) [from the CI Server](https://ci.redbackup.org/browse/RED-REDPRO/latestSuccessful/artifact):

- redbackup-client-cli
- redbackup-node-cli

You can place the binary where you want but we recommend to place them in `/usr/local/bin`. Don't forget to make them executable.

```bash
cp redbackup-client-cli /usr/local/bin
cp redbackup-node-cli /usr/local/bin
chmod a+x /usr/local/bin/redbackup-client-cli
chmod a+x /usr/local/bin/redbackup-node-cli
```

That's it. You can now launch the application.

```
redbackup-client-cli --help
redbackup-node-cli --help
```

## Development Guidelines

The code must follow the most recent [Rust RFCs](https://aturon.github.io/) (naming conventions etc.)

Code must be formatted using [rustfmt](https://github.com/rust-lang-nursery/rustfmt).

## Plans for the future

Because this project is just a prototype, there is still a lot to do and we are far from future complete.

The following sections document ideas for further improvements.
