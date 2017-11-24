use std::io;
use chunk_index::DatabaseError;
use super::chunk_index_builder::BuilderError;


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
        DesignationNotGrantedError(node: String) {
            description("Designation was not granted")
            display("Designation was not granted by the node {}", node)
        }
        ChunkNotAcknowledged(chunk_identifier: String) {
            description("Chunk was not acknowledged")
            display("The Chunk {} was not acknowledged by the node", chunk_identifier)
        }
        GetRemainingChunksFailed {
            description("Could not get remaining chunks")
        }
        NodeCommunicationError {
            description("The node did not respond with the expected message")
        }
    }
}