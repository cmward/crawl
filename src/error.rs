use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("scanner error (line: {line:?}, position {position:?}, lexeme: {lexeme:?})")]
    ScannerError {
        position: usize,
        line: usize,
        lexeme: String,
    },
}
