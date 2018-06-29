pub mod socket;
pub mod send;
pub mod receive;
pub mod print;
mod util;

#[derive(Debug)]
pub struct SbtClientError {
    pub message: String
}

#[derive(Serialize, Debug)]
pub struct CommandParams {
    #[serde(rename = "commandLine")]
    pub command_line: String
}

#[derive(Serialize, Debug)]
pub struct Command {
    pub jsonrpc: String,
    pub id: i32,
    pub method: String,
    pub params: CommandParams
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CommandResult {
    pub status: String,
    #[serde(rename = "exitCode")]
    pub exit_code: u8
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct LogMessageParams {
    #[serde(rename = "type")]
    pub type_: u8,
    pub message: String
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Diagnostic {
    severity: u8,
    message: String
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Message {
    Response { id: i32, result: CommandResult },
    LogMessage { method: String, params: LogMessageParams },
    PublishDiagnostics { method: String, params: PublishDiagnosticsParams }
}

