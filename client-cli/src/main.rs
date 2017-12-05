#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate redbackup_client;

use std::thread;
use std::process;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use redbackup_client::config::{Config, ParseError};
use redbackup_client::{CreateBackupConfig, CreateBackupConfigError, RestoreBackupConfig, RestoreBackupConfigError, Progress};

use clap::{App, Arg, SubCommand};


fn main() {
    let mut app = App::new("redbackup client-cli")
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
                .about("Create a new backup")
                .arg(
                    Arg::with_name("expiration-date")
                        .help("the expiration date of this snapshot (format: %Y-%m-%dT%H:%M)")
                        .required(true),
                )
                .arg(
                    Arg::with_name("local-backup-dir")
                        .help("Directories, that should be backuped")
                        .required(true),
                )
                .arg(
                    Arg::with_name("exclude-from")
                        .help("Exclude glob patterns from FILE")
                        .long_help("Exclude multiple glob patterns from FILE. Define one pattern per line. Patterns are relative to the backup root, e.g. 'pictures/**/*.jpg'. For allowed glob syntax, see https://docs.rs/glob/0/glob/struct.Pattern.html#main")
                        .long("exclude-from")
                        .takes_value(true)
                        .value_name("FILE")
                    ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List available backups on the node."),
        )
        .subcommand(
            SubCommand::with_name("restore")
                .about("List available backups on the node.")
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
        );
    let matches = app.clone().get_matches();

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
            let exclude_from = matches_create.value_of("exclude-from");

            let backup_cfg = CreateBackupConfig::new(
                local_backup_dir,
                expiration_date,
                exclude_from,
            ).unwrap_or_else(|err| {
                match err {
                    CreateBackupConfigError::NonExistingDirectory(err) => {
                        eprintln!("The given directory '{}' does not exist", err)
                    }
                    CreateBackupConfigError::InvalidDateFormat(err) => {
                        eprintln!(
                            "The given date '{}' can not be parsed (format: %Y-%m-%dT%H:%M)",
                            err
                        )
                    }
                    CreateBackupConfigError::DateNotFarEnoughInTheFuture(err) => {
                        eprintln!("The given date '{}' is not far enough in the future", err)
                    }
                    CreateBackupConfigError::ExcludeFromFileReadError(err) => {
                        eprintln!(
                            "The given exclude-from file does not exist or can not be read (Details: {:?})",
                            err
                        );
                    }
                    CreateBackupConfigError::ExcludePatternError(err) => {
                        eprintln!(
                            "Invalid exclude glob specified ({:?})",
                            err
                        );
                    }
                };
                process::exit(1);
            });
            
            let progress_sender = initialize_progress_observer();
            redbackup_client::create_backup(config, backup_cfg, progress_sender).unwrap_or_else(|err| handle_error(err));
        },

        ("list", _) => match redbackup_client::list_backups(config) {
            Err(err)              => handle_error(err),
            Ok(available_backups) => {
                println!("{:64} Expiration Date", "Backup ID"); // Backup ID length is hash dependent.
                for backup in available_backups {
                    println!("{} {}", backup.0, backup.1);
                }
            }
        }

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
            let progress_sender = initialize_progress_observer();
            redbackup_client::restore_backup(config, restore_cfg, progress_sender).unwrap_or_else(|err| handle_error(err));
        },

        (&_, _) => app.print_help().expect("Could not get help options"),
    }
}

fn handle_error<T: std::error::Error>(err: T) {
    eprintln!("Huston, we have a problem! An unexpected error occured.");
    eprintln!("Description: {}", err.description());
    eprintln!("Cause (if any): {:?}", err.cause());
    process::exit(1);
}

fn initialize_progress_observer() -> Sender<Progress> {
    let (tx, rx): (Sender<Progress>, Receiver<_>) = mpsc::channel();
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(msg) => println!("{}", msg.status_msg()),
                Err(_) => break
            }
        }
    });
    tx
}
