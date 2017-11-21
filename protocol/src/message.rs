use chrono::{DateTime, Utc};

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
    GetRootHandles(GetRootHandles),
    ReturnRootHandles(ReturnRootHandles),
    GetChunks(GetChunks),
    ReturnChunks(ReturnChunks),
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

impl Into<ChunkElement> for ChunkContentElement {
    fn into(self) -> ChunkElement{
        ChunkElement {
            chunk_identifier: self.chunk_identifier,
            expiration_date: self.expiration_date,
            root_handle: self.root_handle,
        }
    }
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
pub struct GetRootHandles {
}

impl GetRootHandles {
   pub fn new() -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::GetRootHandles(GetRootHandles {}),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ReturnRootHandles {
    pub root_handle_chunks: Vec<ChunkContentElement>
}

impl ReturnRootHandles {
   pub fn new(root_handle_chunks: Vec<ChunkContentElement>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::ReturnRootHandles(ReturnRootHandles {
                root_handle_chunks
            }),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetChunks {
    pub chunk_identifiers: Vec<String>,
}

impl GetChunks {
   pub fn new(chunk_identifiers: Vec<String>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::GetChunks(GetChunks {
                chunk_identifiers
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ReturnChunks {
    pub chunks: Vec<ChunkContentElement>
}

impl ReturnChunks {
    pub fn new(chunks: Vec<ChunkContentElement>) -> Message {
        Message {
            timestamp: Utc::now(),
            body: MessageKind::ReturnChunks(ReturnChunks {
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
