extern crate serde_json;

use std::io::Write;
use sbtclient::{Command, CommandParams, SbtClientError};
use sbtclient::util::detailed_error;

pub fn send_command<S: Write>(sbt_command_line: String, stream: &mut S) -> Result<(), SbtClientError> {
    let command = Command {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "sbt/exec".to_string(),
        params: CommandParams {
            command_line: sbt_command_line
        }
    };
    let command_json = serde_json::to_string(&command)
        .map_err(|e| detailed_error("Failed to serialize command to JSON", e))?;

    stream.write_all(&with_content_length_header(&command_json))
        .map_err(|e| detailed_error("Failed to write command to Unix socket", e))
}

fn with_content_length_header(command: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}\r\n", command.len() + 2, command).into_bytes()
}

// TODO test
