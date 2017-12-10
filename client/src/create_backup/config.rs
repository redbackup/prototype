use std::path::PathBuf;
use std::str;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use glob::{Pattern, PatternError};

use chrono::{DateTime, Utc, NaiveDateTime};

pub struct CreateBackupConfig {
    pub backup_dir: PathBuf,
    pub expiration_date: DateTime<Utc>,
    pub exclude: Vec<Pattern>,
}

quick_error! {
    #[derive(Debug)]
    pub enum CreateBackupConfigError {
        NonExistingDirectory(dirname: String) {}
        InvalidDateFormat(date: String) {}
        DateNotFarEnoughInTheFuture(date: DateTime<Utc>) {}
        ExcludeFromFileReadError(err: io::Error) {
            from()
            display("ExcludeFromFileReadError: {}", err)
            cause(err)
        }
        ExcludePatternError(err: PatternError) {
            from()
            display("ExcludePatternError: {}", err)
            cause(err)
        }
    }
}

impl CreateBackupConfig {
    pub fn new(
        local_backup_dir: &str,
        expiration_date: &str,
        exclude_from: Option<&str>,
    ) -> Result<CreateBackupConfig, CreateBackupConfigError> {
        let backup_dir = PathBuf::from(local_backup_dir);
        if !backup_dir.is_dir() {
            return Err(CreateBackupConfigError::NonExistingDirectory(
                local_backup_dir.into(),
            ));
        }

        let expiration_date = NaiveDateTime::parse_from_str(expiration_date, "%Y-%m-%dT%H:%M")
            .map_err(|_| {
                CreateBackupConfigError::InvalidDateFormat(expiration_date.into())
            })?;
        let expiration_date = DateTime::from_utc(expiration_date, Utc);

        if expiration_date <= Utc::now() {
            return Err(CreateBackupConfigError::DateNotFarEnoughInTheFuture(
                expiration_date,
            ));
        }

        let mut exclude = Vec::new();
        if let Some(exclude_from) = exclude_from {
            let exclude_from_path = PathBuf::from(exclude_from);
            if !exclude_from_path.is_file() {
                return Err(CreateBackupConfigError::ExcludeFromFileReadError(
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Exclude from file not found",
                    ),
                ));
            }

            exclude.append(&mut Self::parse_exclude_from(&exclude_from_path)?);
        }

        Ok(CreateBackupConfig {
            backup_dir,
            expiration_date,
            exclude,
        })
    }

    fn parse_exclude_from(file: &PathBuf) -> Result<Vec<Pattern>, CreateBackupConfigError> {
        let file = BufReader::new(File::open(file)?);
        let mut patterns = Vec::new();

        for line in file.lines() {
            let line = line?;
            let pattern = Pattern::new(&line)?;
            patterns.push(pattern);
        }

        Ok(patterns)
    }
}
