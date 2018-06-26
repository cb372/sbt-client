mod sbtclient;

extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate ansi_term;

#[macro_use]
extern crate serde_derive;

use sbtclient::Message;
use sbtclient::Message::*;

use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::env;

use regex::Regex;

use ansi_term::Colour;

fn main() {

    // TODO check the working directory is an sbt project (i.e. has a `project` directory)

    // TODO show usage if no args given
    let args: Vec<String> = env::args().skip(1).collect();
    let sbt_command_line = args.join(" ");

    // TODO use serde_json instead of hand-writing json like a loser
    let json_rpc_command = format!(
        r#"{{ "jsonrpc": "2.0", "id": 1, "method": "sbt/exec", "params": {{ "commandLine": "{}" }} }}"#,
        sbt_command_line
    );

    // TODO read the socket URI from active.json
    // TODO Fork an sbt server if no project/target/active.json file exists
    let mut stream = UnixStream::connect("/Users/chris/.sbt/1.0/server/9f10750f3bdedd1e263b/sock").unwrap();
    stream.set_read_timeout(None).unwrap();

    stream.write_all(&with_content_length_header(&json_rpc_command)).unwrap();

    let mut received_result = false;
    while !received_result {
        let mut headers = Vec::with_capacity(1024);
        let mut one_byte = [0];
        while !ends_with_double_newline(&headers) {
            stream.read(&mut one_byte[..]).unwrap();
            headers.push(one_byte[0]);
        }
        let content_length = extract_content_length(String::from_utf8(headers).unwrap());
        let mut buf: Vec<u8> = Vec::with_capacity(content_length);
        buf.resize(content_length, 0);
        let bytes_read = stream.read(&mut buf).unwrap();
        // TODO loop while bytes read so far < content_length
        let raw_json = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();
        let message: Message = serde_json::from_str(&raw_json).unwrap();
        match message {
            Response { id, .. } if id == 1 => received_result = true,
            _ => ()
        }
        render(message);
    }
}

fn with_content_length_header(command: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}\r\n", command.len() + 2, command).into_bytes()
}

fn ends_with_double_newline(vec: &Vec<u8>) -> bool {
    vec.ends_with(&[13, 10, 13, 10])
}

fn extract_content_length(headers: String) -> usize {
    let content_length_header_regex = Regex::new(r"Content-Length: (\d+)").unwrap();
    let captures = content_length_header_regex.captures(&headers).unwrap();
    captures.get(1).unwrap().as_str().parse::<usize>().unwrap()
}

fn render(message: Message) {
    match message {
        LogMessage { method: _, params } => render_log(params.type_, params.message),
        Response { id: _, result } => render_response(result.status, result.exit_code),
        PublishDiagnostics { .. } => () // TODO
    }
}

fn render_log(level: u8, message: String) {
    let (colour, label) = match level {
        1 => (Colour::Red, "error"),
        2 => (Colour::Yellow, "warning"),
        _ => (Colour::White, "info")
    };
    println!("[{}] {}", colour.paint(label), message)
}

fn render_response(status: String, _exit_code: u8) {
    println!("[success] {}", Colour::Green.paint(status))
}

