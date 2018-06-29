extern crate ansi_term;

use std::collections::HashSet;
use sbtclient::{Message, Diagnostic};
use sbtclient::Message::*;
use sbtclient::receive::MessageHandler;
use ansi_term::Colour;

#[derive(Hash, Eq, PartialEq, Debug)]
struct FileAndDiagnostic {
    uri: String,
    diagnostic: Diagnostic
}

pub struct Printer {
    diagnostics: HashSet<FileAndDiagnostic>
}

impl MessageHandler for Printer {

    fn handle(&mut self, message: Message) {
        match message {
            LogMessage { method: _, params } => print_log(params.type_, params.message),
            SuccessResponse { id: _, result } => print_success_response(result.status, result.exit_code),
            ErrorResponse { id: _, error } => print_error_response(error.code, error.message),
            PublishDiagnostics { method: _, params } => self.print_diagnostics(params.uri, params.diagnostics)
        }
    }

}

impl Printer {

    pub fn new() -> Printer {
        Printer {
            diagnostics: HashSet::new()
        }
    }

    fn print_diagnostics(&mut self, uri: String, diagnostics: Vec<Diagnostic>) {
        for diagnostic in &diagnostics {
            self.print_diagnostic(&uri, diagnostic);
        }
    }

    fn print_diagnostic(&mut self, uri: &String, diagnostic: &Diagnostic) {
        let file_and_diagnostic = FileAndDiagnostic {
            uri: uri.to_string(),
            diagnostic: Diagnostic {
                range: diagnostic.range.clone(),
                severity: diagnostic.severity,
                message: diagnostic.message.clone()
            }
        };
        if !self.diagnostics.contains(&file_and_diagnostic) {
            let message = format!(
                "{}:{}:{}: {}",
                uri.replacen("file://", "", 1),
                diagnostic.range.start.line,
                diagnostic.range.start.character,
                diagnostic.message
            );
            // TODO print line of file
            print_log(diagnostic.severity, message);
            self.diagnostics.insert(file_and_diagnostic);
        }
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
