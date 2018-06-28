#[derive(Debug)]
pub struct SbtClientError {
    pub message: String
}

#[derive(Deserialize, Debug)]
pub struct CommandResult {
    pub status: String,
    #[serde(rename = "exitCode")]
    pub exit_code: u8
}

#[derive(Deserialize, Debug)]
pub struct LogMessageParams {
    #[serde(rename = "type")]
    pub type_: u8,
    pub message: String
}

#[derive(Deserialize, Debug)]
pub struct Diagnostic {
    severity: u8,
    message: String
}

#[derive(Deserialize, Debug)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Message {
    Response { id: i32, result: CommandResult },
    LogMessage { method: String, params: LogMessageParams },
    PublishDiagnostics { method: String, params: PublishDiagnosticsParams }
}

