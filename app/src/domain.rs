pub mod account;
pub mod book;
pub mod cabinet;
pub mod category;
pub mod transaction;
pub mod unit;

pub use account::{Account, AccountRepository};
pub use book::{Book, BookRepository};
pub use cabinet::{Cabinet, CabinetRepository};
pub use category::{Category, CategoryUpdater};
pub use transaction::{Entry, Journal, Transaction, TransactionParam, TransactionRepository};
pub use unit::{Unit, UnitRepository};

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

/// CabinetType はキャビネットの種類を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CabinetType {
    /// 自身が所有するキャビネット
    Owned,
    /// 他者から共有されたキャビネット
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

/// MajorCategory は勘定科目の大分類を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MajorCategory {
    /// 資産
    Asset,
    /// 負債
    Liability,
    /// 純資産（資本）
    Equity,
    /// 収益
    Income,
    /// 費用
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

/// CabinetName はキャビネットの名称を表す値オブジェクトです。
/// 3文字以上20文字以下の英数字とハイフンのみを許容します。
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

/// BookSlug は帳簿を識別するためのユーザーごとに一意なスラッグです。
/// 3文字以上20字以下の英数字とハイフンのみを許容します。
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

/// Amount は取引の金額と単位を保持する値オブジェクトです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount {
    value: i32,
    unit: UnitId,
}

impl Amount {
    /// 新しい Amount を作成します。金額が0の場合はエラーを返します。
    pub fn new_debit(amount: u32, unit: UnitId) -> Result<Self> {
        if amount == 0 || amount > i32::MAX as u32 {
            return Err(DomainError::InvalidEntry);
        }
        Ok(Self {
            value: amount as i32,
            unit,
        })
    }

    /// 新しい Amount を作成します。金額が0の場合はエラーを返します。
    pub fn new_credit(amount: u32, unit: UnitId) -> Result<Self> {
        if amount == 0 || amount > i32::MAX as u32 {
            return Err(DomainError::InvalidEntry);
        }
        Ok(Self {
            value: -(amount as i32),
            unit,
        })
    }

    /// インフラ層などで既存のデータから Amount を復元します。
    pub fn reconstruct(debit: u32, credit: u32, unit: UnitId) -> Result<Self> {
        const MAXINT: u32 = i32::MAX as u32;
        if debit > MAXINT || credit > MAXINT {
            Err(DomainError::System)
        } else {
            let value = debit as i32 - credit as i32;
            Ok(Self { value, unit })
        }
    }

    /// 単位の ID を返します。
    pub fn unit(&self) -> UnitId {
        self.unit
    }

    /// 借方の金額（正の値）を返します。
    pub fn debit(&self) -> u32 {
        self.value.max(0) as u32
    }

    /// 貸方の金額（正の値）を返します。
    pub fn credit(&self) -> u32 {
        (-self.value).max(0) as u32
    }

    /// 金額の符号を反転させた Amount を返します。
    pub fn neg(&self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }

    /// 金額がバランスしている（0である）かどうかを返します。
    pub fn is_balanced(&self) -> bool {
        self.value == 0
    }

    /// 金額がバランスしている（0である）かどうかを返します。
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

/// 未分類のカテゴリー ID 定数です。
pub const CATEGORY_UNCLASSIFIED: &str = "eq_unclassified";

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_cabinet_name_validation() {
        // 正常系：許可される文字（英数字、ハイフン）と長さ（3-20）
        assert!(CabinetName::new("my-cabinet".to_string()).is_ok());
        assert!(CabinetName::new("cab123".to_string()).is_ok());
        assert!(CabinetName::new("a-b-c".to_string()).is_ok());

        // 異常系：短すぎる
        assert!(CabinetName::new("ab".to_string()).is_err());
        // 異常系：長すぎる（21文字）
        assert!(CabinetName::new("a".repeat(21)).is_err());
        // 異常系：不正な文字（大文字、アンダースコア、スペース）
        assert!(CabinetName::new("MyCabinet".to_string()).is_err());
        assert!(CabinetName::new("my_cabinet".to_string()).is_err());
        assert!(CabinetName::new("my cabinet".to_string()).is_err());
        // 異常系：ハイフンの使用法（先頭、末尾、連続）
        assert!(CabinetName::new("-abc".to_string()).is_err());
        assert!(CabinetName::new("abc-".to_string()).is_err());
        assert!(CabinetName::new("abc--def".to_string()).is_err());
    }

    #[test]
    fn test_book_slug_validation() {
        // 正常系：許可される形式
        assert!(BookSlug::new("my-book".to_string()).is_ok());
        assert!(BookSlug::new("slug123".to_string()).is_ok());

        // 異常系：短すぎる（3文字未満）
        assert!(BookSlug::new("ab".to_string()).is_err());
        // 異常系：長すぎる
        assert!(BookSlug::new("a".repeat(21)).is_err());
        // 異常系：不正な文字
        assert!(BookSlug::new("MyBook".to_string()).is_err());
    }

    #[test]
    fn test_amount_operations() {
        let unit = UnitId::new(Uuid::now_v7());

        // 正常系：借方(debit)と貸方(credit)の生成と値の整合性
        let d = Amount::new_debit(100, unit).unwrap();
        assert_eq!(d.debit(), 100);
        assert_eq!(d.credit(), 0);

        let c = Amount::new_credit(50, unit).unwrap();
        assert_eq!(c.debit(), 0);
        assert_eq!(c.credit(), 50);

        // 正常系：インフラ層を想定した復元（reconstruct）とバランス判定
        let r = Amount::reconstruct(100, 150, unit).unwrap();
        assert_eq!(r.debit(), 0);
        assert_eq!(r.credit(), 50);
        assert_eq!(r.is_balanced(), false);

        // 正常系：符号反転（neg）
        let n = d.neg();
        assert_eq!(n.credit(), 100);

        // 正常系：金額の加算（バランスの推移）
        let sum = d + c;
        assert_eq!(sum.debit(), 50);
        assert_eq!(sum.credit(), 0);

        let balanced = d + d.neg();
        assert!(balanced.is_balanced());
        assert!(balanced.is_zero());
    }

    #[test]
    #[should_panic(expected = "cannot add amounts with different units")]
    fn test_amount_add_different_units_panic() {
        // 異常系：異なる単位間での加算は panic! することを確認
        let u1 = UnitId::new(Uuid::now_v7());
        let u2 = UnitId::new(Uuid::now_v7());
        let a1 = Amount::new_debit(100, u1).unwrap();
        let a2 = Amount::new_debit(100, u2).unwrap();
        let _ = a1 + a2;
    }
}
