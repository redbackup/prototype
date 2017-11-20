use std::io;

use bytes::{BufMut, BytesMut};
use serde_json;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Message {
    pub timestamp: DateTime<Utc>,
    pub body: MessageKind,
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageKind {
    GetDesignation(GetDesignation),
    ReturnDesignation(ReturnDesignation),
    InvalidRequest(InvalidRequest),
    InternalError(InternalError),
    GetChunkStates(GetChunkStates),
    ReturnChunkStates(ReturnChunkStates),
    PostChunks(PostChunks),
    AcknowledgeChunks(AcknowledgeChunks),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetDesignation {
    pub estimate_size: u64,
    pub expiration_date: DateTime<Utc>,
}

impl GetDesignation {
    pub fn new(estimate_size: u64, expiration_date: DateTime<Utc>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::GetDesignation(GetDesignation {
                estimate_size,
                expiration_date,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ReturnDesignation {
    pub designation: bool,
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct InvalidRequest {
    pub reason: String,
}

impl InvalidRequest {
    pub fn new(reason: &str) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::InvalidRequest(InvalidRequest{
                reason: reason.into()
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct InternalError {
    pub reason: String,
}

impl InternalError {
    pub fn new(reason: &str) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::InternalError(InternalError{
                reason: reason.into()
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ChunkElement {
    pub chunk_identifier: String,
    pub expiration_date: DateTime<Utc>,
    pub root_handle: bool,
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetChunkStates {
    pub chunks: Vec<ChunkElement>
}

impl GetChunkStates {
    pub fn new(chunks: Vec<ChunkElement>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::GetChunkStates(GetChunkStates {
                chunks
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ReturnChunkStates {
    pub chunks: Vec<ChunkElement>
}

impl ReturnChunkStates {
   pub fn new(chunks: Vec<ChunkElement>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::ReturnChunkStates(ReturnChunkStates {
                chunks
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ChunkContentElement {
    pub chunk_identifier: String,
    pub expiration_date: DateTime<Utc>,
    pub root_handle: bool,
    pub chunk_content: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PostChunks {
    pub chunks: Vec<ChunkContentElement>
}

impl PostChunks {
   pub fn new(chunks: Vec<ChunkContentElement>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::PostChunks(PostChunks {
                chunks
            }),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AcknowledgeChunks {
    pub chunks: Vec<ChunkElement>
}

impl AcknowledgeChunks {
   pub fn new(chunks: Vec<ChunkElement>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::AcknowledgeChunks(AcknowledgeChunks {
                chunks
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
            buf.put(raw.as_bytes())
        })
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
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
