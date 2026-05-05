pub mod cabinet;
pub mod book;
pub mod category;
pub mod account;
pub mod unit;
pub mod transaction;

use thiserror::Error;
use uuid::Uuid;
use std::fmt;
use std::str::FromStr;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("system error")]
    System,
    #[error("book slug already exists for this cabinet")]
    BookSlugExists,
    #[error("invalid book slug")]
    InvalidBookSlug,
    #[error("invalid book name")]
    InvalidBookName,
    #[error("帳簿が見つかりません")]
    BookNotFound,
    #[error("口座が見つかりません")]
    AccountNotFound,
    #[error("failed to decode preset_book.toml")]
    FailedToDecodePreset,
    #[error("カテゴリが見つかりません")]
    CategoryNotFound,
    #[error("単位が見つかりません")]
    UnitNotFound,
    #[error("unit code already exists for this cabinet")]
    UnitCodeExists,
    #[error("unit name already exists for this cabinet")]
    UnitNameExists,
    #[error("the specified unit is not owned by the book")]
    NotOwnedUnit,
    #[error("Precision must be zero or positive")]
    InvalidUnitPrecision,
    #[error("取引が見つかりません")]
    TransactionNotFound,
    #[error("取引の操作が無効です")]
    InvalidTransactionOperation,
    #[error("エントリーの単位が無効です")]
    InvalidEntryUnit,
    #[error("エントリーの内容が無効です")]
    InvalidEntry,
    #[error("権限がありません")]
    PermissionDenied,
    #[error("not implemented")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CabinetType {
    Owned,
    Shared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CabinetId(pub Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookId(pub Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(pub Uuid);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryId(pub String);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitId(pub Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(pub Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Relation<I, T> {
    Unloaded(I),
    Loaded(T),
}

impl<I, T> Relation<I, T> {
    pub fn id(&self) -> I 
    where 
        I: Clone,
        T: HasId<Id = I>
    {
        match self {
            Relation::Unloaded(id) => id.clone(),
            Relation::Loaded(entity) => entity.id(),
        }
    }

    pub fn entity(&self) -> Option<&T> {
        match self {
            Relation::Unloaded(_) => None,
            Relation::Loaded(entity) => Some(entity),
        }
    }
}

pub trait HasId {
    type Id;
    fn id(&self) -> Self::Id;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MajorCategory {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl MajorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Liability => "liability",
            Self::Equity => "equity",
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }
}

impl FromStr for MajorCategory {
    type Err = DomainError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "asset" => Ok(Self::Asset),
            "liability" => Ok(Self::Liability),
            "equity" => Ok(Self::Equity),
            "income" => Ok(Self::Income),
            "expense" => Ok(Self::Expense),
            _ => Err(DomainError::System),
        }
    }
}

static SLUG_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabinetName(String);

impl CabinetName {
    pub fn new(name: String) -> Result<Self> {
        if SLUG_PATTERN.is_match(&name) && name.len() >= 3 && name.len() <= 20 {
            Ok(Self(name))
        } else {
            Err(DomainError::System) // Or a more specific error if available
        }
    }
}

impl fmt::Display for CabinetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookSlug(String);

impl BookSlug {
    pub fn new(slug: String) -> Result<Self> {
        if SLUG_PATTERN.is_match(&slug) && slug.len() > 2 && slug.len() <= 20 {
            Ok(Self(slug))
        } else {
            Err(DomainError::InvalidBookSlug)
        }
    }
}

impl fmt::Display for BookSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount {
    pub value: i32,
    pub unit: UnitId,
}

impl Amount {
    pub fn new(value: i32, unit: UnitId) -> Result<Self> {
        if value == 0 {
            return Err(DomainError::InvalidEntry);
        }
        Ok(Self { value, unit })
    }

    pub fn validate(&self) -> Result<()> {
        if self.value == 0 {
            return Err(DomainError::System);
        }
        Ok(())
    }

    pub fn debit(&self) -> i32 {
        self.value.max(0)
    }

    pub fn credit(&self) -> i32 {
        (-self.value).max(0)
    }

    pub fn neg(&self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }

    pub fn is_balance(&self) -> bool {
        self.value == 0
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl std::ops::Add for Amount {
    type Output = Result<Self>;

    fn add(self, other: Self) -> Self::Output {
        if self.unit != other.unit {
            return Err(DomainError::System);
        }
        Ok(Self {
            value: self.value + other.value,
            unit: self.unit,
        })
    }
}

pub const CATEGORY_UNCLASSIFIED: &str = "eq_unclassified";
