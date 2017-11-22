extern crate bytes;
extern crate chrono;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

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
        info!("Started decoding message: {:?}", buf);
        let m = decode_message(buf);
        info!("Finished decoding message {:?}", m);
        if let Err(_) = m {
            return Ok(None)
        }
        m
    }
}

impl Encoder for RedCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> io::Result<()> {
        encode_message(msg, buf)
    }
}

pub fn decode_message(buf: &mut BytesMut) -> io::Result<Option<Message>> {
    let len = buf.len();
    if len == 0 {
        Ok(None)
    } else {
        serde_json::from_slice(buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
            .map(|k| {
                buf.split_to(len);
                k
            })
    }
}

pub fn encode_message(msg: Message, buf: &mut BytesMut) -> io::Result<()> {
    serde_json::to_string(&msg)
        .map(|raw| {
            buf.extend(raw.as_bytes())
        })
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::message::ReturnDesignation;
    use chrono::{Utc, TimeZone};
    use bytes::BufMut;
    use std::error::Error;

    #[test]
    fn decode_invalid_json_incomming_message() {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put("{\"just\":\"some\",\"invalid\":\"data\"}");
        let err = decode_message(&mut buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.description(), "JSON error");
    }

    #[test]
    fn decode_broken_incomming_message() {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put(&b"\x00"[..]);
        let err = decode_message(&mut buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.description(), "JSON error");
    }

    #[test]
    fn test_encode_outgoing_message() {
        let mut buf = BytesMut::with_capacity(1024);
        let msg =  Message {
            timestamp: Utc.ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10),
            body: MessageKind::ReturnDesignation(ReturnDesignation {designation: false, }),
        };
        encode_message(msg, &mut buf).unwrap();
        assert_eq!(buf, b"{\"timestamp\":\"2014-11-28T07:08:09.010Z\",\"body\":{\"ReturnDesignation\":{\"designation\":false}}}"[..]);
    }

}
