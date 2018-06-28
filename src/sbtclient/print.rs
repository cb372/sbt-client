extern crate ansi_term;

use sbtclient::Message;
use sbtclient::Message::*;
use ansi_term::Colour;

pub fn print_message(message: Message) {
    match message {
        LogMessage { method: _, params } => print_log(params.type_, params.message),
        Response { id: _, result } => print_response(result.status, result.exit_code),
        PublishDiagnostics { .. } => () // TODO
    }
}

pub fn print_log(level: u8, message: String) {
    let (colour, label) = match level {
        1 => (Colour::Red, "error"),
        2 => (Colour::Yellow, "warning"),
        _ => (Colour::White, "info")
    };
    println!("[{}] {}", colour.paint(label), message)
}

fn print_response(status: String, _exit_code: u8) {
    println!("[success] {}", Colour::Green.paint(status))
}

