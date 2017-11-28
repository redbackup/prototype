#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate redbackup_client;

use std::process;

use redbackup_client::config::{Config, ParseError};
use redbackup_client::{CreateBackupConfig, CreateBackupConfigError, RestoreBackupConfig, RestoreBackupConfigError};

use clap::{App, Arg, SubCommand};


fn main() {
    let matches = App::new("redbackup client-cli")
        .about("redbackup client")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("node-hostname")
                .help("hostname of the node to contact")
                .short("h")
                .long("node-hostname")
                .takes_value(true)
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::with_name("node-port")
                .help("port of the node to contact")
                .short("p")
                .long("node-port")
                .takes_value(true)
                .default_value("8080"),
        )
        .arg(
            Arg::with_name("chunk-index-storage")
                .help("Folder where chunk indices are stored.")
                .long("chunk-index-storage")
                .takes_value(true)
                .default_value("/tmp/"),
        )
        .subcommand(
            SubCommand::with_name("create")
                .help("Create a new backup")
                .arg(
                    Arg::with_name("expiration-date")
                        .help("the expiration date of this snapshot (format: %Y-%m-%dT%H:%M)")
                        .required(true),
                )
                .arg(
                    Arg::with_name("local-backup-dir")
                        .help("Directories, that should be backuped")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .help("List available backups on the node."),
        )
        .subcommand(
            SubCommand::with_name("restore")
                .help("List available backups on the node.")
                .arg(
                    Arg::with_name("backup-id")
                        .help("ID of the backup that should be restored")
                        .required(true),
                )
                .arg(
                    Arg::with_name("local-restore-dir")
                        .help("Destionation, where the files should be restored to.")
                        .required(true),
                ),
        )
        .get_matches();

    let node_host = matches.value_of("node-hostname").unwrap();
    let node_port = matches.value_of("node-port").unwrap();
    let chunk_index_storage = matches.value_of("chunk-index-storage").unwrap();

    let config = Config::new(node_host, node_port, chunk_index_storage).unwrap_or_else(|err| {
        match err {
            ParseError::InvalidHostname(err) => {
                eprintln!("The given hostname is invalid ({})", err)
            }
            ParseError::InvalidPort(err) => {
                eprintln!("The given Port could not be parsed ({})", err)
            }
            ParseError::InvalidChunkIndexStorage(err) => {
                eprintln!("The given chunk index storage could not be used ({})", err)
            }
        };
        process::exit(1);
    });

    env_logger::init().unwrap();

    match matches.subcommand() {
        ("create", Some(matches_create)) => {
            let local_backup_dir = matches_create.value_of("local-backup-dir").unwrap();
            let expiration_date = matches_create.value_of("expiration-date").unwrap();
            let backup_cfg = CreateBackupConfig::new(local_backup_dir, expiration_date).unwrap_or_else(|err| {
                match err {
                    CreateBackupConfigError::NonExistingDirectory(err) => {
                        eprintln!("The given directory '{}' does not exist", err)
                    }
                    CreateBackupConfigError::InvalidDateFormat(err) => {
                        eprintln!("The given date '{}' can not be parsed (format: %Y-%m-%dT%H:%M)", err)
                    },
                    CreateBackupConfigError::DateNotFarEnoughInTheFuture(err) => {
                        eprintln!("The given date '{}' is not far enough in the future", err)
                    },
                };
                process::exit(1);
            });
            redbackup_client::create_backup(config, backup_cfg)
                .unwrap_or_else(|err| eprintln!("Huston, we have a problem! ({})", err));
        },

        ("list", _) => match redbackup_client::list_backups(config) {
            Err(err)              => eprintln!("Huston, we have a problem! ({})", err),
            Ok(available_backups) => {
                println!("{:64} Expiration Date", "Backup ID"); // Backup ID length is hash dependent.
                for backup in available_backups {
                    println!("{} {}", backup.0, backup.1);
                }
            },
        },

        ("restore", Some(matches_restore)) => {
            let local_restore_dir = matches_restore.value_of("local-restore-dir").unwrap();
            let backup_id = matches_restore.value_of("backup-id").unwrap();
            let restore_cfg = RestoreBackupConfig::new(backup_id, local_restore_dir).unwrap_or_else(|err| {
                match err {
                    RestoreBackupConfigError::NonExistingDirectory(err) => {
                        eprintln!("The given directory '{}' does not exist", err)
                    }
                    RestoreBackupConfigError::InvalidBackupId(err) => {
                        eprintln!("The given backup ID '{}' is invalid", err)
                    },
                };
                process::exit(1);
            });
            redbackup_client::restore_backup(config, restore_cfg)
                .unwrap_or_else(|err| eprintln!("Huston, we have a problem! ({})", err));
        },

        (&_, _) => eprintln!("No command was used!"),
    }
}
