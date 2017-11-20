use std::path::PathBuf;
use std::fs;
use std::io;
use std::fs::DirEntry;
use std::ffi::OsString;

use chrono::prelude::*;
use futures::Future;
use futures_cpupool::CpuPool;
use sha2::{Sha256,Digest};

use super::config::Config;
use super::chunk_index::{ChunkIndex, DatabaseError};
use super::chunk_index::schema::{Folder, NewFolder, File, NewFile, NewChunk};

quick_error! {
    #[derive(Debug)]
    pub enum BuildError {
        DatabaseError(err: DatabaseError) { from() }
        IoError(err: io::Error) { from() }
    }
}

pub fn build(config: &Config, backup_dir: PathBuf) -> Result<ChunkIndex, DatabaseError> {
    let datetime = Utc::now();
    let file_name = format!("{}/chunk_index-{}.db",
        config.chunk_index_storage.to_str().unwrap(),
        datetime.to_rfc3339());
    let chunk_index = ChunkIndex::new(&file_name, datetime)?;
    let pool = CpuPool::new_num_cpus();

    add_path_recursive(pool, chunk_index.clone(), backup_dir, None).expect("Could not add path recursively");
    Ok(chunk_index)
}

fn add_path_recursive(pool: CpuPool,
                      chunk_index: ChunkIndex,
                      folder_path: PathBuf,
                      parent_folder: Option<Folder>
                     ) -> Result<(), DatabaseError> {
    let folder = add_folder(&chunk_index, &folder_path, &parent_folder).expect("Could not add folder");

    if let Ok(entries) = folder_path.read_dir() {
        for entry in entries.filter(|s| s.is_ok()).map(|s| s.unwrap()) {
            match entry.file_type() {
                Ok(ref filetype) if filetype.is_file() => {
                    let chunk_index_clone = chunk_index.clone();
                    let folder_clone = folder.clone();
                         add_file(chunk_index_clone, entry, folder_clone);
                },

                Ok(ref filetype) if filetype.is_dir() => {
                    let pool_clone = pool.clone();
                    let chunk_index_clone = chunk_index.clone();
                    let folder_clone = folder.clone();
                        add_path_recursive(
                            pool_clone,
                            chunk_index_clone,
                            entry.path(),
                            Some(folder_clone)
                        );;
                },

                Ok(filetype) => {
                    unimplemented!("The file type {:?} is not implemented", filetype);
                },
                _ => panic!("Could not read file type"),
            }
        }
    }
    Ok(())
}

fn add_file(chunk_index: ChunkIndex, file_entry: DirEntry, parent_folder: Folder) -> Result<File,BuildError> {
    let metadata = file_entry.metadata().expect("Could not extract metadata");
    let modified = metadata.modified().expect("could not read modification date");
    let modified = DateTime::<Local>::from(modified);

    let file = chunk_index.add_file(NewFile {
        name: file_entry.file_name().into_string().unwrap(),
        last_change_date: modified.naive_local(),
        folder: parent_folder.id,
    })?;

    let mut binary_file = fs::File::open(&file_entry.path())?;
    let hash = Sha256::digest_reader(&mut binary_file)?;
    let hash = hash.iter().map(|e| format!("{:x}", e)).flat_map(|s| s.chars().collect::<Vec<_>>()).collect();

    chunk_index.add_chunk(NewChunk{
        chunk_identifier: hash,
        file: file.id,
        predecessor: None
    })?;

    Ok(file)
}

fn add_folder(chunk_index: &ChunkIndex, folder_path: &PathBuf, parent_folder: &Option<Folder>) -> Result<Folder, DatabaseError> {
    chunk_index.add_folder(NewFolder {
        name: match folder_path.file_name() {
            Some(name) => OsString::from(name).into_string().unwrap(),
            None => String::new(),
        },
        parent_folder: match parent_folder {
            &Some(ref unwrapped_parent_folder) => Some(unwrapped_parent_folder.id),
            &None => None,
        }
    })
}
