use std::io;

quick_error!{
    #[derive(Debug)]
    pub enum ListBackupsError {
        IoError(err: io::Error) {
            from()
            cause(err)
        }
        NodeCommunicationError {
            description("The node did not respond with the expected message")
        }
    }
}
