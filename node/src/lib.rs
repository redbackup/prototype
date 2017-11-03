#[macro_use] extern crate quick_error;
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate chrono;

extern crate redbackup_protocol;

pub mod config;
mod service;

use tokio_proto::TcpServer;

use config::Config;
use redbackup_protocol::RedServerProto;


pub fn run(config: Config) {
    TcpServer::new(RedServerProto, config.addr)
        .serve(|| Ok(service::NodeService));
}
