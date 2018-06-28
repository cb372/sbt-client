use std::os::unix::net::UnixStream;

use sbtclient::SbtClientError;
use sbtclient::util::detailed_error;

pub fn create_stream(socket_file: &str) -> Result<UnixStream, SbtClientError> {
    let stream = UnixStream::connect(socket_file)
        .map_err(|e| detailed_error("Failed to connect to Unix socket", e))?;
    stream.set_read_timeout(None)
        .map_err(|e| detailed_error("Failed to set read timeout on Unix socket", e))?;
    Ok(stream)
}
