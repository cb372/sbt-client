extern crate serde_json;

use std::os::unix::net::UnixStream;
use std::thread::sleep;
use std::io::ErrorKind::ConnectionRefused;
use std::time::Duration;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use serde_json::Value;
use sbtclient::SbtClientError;
use sbtclient::util::{error, detailed_error};

pub fn create_stream(working_directory: &Path) -> Result<UnixStream, SbtClientError> {
    sanity_check(working_directory)?;
    let domain_socket = find_domain_socket(working_directory, false)?;
    connect_to_domain_socket(domain_socket, false)
}

fn sanity_check(working_directory: &Path) -> Result<(), SbtClientError> {
    let mut project_dir = working_directory.to_path_buf();
    project_dir.push("project");
    if !project_dir.is_dir() {
        Err(error("Looks like the working directory is not an sbt project."))
    } else {
        Ok(())
    }
}

fn find_domain_socket(working_directory: &Path, waiting_for_server: bool) -> Result<String, SbtClientError> {
    let mut port_file = working_directory.to_path_buf();
    port_file.push("project/target/active.json");
    if !port_file.is_file() {
        if waiting_for_server {
            println!("Waiting for sbt server to start ...");
            sleep(Duration::from_millis(1000));
            find_domain_socket(working_directory, true)
        } else {
            println!("Looks like sbt server is not running (port file not found). Forking a server ...");
            fork_server()?;
            sleep(Duration::from_millis(1000));
            find_domain_socket(working_directory, true)
        }
    } else {
        parse_port_file(port_file.as_path())
    }
}

fn connect_to_domain_socket(domain_socket: String, waiting_for_server: bool) -> Result<UnixStream, SbtClientError> {
    let stream_result = match UnixStream::connect(&domain_socket) {
        Ok(s) => Ok(s),
        Err(ref e) if e.kind() == ConnectionRefused => {
            if waiting_for_server {
                println!("Waiting for sbt server to start ...");
                sleep(Duration::from_millis(1000));
                connect_to_domain_socket(domain_socket, true)
            } else {
                println!("Looks like sbt server is not running (connection refused). Forking a server ...");
                fork_server()?;
                sleep(Duration::from_millis(1000));
                connect_to_domain_socket(domain_socket, true)
            }
        },
        Err(other) => Err(detailed_error("Failed to connect to Unix socket", other))
    };

    let stream = stream_result?;

    stream.set_read_timeout(None)
        .map_err(|e| detailed_error("Failed to set read timeout on Unix socket", e))?;
    Ok(stream)
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

fn fork_server() -> Result<(), SbtClientError> {
    Command::new("sbt")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| detailed_error("Failed to fork sbt server", e))?;
    Ok(())
}
