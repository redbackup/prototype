use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use redbackup_protocol::message::{ChunkContentElement, ChunkElement};

use super::super::chunk_table::Chunk;

pub struct ExampleChunk {}

impl ExampleChunk {
    pub fn one() -> Chunk {
        Self::new(
            "7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2",
            NaiveDate::from_ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10),
            true,
        )
    }
    pub fn two() -> Chunk {
        Self::new(
            "533f05fc11c16e51f700b8f0be6440eea1579886aaa6eb70fba7982fa9043350",
            NaiveDate::from_ymd(2017, 9, 21).and_hms_milli(2, 4, 0, 1),
            true,
        )
    }
    pub fn three() -> Chunk {
        Self::new(
            "18796cf89632a1efd42f42248f2428b94be593980b03a56fe241b552f3f4bb44",
            NaiveDate::from_ymd(2011, 12, 8).and_hms_milli(7, 2, 12, 33),
            true,
        )
    }

    fn new(chunk_identifier: &str, expiration_date: NaiveDateTime, root_handle: bool) -> Chunk {
        Chunk {
            chunk_identifier: String::from(chunk_identifier),
            expiration_date,
            root_handle,
        }
    }
}

pub struct ExampleChunkElement {}

impl ExampleChunkElement {
    pub fn one() -> ChunkElement {
        Self::new(ExampleChunkContentElement::one())
    }
    pub fn two() -> ChunkElement {
        Self::new(ExampleChunkContentElement::two())
    }
    fn new(element: ChunkContentElement) -> ChunkElement {
        ChunkElement {
            chunk_identifier: element.chunk_identifier,
            expiration_date: element.expiration_date,
            root_handle: element.root_handle,
        }
    }
}

pub struct ExampleChunkContentElement {}

impl ExampleChunkContentElement {
    pub fn one() -> ChunkContentElement {
        Self::new(
            "d2a84f4b8b650937ec8f73cd8be2c74add5a911ba64df27458ed8229da804a26",
            NaiveDate::from_ymd(2017, 11, 01).and_hms_milli(4, 9, 12, 49),
            false,
            vec![
                240,
                159,
                144,
                169,
                72,
                101,
                108,
                108,
                111,
                32,
                87,
                111,
                114,
                108,
                100,
            ],
        )
    }

    pub fn two() -> ChunkContentElement {
        Self::new(
            "a090165bdeced34bd9ba8f0aade9ffc8383c8cf7de862d34316e02b4ea2ea5a6",
            NaiveDate::from_ymd(2019, 1, 2).and_hms_milli(9, 2, 1, 5),
            true,
            vec![111, 114, 108, 100],
        )
    }

    fn new(
        chunk_identifier: &str,
        utc_expiration_date: NaiveDateTime,
        root_handle: bool,
        chunk_content: Vec<u8>,
    ) -> ChunkContentElement {
        ChunkContentElement {
            chunk_identifier: String::from(chunk_identifier),
            expiration_date: DateTime::from_utc(utc_expiration_date, Utc),
            root_handle,
            chunk_content,
        }
    }
}
