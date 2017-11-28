#![recursion_limit = "128"] // required for database inference
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate log;
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
extern crate redbackup_storage;

pub mod config;
mod service;
mod chunk_table;

#[cfg(test)]
mod tests;

use tokio_proto::TcpServer;
use futures_cpupool::CpuPool;

use redbackup_protocol::RedServerProto;
use redbackup_storage::Storage;

use config::Config;
use service::NodeService;
use chunk_table::ChunkTable;

pub fn run(config: Config) {
    let chunk_table = ChunkTable::new(&config.db_location).unwrap();
    let cpu_pool = CpuPool::new_num_cpus();
    let storage = Storage::new(config.storage_location).unwrap();
    info!("Start Server");
    TcpServer::new(RedServerProto, config.addr).serve(move || {
        Ok(NodeService::new(
            cpu_pool.clone(),
            chunk_table.clone(),
            storage.clone(),
        ))
    });
}
