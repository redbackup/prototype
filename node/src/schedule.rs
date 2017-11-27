use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time;
use std::time::Duration;

use futures::Future;
use futures_cpupool::CpuPool;
use futures_cpupool::CpuFuture;
use tokio_core;
use tokio_core::reactor::Handle;
use tokio_proto::TcpClient;
use tokio_service::Service;
use tokio_timer::Timer;


use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::*;
use redbackup_storage::Storage;
use chunk_table::ChunkTable;
use chunk_table::DatabaseError;

trait Task {
    fn exec(&self) -> CpuFuture<(), ()>;
    fn name(&self) -> &'static str;
}

struct Schedule {
    task: Arc<Task>,
    handle: Handle,
    timeout: Duration,
    timer: Timer,
}

impl Schedule {
    fn new(handle: Handle, task: Arc<Task>, timeout: Duration) -> Self {
        let timer = Timer::default();
        Schedule {
            task,
            handle,
            timeout,
            timer,
        }
    }

    fn schedule(self) {
        let sleep = self.timer.sleep(self.timeout);
        let handle = self.handle.clone();
        let task = self.task.clone();

        handle.spawn(sleep.then(move |_| task.exec()).then(move |_| {
            debug!("rescheduling task {}...", self.task.name());
            self.schedule();
            Ok(())
        }));
    }
}

pub fn setup(
    handle: Handle,
    chunk_table: ChunkTable,
    storage: Storage,
    known_nodes: Vec<SocketAddr>,
) {
    info!("Setting up replication schedule..");
    let timeout = Duration::from_millis(5000);
    let replication_task = ReplicateTask::new(storage.clone(), chunk_table.clone(), known_nodes);
    Schedule::new(handle.clone(), Arc::new(replication_task), timeout).schedule();

    info!("Setting up integrity check schedule..");
    let timeout = time::Duration::from_millis(20000);
    let integrity_check_task = IntegrityCheckTask::new(storage, chunk_table);
    Schedule::new(handle.clone(), Arc::new(integrity_check_task), timeout).schedule();
}

struct ReplicateTask {
    pool: CpuPool,
    storage: Storage,
    chunk_table: ChunkTable,
    known_nodes: Vec<SocketAddr>,
}

impl ReplicateTask {
    fn new(storage: Storage, chunk_table: ChunkTable, known_nodes: Vec<SocketAddr>) -> Self {
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


struct IntegrityCheckTask {
    pool: CpuPool,
    storage: Storage,
    chunk_table: ChunkTable,
}
impl IntegrityCheckTask {
    fn new(storage: Storage, chunk_table: ChunkTable) -> Self {
        let pool = CpuPool::new(1);
        IntegrityCheckTask {
            storage,
            pool,
            chunk_table,
        }
    }
}
impl Task for IntegrityCheckTask {
    fn exec(&self) -> CpuFuture<(), ()> {
        let chunk_table = self.chunk_table.clone();
        let storage = self.storage.clone();
        self.pool.spawn_fn(move || {
            let db_res = chunk_table.load_random_chunks(5);
            if !db_res.is_err() {
                for chunk in db_res.unwrap() {
                    let vres = storage.verify(&chunk.chunk_identifier);
                    if vres.is_err() {
                        error!("Corruption detected: {}", vres.unwrap_err())
                    } else {
                        debug!(
                            "Integrity check for chunk {} successful",
                            chunk.chunk_identifier
                        );
                    }
                }
            } else {
                error!(
                    "Failed to load sample from db for integrity check: {}",
                    db_res.unwrap_err()
                );
            }
            let res: Result<(), ()> = Ok(());
            res
        })
    }

    fn name(&self) -> &'static str {
        "integrity check"
    }
}
