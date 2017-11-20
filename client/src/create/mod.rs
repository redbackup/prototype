mod chunk_index_builder;

use std::io;
use std::path::PathBuf;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use futures::Future;
use chrono::prelude::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::GetDesignation;

use super::config::Config;
use super::chunk_index::{ChunkIndex, DatabaseError};
use super::chunk_index::schema::{Folder, NewFolder, File, NewFile, NewChunk};
use self::chunk_index_builder::{ChunkIndexBuilder, BuilderError};

quick_error!{
    #[derive(Debug)]
    pub enum CreateError {
        DatabaseError(err: DatabaseError) {
            from()
            cause(err)
        }
        BuilderError(err: BuilderError) {
            from()
            cause(err)
        }
        IoError(err: io::Error) {
            from()
            cause(err)
        }
    }
}

pub fn run(config: Config, backup_dir: PathBuf) -> Result<(), CreateError> {
    let now = Utc::now();
    let file_name = format!("{}/chunk_index-{}.db",
        config.chunk_index_storage.to_str().unwrap(),
        now.to_rfc3339());
    let chunk_index = ChunkIndex::new(&file_name, now)?;
    info!("Created chunk index {}", file_name);

    let builder = ChunkIndexBuilder::new(&chunk_index, &backup_dir)?;
    builder.build()?;

    let mut event_loop = tokio_core::reactor::Core::new()?;
    let handle = event_loop.handle();

    let test = TcpClient::new(RedClientProto)
        .connect(&config.addr, &handle.clone())
        .and_then(|client| {
            let req = GetDesignation::new(1400, Utc::now());
            println!("req: {:?}", req);
            client.call(req)
    }).map(|res| println!("res: {:?}", res));

    event_loop.run(test).map_err(|e| CreateError::from(e))
}
