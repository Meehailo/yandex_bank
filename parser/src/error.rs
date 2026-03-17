use std::fmt;

#[derive(Debug)]
pub enum ParserError {
    MissingField(usize, usize),
    InvalidTxType(String),
    InvalidTxStatus(String),
    InvalidInteger(String, usize),
    InvalidEnum(String, usize),
    CsvReadError(String),
    CsvWriteError(String),
    IoError(String),
    InvalidFormat(String)
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::MissingField(i, row_i) => {
                write!(f, "[Row {}] Missing field at index {}", row_i, i)
            }
            ParserError::InvalidEnum(msg, row_i) => {
                write!(f, "[Row {}] Invalid enum value: {}", row_i, msg)
            }
            ParserError::InvalidTxType(str) => {
                write!(f, "Enum type for {} not found", str)
            }
            ParserError::InvalidTxStatus(str) => {
                write!(f, "Enum Status for {} not found", str)
            }
            ParserError::InvalidInteger(str, row_i) => {
                write!(f, "[Row {}] integer error: \"{}\"", row_i, str)
            }
            ParserError::CsvReadError(str) => {
                write!(f, "csv read error: \"{}\"", str)
            }
            ParserError::CsvWriteError(str) => {
                write!(f, "csv write error: \"{}\"", str)
            }
            ParserError::IoError(str) => {
                write!(f, "Io Error: \"{}\"", str)
            }
            ParserError::InvalidFormat(str) => {
                write!(f, "Неверный формат: {}", str)
            }
        }
    }
}

impl From<std::io::Error> for ParserError {
    fn from(error: std::io::Error) -> Self {
        ParserError::IoError(error.to_string())
    }
}
