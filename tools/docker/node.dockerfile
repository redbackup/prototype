FROM debian:jessie

ADD target/release/redbackup-node-cli /usr/bin/redbackup-node

ENTRYPOINT [ "/usr/bin/redbackup-node" ]
