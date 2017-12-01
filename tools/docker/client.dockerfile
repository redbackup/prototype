FROM debian:jessie

RUN set -ex; \
	apt-get update; \
    apt-get install -y --no-install-recommends \
        netcat libsqlite3-0; \
    rm -rf /var/lib/apt/lists/*

ADD target/release/redbackup-client-cli /usr/local/bin/redbackup-client

ENTRYPOINT [ "/usr/local/bin/redbackup-client" ]
