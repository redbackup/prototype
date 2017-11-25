use std::io;
use chunk_index::DatabaseError;


quick_error!{
    #[derive(Debug)]
    pub enum RestoreError {
        DatabaseError(err: DatabaseError) {
            from()
            cause(err)
        }
        IoError(err: io::Error) {
            from()
            cause(err)
        }
        NodeCommunicationError {
            description("The node did not respond with the expected message")
        }
    }
}
