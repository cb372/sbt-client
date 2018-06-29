extern crate ansi_term;

use std::collections::HashSet;
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::iter::repeat;
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
            let filename = uri.replacen("file://", "", 1);
            let message = format!(
                "{}:{}:{}: {}",
                filename,
                diagnostic.range.start.line + 1,
                diagnostic.range.start.character + 1,
                diagnostic.message
            );
            print_log(diagnostic.severity, message);
            self.print_code_snippet(filename, 
                                    diagnostic.severity,
                                    diagnostic.range.start.line,
                                    diagnostic.range.start.character);
            self.diagnostics.insert(file_and_diagnostic);
        }
    }

    fn print_code_snippet(&self, filename: String, severity: u8, line_number: usize, char_number: usize) {
        let lines = self.read_all_lines(filename);
        if lines.len() > line_number {
            print_log(severity, lines[line_number].to_string());
            self.print_line_marker(severity, char_number);
        }
    }

    fn print_line_marker(&self, severity: u8, char_number: usize) {
        let message = format!("{}^", repeat(' ').take(char_number).collect::<String>());
        print_log(severity, message);
    }

    fn read_all_lines<P>(&self, filename: P) -> Vec<String>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename).expect("no such file");
        let buf = BufReader::new(file);
        buf.lines()
            .map(|l| l.expect("Could not parse line"))
            .collect()
    }

}

pub fn print_log(level: u8, message: String) {
    let (colour, label) = match level {
        1 => (Colour::Red, "error"),
        2 => (Colour::Yellow, "warning"),
        _ => (Colour::White, "info")
    };
    let labelled_message = message.replace("\n", &format!("\n[{}] ", colour.paint(label)));
    println!("[{}] {}", colour.paint(label), labelled_message)
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
