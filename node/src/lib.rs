#![recursion_limit="128"] // required for database inference
#[macro_use] extern crate quick_error;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_proto;
extern crate tokio_service;
extern crate chrono;
extern crate r2d2;
extern crate r2d2_diesel;


extern crate redbackup_protocol;

pub mod config;
mod service;
mod chunk_table;

use tokio_proto::TcpServer;

use config::Config;
use redbackup_protocol::RedServerProto;
use service::NodeService;

pub fn run(config: Config) {
    TcpServer::new(RedServerProto, config.addr)
        .serve(|| Ok(NodeService::new()));
}
