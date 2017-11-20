use std::net::SocketAddr;
use std::str;
use std::path::PathBuf;
use std;


pub struct Config {
    pub addr: SocketAddr,
    pub storage_location : PathBuf,
    pub db_location: String,
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseError {
        InvalidIp(err: std::net::AddrParseError) {}
        InvalidPort(err: std::num::ParseIntError) {}
    }
}

impl Config {
    pub fn new(ip: &str, port: &str, storage_location: &str, db_location: &str) -> Result<Config, ParseError> {
        let ip = ip.parse().map_err(|e| ParseError::InvalidIp(e))?;
        let port = port.parse().map_err(|e| ParseError::InvalidPort(e))?;
        let addr = SocketAddr::new(ip, port);

        let storage_location = PathBuf::from(storage_location).to_owned();

        let db_location = db_location.to_owned();
        Ok(Config { addr, storage_location,  db_location})
    }
}
