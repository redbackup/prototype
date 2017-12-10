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
use std::error::Error;

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
        Ok(io.framed(RedCodec {}))
    }
}
pub struct RedClientProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for RedClientProto {
    type Request = Message;
    type Response = Message;
    type Transport = Framed<T, RedCodec>;
    type BindTransport = io::Result<Framed<T, RedCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, RedCodec>> {
        Ok(io.framed(RedCodec {}))
    }
}

pub struct RedCodec;

impl Decoder for RedCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Message>> {
        let len = buf.len();
        if len == 0 {
            return Ok(None);
        }
        debug!("Start decoding message");
        match decode_message(buf) {
            Err(e) => {
                debug!(
                    "Failed decoding message (description: {}, cause: {:?})",
                    e.description(),
                    e.cause()
                );
                trace!("Buffer content: {:?}", buf);
                Ok(None)
            }
            Ok(m) => {
                if m.is_some() {
                    debug!("Message decoded successfully (to len {})", len);
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
        debug!("Start encoding message");
        trace!("Message: {:?}", msg);
        let m = encode_message(msg, buf);
        debug!("Message encoded successfully");
        m
    }
}

pub fn decode_message(buf: &mut BytesMut) -> io::Result<Option<Message>> {
    let mut de = Deserializer::new(&buf[..]);
    Deserialize::deserialize(&mut de).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

pub fn encode_message(msg: Message, buf: &mut BytesMut) -> io::Result<()> {
    let mut vec = Vec::new();
    msg.serialize(&mut Serializer::new(&mut vec))
        .map(move |_| buf.extend_from_slice(&vec))
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
    fn decode_broken_incomming_message() {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put(&b"\x00"[..]);
        let err = decode_message(&mut buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.description(), "error while decoding value");
    }

    #[test]
    fn test_encode_outgoing_message() {
        let mut buf = BytesMut::with_capacity(1024);
        let msg = Message {
            timestamp: Utc.ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10),
            body: MessageKind::ReturnDesignation(ReturnDesignation { designation: false }),
        };
        encode_message(msg, &mut buf).unwrap();
        // Debug with: https://kawanet.github.io/msgpack-lite/
        let expected = vec![
            146,
            184,
            50,
            48,
            49,
            52,
            45,
            49,
            49,
            45,
            50,
            56,
            84,
            48,
            55,
            58,
            48,
            56,
            58,
            48,
            57,
            46,
            48,
            49,
            48,
            90,
            146,
            1,
            145,
            145,
            194,
        ];
        assert_eq!(buf, expected[..]);
    }

    #[test]
    fn test_encode_incomming_message() {
        let mut buf = BytesMut::with_capacity(1024);
        // Debug with: https://kawanet.github.io/msgpack-lite/
        let raw = vec![
            146,
            184,
            50,
            48,
            49,
            52,
            45,
            49,
            49,
            45,
            50,
            56,
            84,
            48,
            55,
            58,
            48,
            56,
            58,
            48,
            57,
            46,
            48,
            49,
            48,
            90,
            146,
            1,
            145,
            145,
            194,
        ];
        buf.put(raw);

        let actual = decode_message(&mut buf).unwrap().unwrap();
        let expected = Message {
            timestamp: Utc.ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10),
            body: MessageKind::ReturnDesignation(ReturnDesignation { designation: false }),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_incomming_message_with_invalid_contents() {
        let mut buf = BytesMut::with_capacity(1024);
        // Debug with: https://kawanet.github.io/msgpack-lite/
        let raw = vec![
            146,
            184,
            50,
            48,
            49,
            52,
            45,
            49,
            49,
            45,
            50,
            56,
            84,
            48,
            55,
            58,
            48,
            56,
            58,
            48,
            57,
            46,
            48,
            49,
            48,
            90,
            146,
            205,
            4,
            215,
            145,
            145,
            194,
        ];
        buf.put(raw);

        let err = decode_message(&mut buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.description(), "error while decoding value");
    }
}
