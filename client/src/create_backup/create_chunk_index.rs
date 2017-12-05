use std::path::PathBuf;
use std::io;
use std::fs::DirEntry;
use std::ffi::OsString;

use chrono::prelude::*;
use glob::Pattern;

use super::{ChunkIndex, DatabaseError};
use super::{Folder, NewFolder, File, NewFile, NewChunk};
use super::create_utils;

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

pub struct CreateChunkIndex {
    chunk_index: ChunkIndex,
    path: PathBuf,
    root_path: PathBuf,
    parent_folder: Option<Folder>,
    exclude: Vec<Pattern>,
}

impl CreateChunkIndex {
    pub fn new(chunk_index: &ChunkIndex, path: &PathBuf, exclude: &Vec<Pattern>) -> Result<(), BuilderError> {
        debug!("Create chunk index root folder");
        let mut create_chunk_index = Self {
            chunk_index: chunk_index.clone(),
            path: path.clone(),
            root_path: path.clone(),
            parent_folder: None,
            exclude: exclude.clone(),
        };

        let parent_folder = create_chunk_index.add_folder(path).map_err(|e| BuilderError::from(e))?;
        create_chunk_index.parent_folder = Some(parent_folder);

        debug!("Start building chunk index");
        let result = create_chunk_index.build();
        debug!("Finished building chunk index");
        result
    }

    fn build(self) -> Result<(), BuilderError> {
        debug!("Read content of path {:?}", self.path);
        for entry in self.path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            let local_path = path.strip_prefix(&self.root_path).unwrap();

            if let Some(pattern) = self.exclude.iter().find(|e| e.matches_path(&local_path)) {
                info!("Skipped {:?} because of glob pattern {})", &local_path, pattern.as_str());
                continue;
            }

            match entry.file_type() {
                Ok(ref filetype) if filetype.is_file() => {
                    self.add_file(entry)?;
                },

                Ok(ref filetype) if filetype.is_dir()  => {
                    let folder = self.add_folder(&path)?;
                    Self {
                        chunk_index: self.chunk_index.clone(),
                        path: path.clone(),
                        root_path: self.root_path.clone(),
                        parent_folder: Some(folder),
                        exclude: self.exclude.clone(),
                    }.build()?;
                },

                Ok(filetype) => warn!("The file type {:?} of file {:?} is not implemented",
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

        let chunk_identifier = create_utils::file_hash(&file_entry.path())?;

        debug!("Add chunk {} to chunk index", chunk_identifier);
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

        debug!("Add folder {} to chunk index", name);
        self.chunk_index.add_folder(NewFolder { name, parent_folder }).map_err(|e| BuilderError::from(e))
    }
}
