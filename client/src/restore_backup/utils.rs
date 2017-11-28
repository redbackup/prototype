use std::path::PathBuf;
use std::io::{Error, Write};
use std::fs::{OpenOptions, DirBuilder};

pub fn restore_file_content(content: &[u8], path: &PathBuf) -> Result<(), Error> {
    let mut fhandle = OpenOptions::new().write(true).create_new(true).open(&path)?;
    debug!("Opened file {:?} to write", path);
    fhandle.write_all(content)?;
    debug!("Restored file {:?}", path);
    Ok(())
}

pub fn create_folder(path: &PathBuf) -> Result<(), Error> {
    DirBuilder::new().recursive(true).create(path)?;
    Ok(())
}
