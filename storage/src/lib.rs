#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;

use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::io::Write;

quick_error! {
    #[derive(Debug)]
    pub enum StorageError {
        IoError(err: std::io::Error) {
            from()
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


pub struct Storage {
    location: PathBuf,
}

impl Storage {
    pub fn new(location: PathBuf) -> Result<Storage, StorageError> {
        if !location.exists() {
            info!("Creating nonexisting location {:?}", location);
            fs::create_dir_all(&*location)?;
            info!("Using newly created location {:?} for storage", location);
        } else {
            info!("Using existing location {:?} for storage", location);
        }
        Ok(Storage { location: location })
    }

    pub fn persist(&self, identifier: &str, data: &Vec<u8>) -> Result<(), StorageError> {
        let path = self.filename_for_identifier(identifier);
        info!("Persisting chunk with identifer {} at {:?}", identifier, path);
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
        info!("Loading contents for chunk with identifer {} at {:?}", identifier, path);
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
        info!("Deleting contents for chunk with identifer {} at {:?}", identifier, path);
        if !path.exists() {
            return Err(StorageError::DeleteNonExistingChunk(identifier.into()));
        }
        std::fs::remove_file(path).map_err(|e| StorageError::from(e))
    }

    pub fn location(&self) -> &Path {
        self.location.as_path()
    }

    fn filename_for_identifier(&self, identifier: &str) -> PathBuf {
        self.location().join(identifier)
    }
}
