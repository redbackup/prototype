use futures::future;
use redbackup_protocol::Message;
use redbackup_protocol::message::ReturnDesignation;
use tokio_service::Service;

use std::io;

pub struct NodeService;

impl Service for NodeService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = future::Ok<Message, io::Error>;

    fn call(&self, _request: Message) -> Self::Future {
        let resp = ReturnDesignation::new(false);
        future::ok(resp)
    }
}