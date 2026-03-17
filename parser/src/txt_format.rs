use crate::transaction::{TxStatus, TxType};
use crate::{ParserError, Transaction};
use std::io::{BufRead, BufReader, Read, Write};
use std::str::FromStr;

pub fn read_txt<R: Read>(reader: R) -> Result<Vec<Transaction>, ParserError> {
    let reader = BufReader::new(reader);
    let mut transactions = Vec::new();
    let mut lines = reader.lines();
    let mut current_record = Vec::new();
    let mut record_number = 0;

    while let Some(line) = lines.next() {
        let line = line.map_err(|e| ParserError::IoError(e.to_string()))?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line.starts_with("# Record") {
            if !current_record.is_empty() {
                let transaction = parse_txt_record(&current_record, record_number)?;
                transactions.push(transaction);
                current_record.clear();
            }

            record_number = extract_record_number(line)?;
            continue;
        }

        if !line.is_empty() {
            current_record.push(line.to_string());
        }
    }

    if !current_record.is_empty() {
        let transaction = parse_txt_record(&current_record, record_number)?;
        transactions.push(transaction);
    }

    Ok(transactions)
}

fn extract_record_number(line: &str) -> Result<usize, ParserError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        parts[2]
            .parse::<usize>()
            .map_err(|_| ParserError::InvalidFormat(format!("Invalid record number: {}", line)))
    } else {
        Err(ParserError::InvalidFormat(format!(
            "Invalid record header: {}",
            line
        )))
    }
}

pub fn parse_txt_record(lines: &[String], record_num: usize) -> Result<Transaction, ParserError> {
    let mut tx_id = None;
    let mut tx_type = None;
    let mut from_user_id = None;
    let mut to_user_id = None;
    let mut amount = None;
    let mut timestamp = None;
    let mut status = None;
    let mut description = None;

    for (line_index, line) in lines.iter().enumerate() {
        let (key, value) = parse_key_value(line)?;

        match key {
            "TX_ID" => tx_id = Some(parse_i64(value, "TX_ID", line_index)?),
            "TX_TYPE" => tx_type = Some(parse_tx_type(value, line_index)?),
            "FROM_USER_ID" => from_user_id = Some(parse_i64(value, "FROM_USER_ID", line_index)?),
            "TO_USER_ID" => to_user_id = Some(parse_i64(value, "TO_USER_ID", line_index)?),
            "AMOUNT" => amount = Some(parse_i64(value, "AMOUNT", line_index)?),
            "TIMESTAMP" => timestamp = Some(parse_i64(value, "TIMESTAMP", line_index)?),
            "STATUS" => status = Some(parse_tx_status(value, line_index)?),
            "DESCRIPTION" => description = Some(parse_string(value)?),
            _ => {
                return Err(ParserError::InvalidFormat(format!(
                    "Unknown field '{}' in record {}",
                    key, record_num
                )));
            }
        }
    }

    Ok(Transaction {
        tx_id: tx_id
            .ok_or_else(|| ParserError::InvalidFormat(format!("TX_ID in record {}", record_num)))?,
        tx_type: tx_type.ok_or_else(|| {
            ParserError::InvalidFormat(format!("TX_TYPE in record {}", record_num))
        })?,
        from_user_id: from_user_id.ok_or_else(|| {
            ParserError::InvalidFormat(format!("FROM_USER_ID in record {}", record_num))
        })?,
        to_user_id: to_user_id.ok_or_else(|| {
            ParserError::InvalidFormat(format!("TO_USER_ID in record {}", record_num))
        })?,
        amount: amount.ok_or_else(|| {
            ParserError::InvalidFormat(format!("AMOUNT in record {}", record_num))
        })?,
        timestamp: timestamp.ok_or_else(|| {
            ParserError::InvalidFormat(format!("TIMESTAMP in record {}", record_num))
        })?,
        status: status.ok_or_else(|| {
            ParserError::InvalidFormat(format!("STATUS in record {}", record_num))
        })?,
        description: description.ok_or_else(|| {
            ParserError::InvalidFormat(format!("DESCRIPTION in record {}", record_num))
        })?,
    })
}

fn parse_key_value(line: &str) -> Result<(&str, &str), ParserError> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(ParserError::InvalidFormat(format!(
            "Invalid line format: {}",
            line
        )));
    }

    let key = parts[0].trim();
    let value = parts[1].trim();

    Ok((key, value))
}

fn parse_i64(value: &str, field: &str, line_index: usize) -> Result<i64, ParserError> {
    value.trim_matches('"').parse::<i64>().map_err(|e| {
        ParserError::InvalidInteger(format!("{}: {} - {}", field, value, e), line_index)
    })
}

fn parse_tx_type(value: &str, line_index: usize) -> Result<TxType, ParserError> {
    TxType::from_str(value).map_err(|e| {
        ParserError::InvalidEnum(format!("Invalid TX_TYPE: {} - {}", value, e), line_index)
    })
}

fn parse_tx_status(value: &str, line_index: usize) -> Result<TxStatus, ParserError> {
    TxStatus::from_str(value).map_err(|e| {
        ParserError::InvalidEnum(format!("Invalid STATUS: {} - {}", value, e), line_index)
    })
}

fn parse_string(value: &str) -> Result<String, ParserError> {
    Ok(value.trim_matches('"').to_string())
}

pub fn write_txt<W: Write>(writer: W, transactions: &[Transaction]) -> Result<(), ParserError> {
    let mut writer = writer;

    for (i, trx) in transactions.iter().enumerate() {
        writeln!(writer, "# Record {} ({})", i + 1, trx.tx_type)?;

        writeln!(writer, "TX_TYPE: {}", trx.tx_type)?;
        writeln!(writer, "TO_USER_ID: {}", trx.to_user_id)?;
        writeln!(writer, "FROM_USER_ID: {}", trx.from_user_id)?;
        writeln!(writer, "TIMESTAMP: {}", trx.timestamp)?;
        writeln!(writer, "DESCRIPTION: \"{}\"", trx.description)?;
        writeln!(writer, "TX_ID: {}", trx.tx_id)?;
        writeln!(writer, "AMOUNT: {}", trx.amount)?;
        writeln!(writer, "STATUS: {}", trx.status)?;

        if i < transactions.len() - 1 {
            writeln!(writer)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_txt_record() {
        let lines = vec![
            "TX_TYPE: DEPOSIT".to_string(),
            "TO_USER_ID: 9223372036854775807".to_string(),
            "FROM_USER_ID: 0".to_string(),
            "TIMESTAMP: 1633036860000".to_string(),
            "DESCRIPTION: \"Record number 1\"".to_string(),
            "TX_ID: 1000000000000000".to_string(),
            "AMOUNT: 100".to_string(),
            "STATUS: FAILURE".to_string(),
        ];

        let trx = parse_txt_record(&lines, 1).unwrap();
        assert_eq!(trx.tx_id, 1000000000000000);
        assert_eq!(trx.tx_type, TxType::Deposit);
        assert_eq!(trx.from_user_id, 0);
        assert_eq!(trx.to_user_id, 9223372036854775807);
        assert_eq!(trx.amount, 100);
        assert_eq!(trx.timestamp, 1633036860000);
        assert_eq!(trx.status, TxStatus::Failure);
        assert_eq!(trx.description, "Record number 1");
    }

    #[test]
    fn test_read_txt() {
        let data = r#"# Record 1 (DEPOSIT)
            TX_TYPE: DEPOSIT
            TO_USER_ID: 9223372036854775807
            FROM_USER_ID: 0
            TIMESTAMP: 1633036860000
            DESCRIPTION: "Record number 1"
            TX_ID: 1000000000000000
            AMOUNT: 100
            STATUS: FAILURE

            # Record 2 (TRANSFER)
            DESCRIPTION: "Record number 2"
            TIMESTAMP: 1633036920000
            STATUS: PENDING
            AMOUNT: 200
            TX_ID: 1000000000000001
            TX_TYPE: TRANSFER
            FROM_USER_ID: 9223372036854775807
            TO_USER_ID: 9223372036854775807"#;

        let reader = std::io::Cursor::new(data);
        let transactions = read_txt(reader).unwrap();

        assert_eq!(transactions.len(), 2);

        assert_eq!(transactions[0].tx_id, 1000000000000000);
        assert_eq!(transactions[0].tx_type, TxType::Deposit);
        assert_eq!(transactions[0].description, "Record number 1");

        assert_eq!(transactions[1].tx_id, 1000000000000001);
        assert_eq!(transactions[1].tx_type, TxType::Transfer);
        assert_eq!(transactions[1].status, TxStatus::Pending);
        assert_eq!(transactions[1].description, "Record number 2");
    }
}
