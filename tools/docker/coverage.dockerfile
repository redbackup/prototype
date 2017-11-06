FROM rust:1.21.0-jessie


RUN set -ex; \
	apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake \ 
        python3-pip \
        python3-dev \ 
        python3-wheel \ 
        python3-lxml \ 
        python3-jinja2; \
    rm -rf /var/lib/apt/lists/*
    

RUN pip3 install pycobertura
RUN cargo install cargo-tarpaulin --vers 0.5.4
ADD tools/build-coverage.bash /usr/bin/build-coverage

WORKDIR /source/

RUN rm -Rf /usr/local/cargo/registry/

ENTRYPOINT [ "/usr/bin/build-coverage" ]

# cargo tarpaulin -v -o Xml
# Turn the XML report into shiny HTML
# pycobertura show --format html --output target/cov/coverage.html target/cov/cobertura.xml