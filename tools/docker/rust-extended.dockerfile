FROM rust:1.21.0-jessie


RUN set -ex; \
	apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake \ 
        libsqlite3-dev \
        libssl-dev \
        python3-pip \
        python3-dev \ 
        python3-wheel \ 
        python3-lxml \ 
        python3-jinja2; \
    rm -rf /var/lib/apt/lists/*
    

RUN pip3 install pycobertura
RUN cargo install cargo-tarpaulin --vers 0.5.4
RUN cargo install cargo-test-junit --vers 0.6.1
RUN cargo install diesel_cli --vers 0.16.0 --no-default-features --features "sqlite"

WORKDIR /source/

RUN rm -Rf /usr/local/cargo/registry/

