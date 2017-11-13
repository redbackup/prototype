FROM debian:jessie

ADD target/release/redbackup-client-cli /usr/local/bin/redbackup-client

ENTRYPOINT [ "/usr/local/bin/redbackup-client" ]
