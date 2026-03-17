use csv::StringRecord;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;

use crate::{ParserError, Transaction};

pub fn read_csv<T: Read>(reader: T) -> Result<Vec<Transaction>, ParserError> {
    let mut csv_reader = csv::Reader::from_reader(reader);
    let mut transactions = Vec::new();
    for (row_index, record_result) in csv_reader.records().enumerate() {
        let record = record_result
            .map_err(|csv_error| ParserError::CsvReadError(csv_error.to_string()))?;
        transactions.push(record_to_transaction(&record, row_index + 1)?);
    }
    Ok(transactions)
}

pub fn write_csv<T: Write>(writer: T, transactions: &[Transaction]) -> Result<(), ParserError> {
    let mut csv_writer = csv::Writer::from_writer(writer);

    csv_writer
        .write_record([
            "TX_ID",
            "TX_TYPE",
            "FROM_USER_ID",
            "TO_USER_ID",
            "AMOUNT",
            "TIMESTAMP",
            "STATUS",
            "DESCRIPTION",
        ])
        .map_err(|write_err| ParserError::CsvWriteError(write_err.to_string()))?;

    for trx in transactions {
        csv_writer
            .write_record([
                trx.tx_id.to_string(),
                trx.tx_type.to_string(),
                trx.from_user_id.to_string(),
                trx.to_user_id.to_string(),
                trx.amount.to_string(),
                trx.timestamp.to_string(),
                trx.status.to_string(),
                trx.description.to_string(),
            ])
            .map_err(|write_err| ParserError::CsvWriteError(write_err.to_string()))?;
    }
    csv_writer
        .flush()
        .map_err(|write_err| ParserError::CsvWriteError(write_err.to_string()))?;
    Ok(())
}

fn record_to_transaction(
    record: &StringRecord,
    row_index: usize,
) -> Result<Transaction, ParserError> {
    Ok(Transaction {
        tx_id: integer_value_from_record(record, 0, row_index)?,
        tx_type: enum_value_from_record(record, 1, row_index)?,
        from_user_id: integer_value_from_record(record, 2, row_index)?,
        to_user_id: integer_value_from_record(record, 3, row_index)?,
        amount: integer_value_from_record(record, 4, row_index)?,
        timestamp: integer_value_from_record(record, 5, row_index)?,
        status: enum_value_from_record(record, 6, row_index)?,
        description: string_value_from_record(record, 7, row_index)?,
    })
}

fn string_value_from_record(
    record: &StringRecord,
    i: usize,
    row_index: usize,
) -> Result<String, ParserError> {
    Ok(record
        .get(i)
        .ok_or(ParserError::MissingField(i, row_index))?
        .to_string())
}

fn integer_value_from_record<T: FromStr>(
    record: &StringRecord,
    i: usize,
    row_index: usize,
) -> Result<T, ParserError>
where
    T::Err: std::fmt::Display,
{
    record
        .get(i)
        .ok_or(ParserError::MissingField(i, row_index))?
        .parse::<T>()
        .map_err(|e| ParserError::InvalidInteger(e.to_string(), row_index))
}

fn enum_value_from_record<T: FromStr>(
    record: &StringRecord,
    i: usize,
    row_index: usize,
) -> Result<T, ParserError>
where
    T::Err: std::fmt::Display,
{
    let value = string_value_from_record(record, i, row_index)?;
    T::from_str(&value)
        .map_err(|e| ParserError::InvalidEnum(format!("{}: {}", value, e), row_index))
}

#[cfg(test)]
mod tests {
    use crate::transaction::{TxStatus, TxType};

    use super::*;

    #[test]
    fn string_value_from_record_returns_string_for_existing_field() {
        let record = StringRecord::from(vec!["value1", "value2", "value3"]);
        let result = string_value_from_record(&record, 1, 1);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, "value2");
    }

    #[test]
    fn string_value_from_record_returns_err_for_not_existing_field() {
        let record = StringRecord::from(vec!["value1"]);
        let result = string_value_from_record(&record, 3, 1);
        assert!(result.is_err());
    }

    #[test]
    fn integer_value_from_record_returns_integer_for_existing_field() {
        let record = StringRecord::from(vec!["value1", "value2", "123"]);
        let result: Result<i32, ParserError> = integer_value_from_record(&record, 2, 1);
        assert!(result.is_ok_and(|x| x == 123));
    }

    #[test]
    fn integer_value_from_record_returns_err_for_not_existing_field() {
        let record = StringRecord::from(vec!["value1"]);
        let result: Result<i32, ParserError> = integer_value_from_record(&record, 3, 1);
        assert!(result.is_err());
    }

    #[test]
    fn enum_value_from_record_returns_enum_for_existing_field() {
        let record = StringRecord::from(vec!["value1", "value2", "WITHDRAWAL"]);
        let result: Result<TxType, ParserError> = enum_value_from_record(&record, 2, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TxType::Withdrawal);
    }

    #[test]
    fn enum_value_from_record_returns_err_for_not_existing_field() {
        let record = StringRecord::from(vec!["value1"]);
        let result: Result<TxType, ParserError> = enum_value_from_record(&record, 3, 1);
        assert!(result.is_err());
    }

    #[test]
    fn record_to_transaction_returns_transaction() {
        let record = StringRecord::from(vec![
            "1000000000000000",
            "DEPOSIT",
            "0",
            "9223372036854775807",
            "100",
            "1633036860000",
            "FAILURE",
            "Record number 1",
        ]);
        let result = record_to_transaction(&record, 1);
        assert!(result.is_ok());

        let transaction = result.unwrap();
        assert_eq!(transaction.tx_id, 1000000000000000);
        assert_eq!(transaction.tx_type, TxType::Deposit);
        assert_eq!(transaction.from_user_id, 0);
        assert_eq!(transaction.to_user_id, 9223372036854775807);
        assert_eq!(transaction.amount, 100);
        assert_eq!(transaction.timestamp, 1633036860000);
        assert_eq!(transaction.status, TxStatus::Failure);
        assert_eq!(transaction.description, "Record number 1");
    }

    #[test]
    fn read_csv_returns_vec_of_transactions() {
        let mut csv_data = Vec::new();

        writeln!(
            csv_data,
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION"
        )
        .unwrap();

        writeln!(
            csv_data,
            "1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,\"описание 1\""
        )
        .unwrap();

        writeln!(
            csv_data,
            "1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,150,12346,PENDING,\"описание 2\""
        )
        .unwrap();

        let reader = std::io::Cursor::new(csv_data);
        let transactions = read_csv(reader).unwrap();

        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].tx_id, 1000000000000000);
        assert_eq!(transactions[0].tx_type, TxType::Deposit);
        assert_eq!(transactions[1].tx_id, 1000000000000001);
        assert_eq!(transactions[1].tx_type, TxType::Transfer);
    }

    #[test]
    fn write_csv_creates_csv() {
        let mut writer = Vec::new();
        let mut transactions = Vec::new();
        transactions.push(Transaction {
            tx_id: 1,
            tx_type: TxType::Deposit,
            from_user_id: 2,
            to_user_id: 3,
            amount: 4,
            timestamp: 123,
            status: TxStatus::Pending,
            description: "desc".to_string(),
        });

        assert!(write_csv(&mut writer, &transactions).is_ok());

        let result = String::from_utf8(writer).expect("Not UTF-8");
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(
            lines[0],
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION"
        );
        assert_eq!(lines[1], "1,DEPOSIT,2,3,4,123,PENDING,desc");
    }
}
