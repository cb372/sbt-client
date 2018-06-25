use std::os::unix::net::UnixStream;
use std::io::prelude::*;


fn main() {
    // TODO read the socket URI from active.json
    // TODO Fork an sbt server if no project/target/active.json file exists
    let mut stream = UnixStream::connect("/Users/chris/.sbt/1.0/server/53e97be1433449ba44c3/sock").unwrap();
    stream.set_read_timeout(None).unwrap();
    stream.write_all(&with_content_length_header(r#"{ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "initializationOptions": { } } }"#)).unwrap();
    let mut last_response = Vec::with_capacity(1024);
    let mut buf = [0; 1024];
    loop {
        while !contains_double_newline(&last_response) {
            let bytes_read = stream.read(&mut buf[..]).unwrap();
            println!("Read {} bytes", bytes_read);
            println!("{}", String::from_utf8(buf[0..bytes_read].to_vec()).unwrap());
            last_response.append(&mut buf[0..bytes_read].to_vec());
        }
        // TODO parse headers
        // TODO work out content length
        // TODO read remaining bytes of command if necessary
        last_response.clear();
    }
}

fn with_content_length_header(command: &str) -> Vec<u8> {
    return format!("Content-Length: {}\r\n\r\n{}\r\n", command.len() + 2, command).into_bytes()
}

fn contains_double_newline(last_response: &Vec<u8>) -> bool {
    return last_response.windows(4).any(|bytes| bytes == [13, 10, 13, 10]);
}
