use crate::domain;
use chrono::NaiveDate;
use thiserror::Error;

pub mod accounts;
pub mod history;
pub mod pnl;
pub mod query;
pub mod shop;

pub use query::{
    AccountEntry, AccountReport, AccountService, BookEntry, BookRole, BookService, Filter,
    TransactionReport, TransactionService,
};

pub static EPOCH_DATE: NaiveDate = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    #[error("system error")]
    System,
    #[error("domain error")]
    Domain(#[from] domain::DomainError),
}

pub type Result<T> = std::result::Result<T, ApplicationError>;
