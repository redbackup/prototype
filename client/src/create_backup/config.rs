use std::path::PathBuf;
use std::str;

use chrono::{DateTime, Utc, NaiveDateTime};

pub struct CreateBackupConfig {
    pub backup_dir: PathBuf,
    pub expiration_date: DateTime<Utc>,
}

quick_error! {
    #[derive(Debug)]
    pub enum CreateBackupConfigError {
        NonExistingDirectory(dirname: String) {}
        InvalidDateFormat(date: String) {}
        DateNotFarEnoughInTheFuture(date: DateTime<Utc>) {}
    }
}



impl CreateBackupConfig {
    pub fn new(
        local_backup_dir: &str,
        expiration_date: &str,
    ) -> Result<CreateBackupConfig, CreateBackupConfigError> {
        let backup_dir = PathBuf::from(local_backup_dir);
        if !backup_dir.is_dir() {
            return Err(CreateBackupConfigError::NonExistingDirectory(
                local_backup_dir.into(),
            ));
        }

        let expiration_date = NaiveDateTime::parse_from_str(expiration_date, "%Y-%m-%dT%H:%M").map_err(
            |_| CreateBackupConfigError::InvalidDateFormat(expiration_date.into()),
        )?;
        let expiration_date = DateTime::from_utc(expiration_date, Utc);

        if expiration_date <= Utc::now() {
            return Err(CreateBackupConfigError::DateNotFarEnoughInTheFuture(
                expiration_date,
            ));
        }

        Ok(CreateBackupConfig {
            backup_dir,
            expiration_date,
        })
    }
}
