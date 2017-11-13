FROM debian:jessie

ADD target/release/redbackup-node-cli /usr/local/bin/redbackup-node

ENTRYPOINT [ "/usr/local/bin/redbackup-node" ]
