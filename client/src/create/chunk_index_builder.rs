use std::path::PathBuf;
use std::fs;
use std::io;
use std::fs::DirEntry;
use std::ffi::OsString;

use chrono::prelude::*;
use sha2::{Sha256,Digest};

use super::{ChunkIndex, DatabaseError};
use super::{Folder, NewFolder, File, NewFile, NewChunk};

quick_error! {
    #[derive(Debug)]
    pub enum BuilderError {
        DatabaseError(err: DatabaseError) {
            from()
            cause(err)
        }
        IoError(err: io::Error) {
            from()
            cause(err)
        }
        OsStringError(err: OsString) {
            from()
            display("The path {:?} contains invalid unicode characters", err)
        }
    }
}

pub struct ChunkIndexBuilder {
    chunk_index: ChunkIndex,
    path: PathBuf,
    parent_folder: Option<Folder>,
}

impl ChunkIndexBuilder {
    pub fn new(chunk_index: &ChunkIndex, path: &PathBuf) -> Result<Self, BuilderError> {
        let parentless = Self {
            chunk_index: chunk_index.clone(),
            path: path.clone(),
            parent_folder: None,
        };
        let parent_folder = parentless.add_folder(path).map_err(|e| BuilderError::from(e))?;

        Ok(Self {
            chunk_index: chunk_index.clone(),
            path: path.clone(),
            parent_folder: Some(parent_folder),
        })
    }

    pub fn build(self) -> Result<(), BuilderError> {
        for entry in self.path.read_dir()? {
            let entry = entry?;
            match entry.file_type() {
                Ok(ref filetype) if filetype.is_file() => {
                    self.add_file(entry)?;
                },

                Ok(ref filetype) if filetype.is_dir()  => {
                    let folder = self.add_folder(&entry.path())?;
                    Self {
                        chunk_index: self.chunk_index.clone(),
                        path: entry.path(),
                        parent_folder: Some(folder),
                    }.build()?;
                },

                Ok(filetype) => error!("The file type {:?} of file {:?} is not implemented",
                                       filetype, entry.file_name()),
                _            => error!("Could not read file type of {:?}", entry.file_name()),
            }
        }
        Ok(())
    }

    fn add_file(&self, file_entry: DirEntry) -> Result<File,BuilderError> {
        let metadata = file_entry.metadata()?;
        let modified = metadata.modified()?;
        let modified = DateTime::<Local>::from(modified);

        let folder_id = self.parent_folder.clone().unwrap().id;
        let file = self.chunk_index.add_file(NewFile {
            name: file_entry.file_name().into_string()?,
            last_change_date: modified.naive_local(),
            folder: folder_id,
        })?;

        let mut binary_file = fs::File::open(&file_entry.path())?;
        let hash = Sha256::digest_reader(&mut binary_file)?;
        let chunk_identifier = hash.iter().map(|e| format!("{:x}", e)).flat_map(|s| s.chars().collect::<Vec<_>>()).collect();

        self.chunk_index.add_chunk(NewChunk{
            chunk_identifier,
            file: file.id,
            predecessor: None
        })?;

        Ok(file)
    }

    fn add_folder(&self, folder_path: &PathBuf) -> Result<Folder, BuilderError> {
        let name = OsString::from(folder_path.file_name()
           .ok_or(io::Error::new(io::ErrorKind::NotFound ,"No folder in path given"))?).into_string()?;

        let parent_folder = match self.parent_folder {
            Some(ref folder) => Some(folder.id),
            None => None,
        };

        self.chunk_index.add_folder(NewFolder { name, parent_folder }).map_err(|e| BuilderError::from(e))
    }
}
