use std::net::SocketAddr;
use std::net::IpAddr;
use std::str;
use std::option::Option;
use std::string::String;
use std::path::PathBuf;
use std;

use dns_lookup::lookup_host;


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
            display("Invalid IP given ({})", err)
            cause(err)
        }
        InvalidPort(err: std::num::ParseIntError) {
            from()
            display("Invalid Port given ({})", err)
            cause(err)
        }
        InvalidKnownNode(node: String, err: std::io::Error) {
            display("Failed to resolve known Node {} ({})", node ,err)
            cause(err)
        }
        CannotResolveKonwnNode(node: String) {
            display("Could not resolve Node {} (got no results)", node)
        }
        NoIPsFound(msg: String){
            display("{}", msg)
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
            let mut split: Vec<_> = known_node.rsplitn(2, ':').collect();
            split.reverse();
            let ips = lookup_host(split[0]).map_err(|e| {
                ParseError::InvalidKnownNode(known_node.clone(), e)
            })?;

            // For simplicity, we only support ipv4 for now...
            let ips: Vec<_> = ips.into_iter()
                .filter(|ip| match ip {
                    &IpAddr::V4(_) => true,
                    _ => false,
                })
                .collect();

            if ips.len() == 0 {
                return Err(ParseError::NoIPsFound(format!(
                    "No IPv4 address found associated with the given hostname {}",
                    known_node.clone()
                )));
            } else if ips.len() > 1 {
                warn!(
                    "Found more than one possible IP for the given host. Will use the first one..."
                )
            }

            let port: u16 = if let Some(portstr) = split.get(1) {
                portstr.parse().map_err(|e| ParseError::InvalidPort(e))?
            } else {
                8080
            };
            known_nodes.push(SocketAddr::new(ips[0], port));
        }

        Ok(Config {
            addr,
            storage_location,
            db_location,
            known_nodes,
        })
    }
}
