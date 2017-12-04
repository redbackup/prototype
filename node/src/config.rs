use std::net::SocketAddr;
use std::str;
use std::option::Option;
use std::string::String;
use std::path::PathBuf;
use std;


pub struct Config {
    pub addr: SocketAddr,
    pub storage_location: PathBuf,
    pub db_location: String,
    pub known_nodes: Vec<SocketAddr>,
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseError {
        InvalidIp(err: std::net::AddrParseError) {
            from()
        }
        InvalidPort(err: std::num::ParseIntError) {
            from()
        }
    }
}

impl Config {
    pub fn new(
        ip: &str,
        port: &str,
        storage_location: &str,
        db_location: &str,
        known_nodes_strs: Vec<String>,
    ) -> Result<Config, ParseError> {
        let ip = ip.parse()?;
        let port = port.parse()?;
        let addr = SocketAddr::new(ip, port);

        let storage_location = PathBuf::from(storage_location).to_owned();

        let db_location = db_location.to_owned();

        let mut known_nodes = Vec::new();
        for known_node in known_nodes_strs {
            let addr = known_node.parse()?;
            known_nodes.push(addr)
        }

        Ok(Config {
            addr,
            storage_location,
            db_location,
            known_nodes,
        })
    }
}
