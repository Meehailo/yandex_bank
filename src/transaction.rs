use std::{fmt::Display, str::FromStr};

use crate::ParserError;

/// Транзакция в банковской системе
///
/// Представляет одну операцию перевода, пополнения или вывода средств
///
/// Поле `timestamp` хранится в Unix time в миллисекундах
#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    /// Айди транзакции
    pub tx_id: i64,
    /// Тип транзакции
    pub tx_type: TxType,
    /// Айди отправителя
    pub from_user_id: i64,
    /// Айди получателя
    pub to_user_id: i64,
    /// Сумма
    pub amount: i64,
    /// Время создания
    pub timestamp: i64,
    /// Статус транзакции
    pub status: TxStatus,
    /// Описание
    pub description: String,
}

/// Статус обработки транзакции
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TxStatus {
    /// Завершено
    Success = 0,
    /// Ошибка переводи
    Failure = 1,
    /// В процессе
    Pending = 2,
}

/// Тип транзакции
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TxType {
    /// Пополнение
    Deposit = 0,
    /// Перевод
    Transfer = 1,
    /// Снятие
    Withdrawal = 2,
}

impl TryFrom<u8> for TxType {
    type Error = ParserError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::Transfer),
            2 => Ok(Self::Withdrawal),
            _ => Err(ParserError::InvalidFormat(format!(
                "invalid tx_type code: {}",
                code
            ))),
        }
    }
}

impl TryFrom<u8> for TxStatus {
    type Error = ParserError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            2 => Ok(Self::Pending),
            _ => Err(ParserError::InvalidFormat(format!(
                "invalid tx_type code: {}",
                code
            ))),
        }
    }
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
