use std::net::SocketAddr;
use std::str;
use std;

pub struct Config {
    pub addr: SocketAddr,
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseError {
        InvalidIp(err: std::net::AddrParseError) {}
        InvalidPort(err: std::num::ParseIntError) {}
    }
}

impl Config {
    pub fn new(ip: &str, port: &str) -> Result<Config, ParseError> {
        let ip = ip.parse().map_err(|e| ParseError::InvalidIp(e))?;
        let port = port.parse().map_err(|e| ParseError::InvalidPort(e))?;
        let addr = SocketAddr::new(ip, port);

        Ok(Config { addr })
    }
}
