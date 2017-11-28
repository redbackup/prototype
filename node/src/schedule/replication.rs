use std::io;
use std::net::SocketAddr;

use futures::Future;
use futures_cpupool::{CpuFuture, CpuPool};
use tokio_core;
use tokio_core::reactor::Core;
use tokio_proto::TcpClient;
use tokio_service::Service;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::*;
use redbackup_storage::Storage;
use chunk_table::{Chunk, ChunkTable, DatabaseError};

use super::Task;
use super::super::utils;

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
            info!("begin with replication");
            replicate(chunk_table, storage, known_nodes).map_err(|e| {
                error!("replication has failed with a problem: {}", e);
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
    pub enum ReplicationError {
        NodeCommunicationError
        DatabaseError(err: DatabaseError) {
            from()
            display("DatabaseError: {}", err)
            cause(err)
        }
        IoError(err: io::Error) {
            from()
            display("I/O error: {}", err)
            cause(err)
        }
        MessageSendProblem(err: io::Error, peer: SocketAddr) {
            display("An io problem occured when communicating with {}: {}", peer, err)
            cause(err)
        }
        ChunkNotAcknowledged(chunk_identifier: String) {
            description("Chunk was not acknowledged")
            display("The Chunk {} was not acknowledged by the node", chunk_identifier)
        }
        WrongChunkAcknowledged(expected: String, actual: String) {
            description("The wrong Chunk was acknowledged")
            display("Expected chunk {} acknowledgement, got {}", expected, actual)
        }

    }
}


fn replicate(
    chunk_table: ChunkTable,
    storage: Storage,
    known_nodes: Vec<SocketAddr>,
) -> Result<(), ReplicationError> {
    info!("Loading chunks to replicate...");
    let chunks = chunk_table.load_random_chunks(5)?;
    debug!("Loading chunks: {:?}", chunks);

    let mut event_loop = tokio_core::reactor::Core::new()?;
    let chunk_elements: Vec<_> = chunks.clone().into_iter().map(|c| c.into()).collect();

    if chunk_elements.len() == 0 {
        info!("No chunks to replicate");
        return Ok(());
    }

    debug!(
        "Replicating selected chunks to {} nodes...",
        known_nodes.len()
    );
    for node_addr in known_nodes {
        debug!("Replicating selected chunks to node {}", node_addr);

        let node_chunks =
            get_available_chunks_from_node(chunk_elements.clone(), &node_addr, &mut event_loop)?;
        let mut missing_chunks = chunks.clone();
        info!(
            "{} of total {} chunks are already present on node {}",
            node_chunks.len(),
            missing_chunks.len(),
            node_addr
        );
        reduce_by_remaining_chunks(&mut missing_chunks, &node_chunks);

        debug!("missing_chunks: {:?}", missing_chunks);

        for chunk in missing_chunks {
            send_chunk_to_node(chunk, &storage, &node_addr, &mut event_loop)?;
        }
    }
    info!("Replication completed successfully");
    Ok(())
}

fn get_available_chunks_from_node(
    chunk_elements: Vec<ChunkElement>,
    node_addr: &SocketAddr,
    event_loop: &mut Core,
) -> Result<Vec<ChunkElement>, ReplicationError> {
    let req = GetChunkStates::new(chunk_elements);
    message_node_sync(req, node_addr, event_loop).map(|res| match res.body {
        MessageKind::ReturnChunkStates(body) => Ok(body.chunks),
        _ => Err(ReplicationError::NodeCommunicationError),
    })?
}

fn send_chunk_to_node(
    chunk: Chunk,
    storage: &Storage,
    node_addr: &SocketAddr,
    event_loop: &mut Core,
) -> Result<(), ReplicationError> {
    let chunk_identifier = chunk.chunk_identifier.clone();
    debug!("Sending missing chunk {} to node {}", node_addr, node_addr);
    let chunk = utils::chunk_to_chunk_contents_element(chunk, storage).unwrap();
    let req = PostChunks::new(vec![chunk]);
    let acknowledged_chunks =
        message_node_sync(req, node_addr, event_loop).map(|res| match res.body {
            MessageKind::AcknowledgeChunks(body) => Ok(body.chunks),
            _ => Err(ReplicationError::NodeCommunicationError),
        })??;

    let acknowledged_chunk: &ChunkElement = acknowledged_chunks.get(0).ok_or(
        ReplicationError::ChunkNotAcknowledged(chunk_identifier.clone()),
    )?;

    if acknowledged_chunk.chunk_identifier != chunk_identifier {
        return Err(ReplicationError::WrongChunkAcknowledged(
            chunk_identifier.clone(),
            acknowledged_chunk.chunk_identifier.clone(),
        ));
    }
    debug!(
        "Chunk {} is now replicated",
        acknowledged_chunk.chunk_identifier
    );
    Ok(())
}

fn reduce_by_remaining_chunks(elements: &mut Vec<Chunk>, reduction: &Vec<ChunkElement>) {
    elements.retain(|e| {
        reduction
            .iter()
            .filter(|x| e.chunk_identifier == x.chunk_identifier)
            .count() == 0
    });
}

fn message_node_sync(
    message: Message,
    peer_addr: &SocketAddr,
    event_loop: &mut Core,
) -> Result<Message, ReplicationError> {
    let handle = event_loop.handle();
    let future = TcpClient::new(RedClientProto)
        .connect(&peer_addr, &handle)
        .and_then(|client| client.call(message));
    event_loop.run(future).map_err(|e| {
        ReplicationError::MessageSendProblem(e, peer_addr.clone())
    })
}
