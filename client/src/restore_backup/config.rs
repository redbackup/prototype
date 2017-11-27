use std::path::PathBuf;
use std::str;

pub struct RestoreBackupConfig {
    pub backup_id: String,
    pub restore_dir: PathBuf,
}

quick_error! {
    #[derive(Debug)]
    pub enum RestoreBackupConfigError {
        NonExistingDirectory(dirname: String) {}
        InvalidBackupId(id: String) {}
    }
}

impl RestoreBackupConfig {
    pub fn new(backup_id: &str,local_restore_dir: &str) -> Result<RestoreBackupConfig, RestoreBackupConfigError> {
        let restore_dir = PathBuf::from(local_restore_dir);
        if !restore_dir.is_dir() {
            return Err(RestoreBackupConfigError::NonExistingDirectory(
                local_restore_dir.into(),
            ));
        }

        let backup_id = String::from(backup_id);
        if backup_id.len() != 64 { // This validation is hash dependent.
            return Err(RestoreBackupConfigError::InvalidBackupId(backup_id));
        };

        Ok(RestoreBackupConfig {
            backup_id,
            restore_dir,
        })
    }
}
