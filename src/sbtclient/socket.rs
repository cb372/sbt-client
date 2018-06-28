extern crate serde_json;

use std::os::unix::net::UnixStream;
use std::fs::File;
use std::path::Path;
use serde_json::Value;
use sbtclient::SbtClientError;
use sbtclient::util::{error, detailed_error};

pub fn create_stream(working_directory: &Path) -> Result<UnixStream, SbtClientError> {
    let domain_socket = find_domain_socket(working_directory)?;
    connect_to_domain_socket(&domain_socket)
}

fn find_domain_socket(working_directory: &Path) -> Result<String, SbtClientError> {
    let mut port_file = working_directory.to_path_buf();
    port_file.push("project/target/active.json");
    if !port_file.is_file() {
        Err(error("Looks like the working directory is not an sbt project, or sbt server is not running."))
    } else {
        parse_port_file(port_file.as_path())
    }
}

fn parse_port_file(path: &Path) -> Result<String, SbtClientError> {
    let file = File::open(path)
        .map_err(|e| detailed_error("Failed to open sbt port file", e))?;
    let json: Value = serde_json::from_reader(file)
        .map_err(|e| detailed_error("Failed to parse sbt port file as JSON", e))?;
    let uri = json["uri"].as_str()
        .ok_or(error("Failed to extract domain socket path from port file"))?;
    Ok(uri.replacen("local://", "", 1))
}

fn connect_to_domain_socket(domain_socket: &str) -> Result<UnixStream, SbtClientError> {
    // TODO handle Connection Refused, fork an sbt server and wait for it to start
    let stream = UnixStream::connect(domain_socket)
        .map_err(|e| detailed_error("Failed to connect to Unix socket", e))?;
    stream.set_read_timeout(None)
        .map_err(|e| detailed_error("Failed to set read timeout on Unix socket", e))?;
    Ok(stream)
}
