use futures::future;
use futures::Future;
use tokio_service::Service;
use futures_cpupool::CpuPool;

use redbackup_protocol::{Message, MessageKind};
use redbackup_protocol::message::GetDesignation;
use redbackup_protocol::message::ReturnDesignation;
use redbackup_protocol::message::GetChunkStates;
use redbackup_protocol::message::ReturnChunkStates;
use redbackup_protocol::message::InvalidRequest;

use std::io;
#[cfg(test)]
mod tests;

pub struct NodeService{
    pool: CpuPool
}

impl NodeService {
    pub fn new() -> NodeService {
        NodeService {
            pool: CpuPool::new_num_cpus(),
        }
    }
    pub fn pool(&self) -> &CpuPool {
        &self.pool
    }
}

impl Service for NodeService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = io::Error>>; //future::Ok<Message, io::Error>;

    fn call(&self, request: Message) -> Self::Future {
        match request.body {
            MessageKind::GetDesignation(ref body) => handle_designation(self.pool(), &body),
            MessageKind::GetChunkStates(ref body) => handle_get_chunks(self.pool(), &body),
            _ => handle_unknown(),
        }
    }
}


fn handle_unknown() -> Box<Future<Item=Message, Error=io::Error>> {
    Box::new(future::ok(InvalidRequest::new("Node cannot handle this message kind")))
}

fn handle_designation(pool: &CpuPool, body: &GetDesignation) -> Box<Future<Item=Message, Error=io::Error>> {
    Box::new(future::ok(ReturnDesignation::new(true)))
}

fn handle_get_chunks(pool: &CpuPool, body: &GetChunkStates) -> Box<Future<Item=Message, Error=io::Error>> {
    Box::new(pool.spawn_fn(|| {
        // TODO: long running operation...
        Ok(ReturnChunkStates::new(Vec::new()))
    }))
}
