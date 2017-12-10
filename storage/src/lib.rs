#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
extern crate sha2;

use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use sha2::{Sha256, Digest};

quick_error! {
    #[derive(Debug)]
    pub enum StorageError {
        IoError(err: std::io::Error) {
            from()
            cause(err)
        }
        CorruptedChunk(identifier: String, actual_identifier: String){
            description("Corrupted chunk detected!")
            display("The chunk with identifier {} produces another digest than its identifier (actual: {})", identifier, actual_identifier)
        }
        DeleteNonExistingChunk(identifier: String){
            description("Can not delete non-existing chunk")
            display("Can not delete non-existing chunk with identifier {}", identifier)
        }
        PersistExistingChunk(identifier: String){
            description("Can not persist already existing chunk")
            display("Can not persist already existing chunk with identifier {}", identifier)
        }
        GetNonExistingChunk(identifier: String){
            description("The chunk with the given identifier is not persisted")
            display("The chunk with the identifier {} is not persisted", identifier)
        }
    }
}

#[derive(Debug)]
pub struct Storage {
    location: PathBuf,
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Self { location: self.location.clone() }
    }
}

impl Storage {
    pub fn new(location: PathBuf) -> Result<Storage, StorageError> {
        if !location.exists() {
            debug!("Create nonexisting location {:?}", location);
            fs::create_dir_all(&*location)?;
            debug!("Use newly created location {:?} for storage", location);
        } else {
            debug!("Use existing location {:?} for storage", location);
        }
        info!("Initialised storage at {:?}", location);
        Ok(Storage { location: location })
    }

    pub fn persist(&self, identifier: &str, data: &Vec<u8>) -> Result<(), StorageError> {
        let path = self.filename_for_identifier(identifier);
        debug!("Persist chunk with identifer {} at {:?}", identifier, path);
        if path.exists() {
            return Err(StorageError::PersistExistingChunk(identifier.into()));
        }
        let mut fhandle = File::create(self.filename_for_identifier(identifier))?;
        fhandle.write_all(&data[..])?;
        fhandle.flush()?;
        Ok(())
    }

    pub fn get(&self, identifier: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.filename_for_identifier(identifier);
        debug!(
            "Load contents for chunk with identifer {} at {:?}",
            identifier,
            path
        );
        if !path.exists() {
            return Err(StorageError::GetNonExistingChunk(identifier.into()));
        }
        let mut fhandle = File::open(path)?;
        let mut buf = Vec::new();
        fhandle.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn delete(&self, identifier: &str) -> Result<(), StorageError> {
        let path = self.filename_for_identifier(identifier);
        debug!(
            "Delete contents for chunk with identifer {} at {:?}",
            identifier,
            path
        );
        if !path.exists() {
            return Err(StorageError::DeleteNonExistingChunk(identifier.into()));
        }
        std::fs::remove_file(path).map_err(|e| StorageError::from(e))
    }

    pub fn verify(&self, identifier: &str) -> Result<(), StorageError> {
        let path = self.filename_for_identifier(identifier);
        debug!(
            "Loading contents for chunk with identifer {} at {:?}",
            identifier,
            path
        );
        if !path.exists() {
            return Err(StorageError::GetNonExistingChunk(identifier.into()));
        }

        let mut file_pointer = fs::File::open(path)?;
        let hash = Sha256::digest_reader(&mut file_pointer)?;
        let actual_identifier: String = hash.iter().map(|e| format!("{:02x}", e)).fold(
            String::new(),
            |mut acc,
             s: String| {
                acc.push_str(&s);
                acc
            },
        );


        if actual_identifier != identifier {
            return Err(StorageError::CorruptedChunk(
                identifier.into(),
                actual_identifier,
            ));
        }
        Ok(())
    }

    pub fn location(&self) -> &Path {
        self.location.as_path()
    }

    fn filename_for_identifier(&self, identifier: &str) -> PathBuf {
        self.location().join(identifier)
    }
}
