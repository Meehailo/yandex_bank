use std::{fmt::Display, str::FromStr};

use crate::ParserError;

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub tx_id: i64,
    pub tx_type: TxType,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub amount: i64,
    pub timestamp: i64,
    pub status: TxStatus,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TxStatus {
    Failure,
    Pending,
    Success,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl FromStr for TxType {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEPOSIT" => Ok(TxType::Deposit),
            "TRANSFER" => Ok(TxType::Transfer),
            "WITHDRAWAL" => Ok(TxType::Withdrawal),
            _ => Err(ParserError::InvalidTxType(s.to_string())),
        }
    }
}

impl Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxType::Deposit => write!(f, "DEPOSIT"),
            TxType::Transfer => write!(f, "TRANSFER"),
            TxType::Withdrawal => write!(f, "WITHDRAWAL"),
        }
    }
}

impl FromStr for TxStatus {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FAILURE" => Ok(TxStatus::Failure),
            "PENDING" => Ok(TxStatus::Pending),
            "SUCCESS" => Ok(TxStatus::Success),
            _ => Err(ParserError::InvalidTxStatus(s.to_string())),
        }
    }
}

impl Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxStatus::Failure => write!(f, "FAILURE"),
            TxStatus::Pending => write!(f, "PENDING"),
            TxStatus::Success => write!(f, "SUCCESS"),
        }
    }
}
