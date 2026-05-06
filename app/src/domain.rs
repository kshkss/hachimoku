pub mod account;
pub mod book;
pub mod cabinet;
pub mod category;
pub mod transaction;
pub mod unit;

use regex::Regex;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;
use thiserror::Error;
use uuid::Uuid;

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
pub struct CabinetId(Uuid);
impl CabinetId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookId(Uuid);
impl BookId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(Uuid);
impl AccountId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryId(String);
impl CategoryId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitId(Uuid);
impl UnitId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(Uuid);
impl TransactionId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryId(Uuid);
impl EntryId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Relation<I, T> {
    Unloaded(I),
    Loaded(T),
}

impl<I, T> Relation<I, T> {
    pub fn id(&self) -> I
    where
        I: Clone,
        T: HasId<Id = I>,
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

static SLUG_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabinetName(String);

impl CabinetName {
    pub fn new(name: String) -> Result<Self> {
        if SLUG_PATTERN.is_match(&name) && name.len() >= 3 && name.len() <= 20 {
            Ok(Self(name))
        } else {
            Err(DomainError::System)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BookSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount {
    value: i32,
    unit: UnitId,
}

impl Amount {
    pub fn new(value: i32, unit: UnitId) -> Result<Self> {
        if value == 0 {
            return Err(DomainError::InvalidEntry);
        }
        Ok(Self { value, unit })
    }

    pub fn reconstruct(amount: i32, unit: UnitId) -> Self {
        Self { value, unit }
    }

    pub fn unit(&self) -> UnitId {
        self.unit
    }

    pub fn debit(&self) -> i32 {
        self.value.max(0)
    }

    pub fn credit(&self) -> u32 {
        (-self.value).max(0)
    }

    pub fn neg(&self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }

    pub fn is_balanced(&self) -> bool {
        self.value == 0
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl std::ops::Add for Amount {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if self.unit != other.unit {
            panic!("cannot add amounts with different units");
        }
        Self {
            value: self.value + other.value,
            unit: self.unit,
        }
    }
}

pub const CATEGORY_UNCLASSIFIED: &str = "eq_unclassified";
