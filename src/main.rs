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
use sbtclient::print::{Printer, print_log};
use sbtclient::receive::HeaderParser;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 0 {
        println!("Usage: sbt-client <command line to send to sbt>")
    } else {
        let sbt_command_line = args.join(" ");

        process::exit(match run(sbt_command_line) {
            Ok(_) => 0, // yay
            Err(e) => {
                print_log(1, e.message);
                1
            }
        })
    }
}

fn run(sbt_command_line: String) -> Result<(), SbtClientError> {
    let working_directory = env::current_dir().unwrap();
    let mut stream = socket::create_stream(working_directory.as_path())?;

    send::send_command(sbt_command_line, &mut stream)?;

    let header_parser = HeaderParser::new();
    let mut printer = Printer::new();
    while !receive::receive_next_message(&mut stream, &header_parser, &mut printer)? {}

    Ok(())
}

