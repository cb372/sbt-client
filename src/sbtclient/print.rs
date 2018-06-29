extern crate ansi_term;

use sbtclient::{Message, Diagnostic};
use sbtclient::Message::*;
use ansi_term::Colour;

pub fn print_message(message: Message) {
    match message {
        LogMessage { method: _, params } => print_log(params.type_, params.message),
        SuccessResponse { id: _, result } => print_success_response(result.status, result.exit_code),
        ErrorResponse { id: _, error } => print_error_response(error.code, error.message),
        PublishDiagnostics { method: _, params } => print_diagnostics(params.uri, params.diagnostics)
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

fn print_success_response(status: String, _exit_code: u8) {
    println!("[{}] {}", Colour::Green.paint("success"), status)
}

fn print_error_response(code: i32, message: String) {
    let full_message = format!(
        "sbt failed to execute our command. Error code: {}, message: '{}'",
        code,
        message
    );
    println!("[{}] {}", Colour::Red.paint("error"), full_message)
}

fn print_diagnostics(uri: String, diagnostics: Vec<Diagnostic>) {
    for diagnostic in &diagnostics {
        print_diagnostic(&uri, diagnostic);
    }
}

fn print_diagnostic(uri: &String, diagnostic: &Diagnostic) {
    let message = format!(
        "{}:{}:{}: {}",
        uri,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
        diagnostic.message
    );
    print_log(diagnostic.severity, message);
}

