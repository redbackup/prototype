use std::io;
use std::net::SocketAddr;

use futures::Future;
use futures_cpupool::CpuPool;
use futures_cpupool::CpuFuture;
use tokio_core;
use tokio_proto::TcpClient;
use tokio_service::Service;


use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::*;
use redbackup_storage::Storage;
use chunk_table::ChunkTable;
use chunk_table::DatabaseError;

use super::Task;

pub struct ReplicateTask {
    pool: CpuPool,
    storage: Storage,
    chunk_table: ChunkTable,
    known_nodes: Vec<SocketAddr>,
}

impl ReplicateTask {
    pub fn new(storage: Storage, chunk_table: ChunkTable, known_nodes: Vec<SocketAddr>) -> Self {
        let pool = CpuPool::new(1);
        ReplicateTask {
            storage,
            pool,
            chunk_table,
            known_nodes,
        }
    }
}

impl Task for ReplicateTask {
    fn exec(&self) -> CpuFuture<(), ()> {
        let chunk_table = self.chunk_table.clone();
        let storage = self.storage.clone();
        let known_nodes = self.known_nodes.clone();

        self.pool.spawn_fn(move || {
            replicate(chunk_table, storage, known_nodes).map_err(|e| {
                error!("{}", e);
                ()
            })
        })
    }
    fn name(&self) -> &'static str {
        "replicate"
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum TODOError {
        NodeCommunicationError
        DatabaseError(err: DatabaseError) {
            from()
            cause(err)
        }
        IoError(err: io::Error) {
            from()
            cause(err)
        }

    }
}


fn replicate(
    chunk_table: ChunkTable,
    storage: Storage,
    known_nodes: Vec<SocketAddr>,
) -> Result<(), TODOError> {
    let chunks = chunk_table.load_random_chunks(5)?;

    let mut event_loop = tokio_core::reactor::Core::new()?;
    let handle = event_loop.handle();

    let chunk_elements: Vec<_> = chunks.into_iter().map(|c| c.into()).collect();

    for node_addr in known_nodes {
        // TODO: extract & handle properly...
        let req = GetChunkStates::new(chunk_elements.clone());
        let future = TcpClient::new(RedClientProto)
            .connect(&node_addr, &handle)
            .and_then(|client| client.call(req));
        let chunks = event_loop
            .run(future)
            .map(|res| match res.body {
                MessageKind::ReturnChunkStates(body) => Ok(body.chunks),
                _ => Err(TODOError::NodeCommunicationError),
            })
            .unwrap()
            .unwrap();
        let missing_chunks = chunk_elements.clone().retain(|e| {
            chunks
                .iter()
                .filter(|x| e.chunk_identifier == x.chunk_identifier)
                .count() == 0
        });
    }
    Ok(())
}

