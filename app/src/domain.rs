pub mod account;
pub mod book;
pub mod cabinet;
pub mod category;
pub mod transaction;
pub mod unit;

pub use account::{Account, AccountRepository};
pub use book::{Book, BookRepository, BookSlug};
pub use cabinet::{Cabinet, CabinetName, CabinetRepository, CabinetType};
pub use category::{Category, CategoryUpdater, MajorCategory};
pub use transaction::{Entry, Journal, Transaction, TransactionParam, TransactionRepository};
pub use unit::{
    Amount, MAX_PRECISION, MAX_RAW_INT, MAX_RAW_UINT, RawInt, RawUint, Unit, UnitRepository,
};

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
    #[error("Precision must be between 0 and 8")]
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

/// Relation は、IDのみを持つ（未読み込み）か、エンティティ全体を持つ（読み込み済み）かの状態を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Relation<I, T> {
    /// IDのみを保持する未読み込み状態
    Unloaded(I),
    /// エンティティの実体を保持する読み込み済み状態
    Loaded(T),
}

impl<I, T> Relation<I, T> {
    /// 保持している ID を返します。Loaded の場合はエンティティから ID を取得します。
    pub fn id(&self) -> I
    where
        I: Clone,
        T: Entity<Id = I>,
    {
        match self {
            Relation::Unloaded(id) => id.clone(),
            Relation::Loaded(entity) => entity.id(),
        }
    }

    /// Loaded 状態の場合にのみ、エンティティへの参照を返します。
    pub fn entity(&self) -> Option<&T> {
        match self {
            Relation::Unloaded(_) => None,
            Relation::Loaded(entity) => Some(entity),
        }
    }
}

/// Entity は、識別子（ID）を持つドメインエンティティを表すトレイトです。
pub trait Entity {
    type Id: PartialEq;
    /// エンティティの ID を返します。
    fn id(&self) -> Self::Id;
    /// 指定された ID がこのエンティティの ID と一致するかどうかを判定します。
    fn has_id(&self, id: Self::Id) -> bool {
        self.id() == id
    }
}

/// RevisionId は、エンティティのリビジョンを表すトレイトです。
pub trait RevisionId {
    fn as_u32(&self) -> u32;
}
