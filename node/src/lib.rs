#![recursion_limit = "128"] // required for database inference
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate quick_error;

extern crate chrono;
extern crate futures;
extern crate futures_cpupool;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate tokio_proto;
extern crate tokio_service;


extern crate redbackup_protocol;

pub mod config;
mod service;
mod chunk_table;

use tokio_proto::TcpServer;
use futures_cpupool::CpuPool;


use config::Config;
use redbackup_protocol::RedServerProto;
use service::NodeService;
use chunk_table::ChunkTable;

pub fn run(config: Config) {
    let chunk_table = ChunkTable::new("demo.sqlite3").unwrap();
    let cpu_pool = CpuPool::new_num_cpus();

    TcpServer::new(RedServerProto, config.addr).serve(move || {
        Ok(NodeService::new(cpu_pool.clone(), chunk_table.clone()))
    });
}
