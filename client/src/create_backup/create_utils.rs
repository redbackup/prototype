use std::path::PathBuf;
use std::fs::File;
use std::io::Error;

use sha2::{Sha256, Digest};
use std::io::Read;

pub fn read_file_content(path: &PathBuf) -> Result<Vec<u8>, Error> {
    debug!("Read file content {:?}", path);
    let mut fhandle = File::open(path)?;
    let mut buf = Vec::new();
    fhandle.read_to_end(&mut buf)?;
    debug!("Successfully retreived content of {:?}", path);
    Ok(buf)
}

pub fn file_hash(file_path: &PathBuf) -> Result<String, Error> {
    debug!("Calculate hash of file {:?}", file_path);
    let mut file_pointer = File::open(&file_path)?;
    let hash = Sha256::digest_reader(&mut file_pointer)?;

    let string: String = hash.iter().map(|e| format!("{:02x}", e)).fold(
        String::new(),
        |mut acc,
         s: String| {
            acc.push_str(&s);
            acc
        },
    );

    debug!("Hash of file {:?} is {}", file_path, string);
    Ok(string)
}
