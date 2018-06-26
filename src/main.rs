extern crate regex;

use std::os::unix::net::UnixStream;
use std::io::prelude::*;

use regex::Regex;


fn main() {
    // TODO read the socket URI from active.json
    // TODO Fork an sbt server if no project/target/active.json file exists
    let mut stream = UnixStream::connect("/Users/chris/.sbt/1.0/server/53e97be1433449ba44c3/sock").unwrap();
    stream.set_read_timeout(None).unwrap();
    stream.write_all(&with_content_length_header(r#"{ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "initializationOptions": { } } }"#)).unwrap();
    loop {
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
        println!("{}", String::from_utf8(buf[0..bytes_read].to_vec()).unwrap());
    }
}

fn with_content_length_header(command: &str) -> Vec<u8> {
    return format!("Content-Length: {}\r\n\r\n{}\r\n", command.len() + 2, command).into_bytes()
}

fn ends_with_double_newline(vec: &Vec<u8>) -> bool {
    return vec.ends_with(&[13, 10, 13, 10]);
}

fn extract_content_length(headers: String) -> usize {
    // TODO parse headers properly
    let content_length_header_regex = Regex::new(r"Content-Length: (\d+)").unwrap();
    let captures = content_length_header_regex.captures(&headers).unwrap();
    return captures.get(1).unwrap().as_str().parse::<usize>().unwrap();
}

