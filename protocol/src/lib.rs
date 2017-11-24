extern crate bytes;
extern crate chrono;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate serde;
extern crate serde_bytes;
extern crate rmp_serde as rmps;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

pub mod message;

use std::io;

pub use message::Message;
pub use message::MessageKind;


use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;
use tokio_proto::pipeline::ClientProto;
use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};

pub struct RedServerProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for RedServerProto {
    type Request = Message;
    type Response = Message;
    type Transport = Framed<T, RedCodec>;
    type BindTransport = io::Result<Framed<T, RedCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, RedCodec>> {
        Ok(io.framed(RedCodec{}))
    }
}
pub struct RedClientProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for RedClientProto {
    type Request = Message;
    type Response = Message;
    type Transport = Framed<T, RedCodec>;
    type BindTransport = io::Result<Framed<T, RedCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, RedCodec>> {
        Ok(io.framed(RedCodec{}))
    }
}

pub struct RedCodec;

impl Decoder for RedCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Message>> {
        let len = buf.len();
        if len == 0 {
            return Ok(None)
        }
        debug!("Started decoding message");
        match decode_message(buf) {
            Err(e) => {
                debug!("Failed decoding message ({:?})", e);
                Ok(None)
            },
            Ok(m) => {
                if m.is_some() {
                    debug!("Message decoded");
                    buf.split_to(len);
                }
                Ok(m)
            }
        }
    }
}

impl Encoder for RedCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> io::Result<()> {
        debug!("Started encoding message");
        let m = encode_message(msg, buf);
        debug!("message encoded");
        m
    }
}

pub fn decode_message(buf: &mut BytesMut) -> io::Result<Option<Message>> {
        let mut de = Deserializer::new(&buf[..]);
        Deserialize::deserialize(&mut de)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

pub fn encode_message(msg: Message, buf: &mut BytesMut) -> io::Result<()> {
    let mut vec = Vec::new();
    msg.serialize(&mut Serializer::new(&mut vec))
        .map(move |_|buf.extend_from_slice(&vec))
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}
