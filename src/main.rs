mod sbtclient;

extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate ansi_term;

#[macro_use]
extern crate serde_derive;

use sbtclient::SbtClientError;
use sbtclient::socket;
use sbtclient::send;
use sbtclient::receive;
use sbtclient::print::{print_message,print_log};
use sbtclient::receive::HeaderParser;

use std::env;

fn main() {

    // TODO check the working directory is an sbt project (i.e. has a `project` directory)

    // TODO show usage if no args given
    let args: Vec<String> = env::args().skip(1).collect();
    let sbt_command_line = args.join(" ");

    match run(sbt_command_line) {
        Ok(_) => (), // yay
        Err(e) => print_log(1, e.message)
    }
}

fn run(sbt_command_line: String) -> Result<(), SbtClientError> {

    // TODO read the socket URI from active.json
    // TODO Fork an sbt server if no project/target/active.json file exists
    let mut stream = socket::create_stream("/Users/chris/.sbt/1.0/server/9f10750f3bdedd1e263b/sock")?;

    send::send_command(sbt_command_line, &mut stream)?;

    let header_parser = HeaderParser::new();
    while !receive::receive_next_message(&mut stream, &header_parser, print_message)? {}

    Ok(())
}

