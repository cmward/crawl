use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("scanner error (line: {line:?}, position {position:?}, lexeme: {lexeme:?}, reason: {reason:?})")]
    ScannerError {
        position: usize,
        line: usize,
        lexeme: String,
        reason: String,
    },
    #[error("parser error (token: {token:?})")]
    ParserError { token: String },  // TODO: get token info for line, position, etc.
    #[error("interpreter error (reason: {reason:?})")]
    InterpreterError { reason: String },  // TODO: get token info for line, position, etc.
}
