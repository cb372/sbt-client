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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_content_length_header_adds_the_header() {
        let command = r#"{ "jsonrpc": "2.0", "id": 2, "method": "sbt/exec", "params": { "commandLine": "clean" } }"#;

        let expected = "Content-Length: 91\r\n\r\n{ \"jsonrpc\": \"2.0\", \"id\": 2, \"method\": \"sbt/exec\", \"params\": { \"commandLine\": \"clean\" } }\r\n".as_bytes().to_vec();

        assert_eq!(expected, with_content_length_header(command));
    }

    #[test]
    fn send_command_writes_an_sbt_exec_command_to_the_stream() {
        let sbt_command_line = "clean";

        let expected = "Content-Length: 79\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"sbt/exec\",\"params\":{\"commandLine\":\"clean\"}}\r\n".as_bytes().to_vec();

        let mut buf = Vec::new();
        send_command(sbt_command_line.to_string(), &mut buf).unwrap();

        assert_eq!(expected, buf);
    }

}
