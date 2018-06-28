use std::fmt::Display;
use sbtclient::SbtClientError;

pub fn error(message: &str) -> SbtClientError {
    SbtClientError { message: message.to_string() }
}

pub fn detailed_error<E: Display>(message: &str, e: E) -> SbtClientError {
    let error_message = format!("{}. Details: {}", message, e);
    error(&error_message)
}
