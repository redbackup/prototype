#![recursion_limit="128"] // required for database inference
#[macro_use] extern crate quick_error;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate chrono;
extern crate uuid;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dns_lookup;
extern crate sha2;

extern crate redbackup_protocol;

pub mod config;
mod chunk_index;
mod chunk_index_builder;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::GetDesignation;

use std::io;
use std::path::PathBuf;

use tokio_proto::TcpClient;
use futures::Future;
use tokio_service::Service;
use chrono::prelude::*;

pub fn backup(config: config::Config, backup_dir: PathBuf) -> Result<(), io::Error> {

    let _chunk_index = chunk_index_builder::build(&config, backup_dir)
        .expect("Could not build chunk index!");


    /*
    let mut event_loop = tokio_core::reactor::Core::new()?;
    let handle = event_loop.handle();

    let test = TcpClient::new(RedClientProto)
        .connect(&config.addr, &handle.clone())
        .and_then(|client| {
            let req = GetDesignation::new(1400, Utc::now());
            println!("req: {:?}", req);
            client.call(req)
    }).map(|res| println!("res: {:?}", res));

    event_loop.run(test)
    */
    Ok(())
}
