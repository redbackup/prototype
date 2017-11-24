#![recursion_limit="128"] // required for database inference
#[macro_use] extern crate quick_error;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate log;
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate chrono;
extern crate uuid;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dns_lookup;
extern crate sha2;

extern crate redbackup_protocol;

#[cfg(test)]
mod tests;

pub mod config;
pub mod create;
mod chunk_index;

pub use create::config::{CreateConfig, CreateConfigError};

pub fn create(config: config::Config, create_config: CreateConfig) -> Result<(), create::CreateError> {
    create::Create::new(
        config,
        create_config,
    )?.run()
}
