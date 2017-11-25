use std::path::PathBuf;
use std::str;

pub struct RestoreConfig {
    pub backup_id: String,
    pub restore_dir: PathBuf,
}

quick_error! {
    #[derive(Debug)]
    pub enum RestoreConfigError {
        NonExistingDirectory(dirname: String) {}
        InvalidBackupId(id: String) {}
    }
}



impl RestoreConfig {
    pub fn new(backup_id: &str,local_restore_dir: &str) -> Result<RestoreConfig, RestoreConfigError> {
        let restore_dir = PathBuf::from(local_restore_dir);
        if !restore_dir.is_dir() {
            return Err(RestoreConfigError::NonExistingDirectory(
                local_restore_dir.into(),
            ));
        }

        let backup_id = String::from(backup_id);
        if backup_id.len() != 64 {
            return Err(RestoreConfigError::InvalidBackupId(backup_id));
        };

        Ok(RestoreConfig {
            backup_id,
            restore_dir,
        })
    }
}
