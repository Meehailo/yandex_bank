//! Библиотека для чтения и записи банковских транзакций
//! в форматах BIN, CSV и TXT.
//!
//! Поддерживает:
//! - чтение транзакций из BIN, CSV и TXT;
//! - запись транзакций в BIN, CSV и TXT;
//! - валидацию входных данных при парсинге
#![warn(missing_docs)]

mod bin_format;
mod csv_format;
mod error;
/// Транзакция
pub mod transaction;
mod txt_format;

pub use bin_format::{read_bin, write_bin};
pub use csv_format::{read_csv, write_csv};
pub use error::ParserError;
pub use transaction::Transaction;
pub use txt_format::{read_txt, write_txt};
