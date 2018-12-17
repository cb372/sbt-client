extern crate serde_json;
extern crate regex;

use regex::Regex;
use std::io::Read;
use sbtclient::{Message, SbtClientError};
use sbtclient::Message::{SuccessResponse,ErrorResponse};
use sbtclient::util::{error, detailed_error};

pub trait MessageHandler {
    fn handle(&mut self, message: Message);
}

pub struct HeaderParser {
    content_length_header_regex: Regex
}

impl HeaderParser {

    pub fn new() -> HeaderParser {
        HeaderParser {
            content_length_header_regex: Regex::new(r"Content-Length: (\d+)").unwrap()
        }
    }

    pub fn extract_content_length(&self, raw_headers: String) -> Result<usize, SbtClientError> {
        let captures = self.content_length_header_regex.captures(&raw_headers)
            .ok_or(error("Failed to extract content length from headers"))?;
        let capture = captures.get(1)
            .ok_or(error("Failed to extract content length from headers"))?;
        capture.as_str().parse::<usize>()
            .map_err(|e| detailed_error("Failed to extract content length from headers", e))
    }

}

/*
 * Receives, deserializes and handles the next message from the server.
 * Returns true if it was the final message in response to our command, meaning we can stop looping.
 */
pub fn receive_next_message<S: Read, H: MessageHandler>(stream: &mut S,
                                                        header_parser: &HeaderParser,
                                                        handler: &mut H) -> Result<bool, SbtClientError> {
    let headers = read_headers(stream)?;
    let content_length = header_parser.extract_content_length(headers)?;
    let mut buf: Vec<u8> = Vec::with_capacity(content_length);
    buf.resize(content_length, 0);
    stream.read_exact(&mut buf)
        .map_err(|e| detailed_error("Failed to read bytes from Unix socket", e))?;
    let raw_json = String::from_utf8(buf.to_vec())
        .map_err(|e| detailed_error("Failed to decode message as UTF-8 string", e))?;
    let message: Message = serde_json::from_str(&raw_json)
        .map_err(|e| detailed_error(&format!("Failed to deserialize message from JSON '{}'", raw_json), e))?;
    let received_result = match message {
        SuccessResponse { id, .. } if id == 1 => Ok(true),
        ErrorResponse { id, .. } if id == 1 => Err(error("Error from sbt")),
        _ => Ok(false)
    };
    handler.handle(message);
    received_result
}

fn read_headers<S: Read>(stream: &mut S) -> Result<String, SbtClientError> {
    let mut headers = Vec::with_capacity(1024);
    let mut one_byte = [0];
    while !ends_with_double_newline(&headers) {
        try! (
            stream.read_exact(&mut one_byte)
                .map(|_| headers.push(one_byte[0]))
                .map_err(|e| detailed_error("Failed to read next byte of headers", e))
        )
    }
    String::from_utf8(headers)
        .map_err(|e| detailed_error("Failed to read headers as a UTF-8 string", e))
}

fn ends_with_double_newline(vec: &Vec<u8>) -> bool {
    vec.ends_with(&[13, 10, 13, 10])
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbtclient::*;
    use sbtclient::Message::*;

    struct TestMessageHandler {
        expected: Message
    }

    impl MessageHandler for TestMessageHandler {
        fn handle(&mut self, message: Message) {
            assert_eq!(self.expected, message);
        }
    }

    #[test]
    fn receive_successful_result() {
        let mut lsp_message = "Content-Type: application/vscode-jsonrpc; charset=utf-8\r
Content-Length: 126\r
\r
{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"status\":\"Done\",\"channelName\":\"network-1\",\"execId\":1,\"commandQueue\":[\"shell\"],\"exitCode\":0}}".as_bytes();

        let mut handler = TestMessageHandler {
            expected: SuccessResponse {
                id: 1,
                result: CommandResult {
                    status: "Done".to_string(),
                    exit_code: 0
                }
            }
        };

        let received_final_message = receive_next_message(
            &mut lsp_message,
            &HeaderParser::new(),
            &mut handler).unwrap();

        assert_eq!(true, received_final_message);
    }

    #[test]
    fn receive_error_response() {
        let mut lsp_message = "Content-Type: application/vscode-jsonrpc; charset=utf-8\r
Content-Length: 61\r
\r
{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"code\":-33000,\"message\":\"\"}}".as_bytes();

        let mut handler = TestMessageHandler {
            expected: ErrorResponse {
                id: 1,
                error: ErrorDetails {
                    code: -33000,
                    message: "".to_string(),
                }
            }
        };

        let received_final_message = receive_next_message(
            &mut lsp_message,
            &HeaderParser::new(),
            &mut handler).unwrap();

        assert_eq!(true, received_final_message);
    }

    #[test]
    fn receive_log_message() {
        let mut lsp_message = "Content-Type: application/vscode-jsonrpc; charset=utf-8\r
Content-Length: 89\r
\r
{\"jsonrpc\":\"2.0\",\"method\":\"window/logMessage\",\"params\":{\"type\":4,\"message\":\"Processing\"}}".as_bytes();

        let mut handler = TestMessageHandler {
            expected: LogMessage {
                method: "window/logMessage".to_string(),
                params: LogMessageParams {
                    type_: 4,
                    message: "Processing".to_string(),
                }
            }
        };

        let received_final_message = receive_next_message(
            &mut lsp_message,
            &HeaderParser::new(),
            &mut handler).unwrap();

        assert_eq!(false, received_final_message);
    }

    #[test]
    fn receive_compilation_errors() {
        let mut lsp_message = "Content-Type: application/vscode-jsonrpc; charset=utf-8\r
Content-Length: 609\r
\r
{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\"params\":{\"uri\":\"file:///Users/chris/code/cats-retry/modules/core/src/main/scala/retry/Fibonacci.scala\",\"diagnostics\":[{\"range\":{\"start\":{\"line\":17,\"character\":21},\"end\":{\"line\":17,\"character\":22}},\"severity\":1,\"source\":\"sbt\",\"message\":\"value * is not a member of Any\"},{\"range\":{\"start\":{\"line\":13,\"character\":28},\"end\":{\"line\":13,\"character\":29}},\"severity\":1,\"source\":\"sbt\",\"message\":\"not found: type Longg\"},{\"range\":{\"start\":{\"line\":5,\"character\":8},\"end\":{\"line\":5,\"character\":9}},\"severity\":1,\"source\":\"sbt\",\"message\":\"not found: value m\"}]}}".as_bytes();

        let mut handler = TestMessageHandler {
            expected: PublishDiagnostics {
                method: "textDocument/publishDiagnostics".to_string(),
                params: PublishDiagnosticsParams {
                    uri: "file:///Users/chris/code/cats-retry/modules/core/src/main/scala/retry/Fibonacci.scala".to_string(),
                    diagnostics: vec! [
                        Diagnostic {
                            range: Range {
                                start: Position { line: 17, character: 21 },
                                end:   Position { line: 17, character: 22 }
                            },
                            severity: 1,
                            message: "value * is not a member of Any".to_string()
                        },
                        Diagnostic {
                            range: Range {
                                start: Position { line: 13, character: 28 },
                                end:   Position { line: 13, character: 29 }
                            },
                            severity: 1,
                            message: "not found: type Longg".to_string()
                        },
                        Diagnostic {
                            range: Range {
                                start: Position { line: 5, character: 8 },
                                end:   Position { line: 5, character: 9 }
                            },
                            severity: 1,
                            message: "not found: value m".to_string()
                        }
                    ]
                }
            }
        };

        let received_final_message = receive_next_message(
            &mut lsp_message,
            &HeaderParser::new(),
            &mut handler).unwrap();

        assert_eq!(false, received_final_message);
    }

}
