use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;

use crate::transaction::{TxStatus, TxType};
use crate::{ParserError, Transaction};

const RECORD_MAGIC: &[u8; 4] = b"YPBN";
const FIXED_PAYLOAD_SIZE: usize = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4;

pub fn read_bin<R: Read>(mut reader: R) -> Result<Vec<Transaction>, ParserError> {
    let mut transactions = Vec::new();
    let mut record_index = 0usize;

    loop {
        match read_one_record(&mut reader, record_index + 1)? {
            Some(transaction) => {
                transactions.push(transaction);
                record_index += 1;
            }
            None => return Ok(transactions),
        }
    }
}

pub fn write_bin<W: Write>(mut writer: W, transactions: &[Transaction]) -> Result<(), ParserError> {
    for (record_index, transaction) in transactions.iter().enumerate() {
        write_one_record(&mut writer, transaction, record_index + 1)?;
    }

    writer
        .flush()
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    Ok(())
}

fn read_one_record<R: Read>(
    reader: &mut R,
    record_index: usize,
) -> Result<Option<Transaction>, ParserError> {
    let mut magic_buffer = [0u8; 4];

    match reader.read_exact(&mut magic_buffer) {
        Ok(()) => {}
        Err(read_error) if read_error.kind() == ErrorKind::UnexpectedEof => {
            return Ok(None);
        }
        Err(read_error) => {
            return Err(ParserError::IoError(format!(
                "failed to read record {} magic: {}",
                record_index, read_error
            )));
        }
    }

    if &magic_buffer != RECORD_MAGIC {
        return Err(ParserError::InvalidFormat(format!(
            "record {} has invalid magic: {:?}",
            record_index, magic_buffer
        )));
    }

    let payload_length = read_u32_be(reader, record_index, "payload_length")? as usize;

    if payload_length < FIXED_PAYLOAD_SIZE {
        return Err(ParserError::InvalidFormat(format!(
            "record {} payload is too short: {}",
            record_index, payload_length
        )));
    }

    let tx_id = read_i64_be(reader, record_index, "tx_id")?;
    let tx_type_code = read_u8(reader, record_index, "tx_type")?;
    let from_user_id = read_i64_be(reader, record_index, "from_user_id")?;
    let to_user_id = read_i64_be(reader, record_index, "to_user_id")?;
    let amount = read_i64_be(reader, record_index, "amount")?;
    let timestamp = read_i64_be(reader, record_index, "timestamp")?;
    let status_code = read_u8(reader, record_index, "status")?;
    let description_length = read_u32_be(reader, record_index, "description_length")? as usize;

    let expected_payload_length = FIXED_PAYLOAD_SIZE + description_length;
    if payload_length != expected_payload_length {
        return Err(ParserError::InvalidFormat(format!(
            "record {} payload length mismatch: declared {}, expected {}",
            record_index, payload_length, expected_payload_length
        )));
    }

    let description = read_string(reader, description_length, record_index, "description")?;
    let description = description.trim_matches('"').to_string();

    Ok(Some(Transaction {
        tx_id,
        tx_type: tx_type_from_code(tx_type_code, record_index)?,
        from_user_id,
        to_user_id,
        amount,
        timestamp,
        status: tx_status_from_code(status_code, record_index)?,
        description,
    }))
}

fn write_one_record<W: Write>(
    writer: &mut W,
    transaction: &Transaction,
    record_index: usize,
) -> Result<(), ParserError> {
    let description = format!("\"{}\"", transaction.description);
    let description_bytes = description.as_bytes();

    let payload_length = FIXED_PAYLOAD_SIZE
        .checked_add(description_bytes.len())
        .ok_or_else(|| {
            ParserError::InvalidFormat(format!("record {} payload length overflow", record_index))
        })?;

    let payload_length_u32 = u32::try_from(payload_length).map_err(|_| {
        ParserError::InvalidFormat(format!(
            "record {} payload too large: {}",
            record_index, payload_length
        ))
    })?;

    writer
        .write_all(RECORD_MAGIC)
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&payload_length_u32.to_be_bytes())
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&transaction.tx_id.to_be_bytes())
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&[tx_type_to_code(&transaction.tx_type)])
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&transaction.from_user_id.to_be_bytes())
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&transaction.to_user_id.to_be_bytes())
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&transaction.amount.to_be_bytes())
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&transaction.timestamp.to_be_bytes())
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(&[tx_status_to_code(&transaction.status)])
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    let description_length_u32 = u32::try_from(description_bytes.len()).map_err(|_| {
        ParserError::InvalidFormat(format!(
            "record {} description too large: {}",
            record_index,
            description_bytes.len()
        ))
    })?;

    writer
        .write_all(&description_length_u32.to_be_bytes())
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    writer
        .write_all(description_bytes)
        .map_err(|write_error| ParserError::IoError(write_error.to_string()))?;

    Ok(())
}

fn read_u8<R: Read>(
    reader: &mut R,
    record_index: usize,
    field_name: &str,
) -> Result<u8, ParserError> {
    let mut buffer = [0u8; 1];
    reader.read_exact(&mut buffer).map_err(|read_error| {
        ParserError::IoError(format!(
            "failed to read {} for record {}: {}",
            field_name, record_index, read_error
        ))
    })?;
    Ok(buffer[0])
}

fn read_u32_be<R: Read>(
    reader: &mut R,
    record_index: usize,
    field_name: &str,
) -> Result<u32, ParserError> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer).map_err(|read_error| {
        ParserError::IoError(format!(
            "failed to read {} for record {}: {}",
            field_name, record_index, read_error
        ))
    })?;
    Ok(u32::from_be_bytes(buffer))
}

fn read_i64_be<R: Read>(
    reader: &mut R,
    record_index: usize,
    field_name: &str,
) -> Result<i64, ParserError> {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer).map_err(|read_error| {
        ParserError::IoError(format!(
            "failed to read {} for record {}: {}",
            field_name, record_index, read_error
        ))
    })?;
    Ok(i64::from_be_bytes(buffer))
}

fn read_string<R: Read>(
    reader: &mut R,
    length: usize,
    record_index: usize,
    field_name: &str,
) -> Result<String, ParserError> {
    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer).map_err(|read_error| {
        ParserError::IoError(format!(
            "failed to read {} for record {}: {}",
            field_name, record_index, read_error
        ))
    })?;

    String::from_utf8(buffer).map_err(|utf8_error| {
        ParserError::InvalidFormat(format!(
            "invalid utf-8 in {} for record {}: {}",
            field_name, record_index, utf8_error
        ))
    })
}

fn tx_type_from_code(code: u8, record_index: usize) -> Result<TxType, ParserError> {
    match code {
        0 => Ok(TxType::Deposit),
        1 => Ok(TxType::Transfer),
        2 => Ok(TxType::Withdrawal),
        _ => Err(ParserError::InvalidFormat(format!(
            "record {} has invalid tx_type code: {}",
            record_index, code
        ))),
    }
}

fn tx_status_from_code(code: u8, record_index: usize) -> Result<TxStatus, ParserError> {
    match code {
        0 => Ok(TxStatus::Success),
        1 => Ok(TxStatus::Failure),
        2 => Ok(TxStatus::Pending),
        _ => Err(ParserError::InvalidFormat(format!(
            "record {} has invalid status code: {}",
            record_index, code
        ))),
    }
}

fn tx_type_to_code(tx_type: &TxType) -> u8 {
    match tx_type {
        TxType::Deposit => 0,
        TxType::Transfer => 1,
        TxType::Withdrawal => 2,
    }
}

fn tx_status_to_code(status: &TxStatus) -> u8 {
    match status {
        TxStatus::Success => 0,
        TxStatus::Failure => 1,
        TxStatus::Pending => 2,
    }
}

#[test]
fn read_bin_reads_example_file() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_data")
        .join("records_example.bin");
    let file_bytes = std::fs::read(path).unwrap();
    let transactions = read_bin(std::io::Cursor::new(file_bytes)).unwrap();

    assert_eq!(transactions.len(), 1000);
    assert_eq!(transactions[0].tx_id, 1000000000000000);
    assert_eq!(transactions[0].tx_type, TxType::Deposit);
    assert_eq!(transactions[0].status, TxStatus::Failure);
    assert_eq!(transactions[0].description, "Record number 1");
}

#[test]
fn write_bin_then_read_bin_roundtrip() {
    let transactions = vec![Transaction {
        tx_id: 1,
        tx_type: TxType::Deposit,
        from_user_id: 0,
        to_user_id: 10,
        amount: 100,
        timestamp: 123456,
        status: TxStatus::Success,
        description: "hello".to_string(),
    }];

    let mut buffer = Vec::new();
    write_bin(&mut buffer, &transactions).unwrap();

    let decoded = read_bin(std::io::Cursor::new(buffer)).unwrap();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].tx_id, 1);
    assert_eq!(decoded[0].description, "hello");
}
