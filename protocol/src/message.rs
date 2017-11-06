use std::io;

use bytes::{BufMut, BytesMut};
use serde_json;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub timestamp: DateTime<Utc>,
    pub body: MessageKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageKind {
    GetDesignation(GetDesignation),
    ReturnDesignation(ReturnDesignation),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDesignation {
    estimate_size: u64,
    expiration_date: DateTime<Utc>,
}

impl GetDesignation {
    pub fn new(estimate_size: u32, expiration_date: DateTime<Utc>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::GetDesignation(GetDesignation {
                estimate_size,
                expiration_date,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReturnDesignation {
    designation: bool,
}

impl ReturnDesignation {
    pub fn new(designation: bool) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::ReturnDesignation(ReturnDesignation {
                designation,
            }),
        }
    }
}

pub fn decode(buf: &mut BytesMut) -> io::Result<Option<Message>> {
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

pub fn encode(msg: Message, buf: &mut BytesMut) -> io::Result<()> {
    serde_json::to_string(&msg)
        .map(|raw| {
            push(buf, raw.as_bytes());
            ()
        })
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

fn push(buf: &mut BytesMut, data: &[u8]) {
    buf.reserve(data.len());
    unsafe {
        buf.bytes_mut()[..data.len()].copy_from_slice(data);
        buf.advance_mut(data.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::BufMut;
    use std::error::Error;

    #[test]
    fn decode_invalid_json_incomming_message() {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put("{\"just\":\"some\",\"invalid\":\"data\"}");
        let err = decode(&mut buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.description(), "JSON error");
    }

    #[test]
    fn decode_broken_incomming_message() {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put(&b"\x00"[..]);
        let err = decode(&mut buf).unwrap_err();
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
        encode(msg, &mut buf).unwrap();
        assert_eq!(buf, b"{\"timestamp\":\"2014-11-28T07:08:09.010Z\",\"body\":{\"ReturnDesignation\":{\"designation\":false}}}"[..]);
    }

}
