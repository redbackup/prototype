use std::io;
use chunk_index::DatabaseError;


quick_error!{
    #[derive(Debug)]
    pub enum RestoreBackupError {
        DatabaseError(err: DatabaseError) {
            from()
            display("Database Error occured during restore: {} ", err)
            cause(err)
        }
        IoError(err: io::Error) {
            from()
            display("I/O Error occured during restore: {} ", err)
            cause(err)
        }
        NodeCommunicationError {
            description("The node did not respond with the expected message")
        }
        RootHandleChunkNotAvailable(err: String) {
            description("Root Handle is not available on node")
            display("Root Handle {} is not available on the node", err)
        }
        ChunkNotAvailable(err: String) {
            description("Chunk is not available on node")
            display("Chunk {} is not available on the node", err)
        }
    }
}
