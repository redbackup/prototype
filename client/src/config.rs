use std::net::SocketAddr;
use std::str;
use std;

use dns_lookup::lookup_host;
use std::error::Error;

pub struct Config {
    pub addr: SocketAddr,
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseError {
        InvalidHostname(err: String) {}
        InvalidPort(err: std::num::ParseIntError) {}
    }
}

impl Config {
    pub fn new(hostname: &str, port: &str) -> Result<Config, ParseError> {
        let ips = lookup_host(hostname).map_err(|e| ParseError::InvalidHostname(e.description().into()))?;
        if ips.len() == 0 {
            return Err(ParseError::InvalidHostname("No IPs found associated with the given hostname".into()))
        }
        let port = port.parse().map_err(|e| ParseError::InvalidPort(e))?;
        let addr = SocketAddr::new(ips.get(0).unwrap().clone(), port);

        Ok(Config { addr })
    }
}
