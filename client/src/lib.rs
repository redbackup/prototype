extern crate redbackup_protocol;
#[macro_use] extern crate quick_error;
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate chrono;

pub mod config;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::GetDesignation;

use std::io;

use tokio_proto::TcpClient;
use futures::Future;
use tokio_service::Service;
use chrono::prelude::*;

pub fn backup(config: config::Config) -> Result<(), io::Error> {
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
}

