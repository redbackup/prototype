use std::error::Error;
use std::net::SocketAddr;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str;
use std;

use dns_lookup::lookup_host;

/// Shared configuration by the backup client.
pub struct Config {
    pub addr: SocketAddr,
    pub chunk_index_storage: PathBuf,
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseError {
        InvalidHostname(err: String) {}
        InvalidPort(err: std::num::ParseIntError) {}
        InvalidChunkIndexStorage(err: String) {}
    }
}

impl Config {
    pub fn new(
        hostname: &str,
        port: &str,
        chunk_index_storage: &str,
    ) -> Result<Config, ParseError> {
        let ips = lookup_host(hostname).map_err(|e| {
            ParseError::InvalidHostname(e.description().into())
        })?;
        // For simplicity, we only support ipv4 for now...
        let ips: Vec<_> = ips.into_iter()
            .filter(|ip| match ip {
                &IpAddr::V4(_) => true,
                _ => false,
            })
            .collect();
        if ips.len() == 0 {
            return Err(ParseError::InvalidHostname(
                "No IPs found associated with the given hostname".into(),
            ));
        }
        let port = port.parse().map_err(|e| ParseError::InvalidPort(e))?;
        let addr = SocketAddr::new(ips.get(0).unwrap().clone(), port);

        let chunk_index_storage = PathBuf::from(chunk_index_storage);
        if !chunk_index_storage.is_dir() {
            return Err(ParseError::InvalidChunkIndexStorage(
                "No valid chunk index storage directory given".into(),
            ));
        }

        Ok(Config {
            addr,
            chunk_index_storage,
        })
    }
}
