FROM debian:jessie

ADD target/release/redbackup-client-cli /usr/bin/redbackup-client

ENTRYPOINT [ "/usr/bin/redbackup-client" ]
