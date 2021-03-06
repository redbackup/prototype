use std::path::PathBuf;
use std::io::{Error, Write};
use std::fs::{OpenOptions, DirBuilder};

/// Writes the content buffer to a file path.
pub fn restore_file_content(content: &[u8], path: &PathBuf) -> Result<(), Error> {
    debug!("Restore file content to {:?}", path);
    let mut fhandle = OpenOptions::new().write(true).create_new(true).open(&path)?;
    debug!("Opened file {:?} to write", path);
    fhandle.write_all(content)?;
    debug!("Restored file {:?}", path);
    Ok(())
}

/// Create a folder recursively (with parent folders)
pub fn create_folder(path: &PathBuf) -> Result<(), Error> {
    debug!("Create folder {:?}", path);
    DirBuilder::new().recursive(true).create(path)?;
    Ok(())
}
