FROM debian:jessie

RUN set -ex; \
	apt-get update; \
    apt-get install -y --no-install-recommends \
        netcat libsqlite3-0; \
    rm -rf /var/lib/apt/lists/*

ADD target/release/redbackup-node-cli /usr/local/bin/redbackup-node

ENTRYPOINT [ "/usr/local/bin/redbackup-node" ]
