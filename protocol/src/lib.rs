extern crate bytes;
extern crate chrono;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

#[macro_use]
extern crate serde_derive;

pub mod message;

use std::io;

pub use message::Message;
pub use message::MessageKind;


use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;
use tokio_proto::pipeline::ClientProto;

pub struct RedServerProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for RedServerProto {
    type Request = Message;
    type Response = Message;
    type Transport = Framed<T, RedCodec>;
    type BindTransport = io::Result<Framed<T, RedCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, RedCodec>> {
        Ok(io.framed(RedCodec))
    }
}
pub struct RedClientProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for RedClientProto {
    type Request = Message;
    type Response = Message;
    type Transport = Framed<T, RedCodec>;
    type BindTransport = io::Result<Framed<T, RedCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, RedCodec>> {
        Ok(io.framed(RedCodec))
    }
}

pub struct RedCodec;

impl Decoder for RedCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Message>> {
        message::decode(buf)
    }
}

impl Encoder for RedCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> io::Result<()> {
        message::encode(msg, buf)
    }
}
