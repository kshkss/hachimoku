use crate::domain;
use crate::usecase::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};

pub struct ServerInfo {
    pub base_url: String,
    pub port: String,
    pub protocol: String,
}

pub struct UserInfo {
    pub id: domain::AccountId,
    pub name: String,
    pub email: EMailAddress,
}

pub struct EMailAddress(String);

pub enum BookRole {
    Readonly,
    Writable,
}

pub struct SyncStatus {
    pub server_location: ServerInfo,
    pub last_synced_at: DateTime<Utc>,
}

pub struct BookEntry {
    pub id: domain::BookId,
    pub slug: String,
    pub name: String,
    pub role: BookRole,
    pub cabinet_id: domain::CabinetId,
    pub cabinet_name: String,
    pub cabinet_type: domain::CabinetType,
    pub unit_id: domain::UnitId,
    pub unit_code: String,
    pub unit_name: String,
    pub unit_symbol: String,
    pub revision: u32,
    pub sync_status: Option<SyncStatus>,
}

pub struct AccountEntry {
    pub id: domain::AccountId,
    pub name: String,
    pub category_id: domain::CategoryId,
    pub major_category: domain::MajorCategory,
    pub minor_category: String,
    pub num_entries: usize,
    /// num_raw_transactionsは"未分類"Accountと関連するTransactionの数
    pub num_raw_transactions: usize,
    pub deprecated: bool,
    pub book_id: domain::BookId,
    pub book_slug: domain::BookSlug,
    pub book_name: String,
    pub book_role: BookRole,
}

#[async_trait]
pub trait BookService: Send + Sync {
    async fn list_books(&self) -> domain::Result<Vec<BookEntry>>;
    async fn find_book(
        &self,
        cabinet_name: domain::CabinetName,
        book_slug: domain::BookSlug,
    ) -> domain::Result<BookEntry>;
    async fn list_accounts(&self, book: &domain::BookId) -> Result<Vec<AccountEntry>>;
}

pub struct AccountReport {
    pub id: domain::AccountId,
    pub name: String,
    pub category_id: domain::CategoryId,
    pub major_category: domain::MajorCategory,
    pub minor_category: String,
    pub unit_id: domain::UnitId,
    pub unit_code: String,
    pub unit_name: String,
    pub unit_symbol: String,
    pub precision: u8,
    pub debit: domain::RawUint,
    pub credit: domain::RawUint,
}

pub struct Filter {
    from: NaiveDate,
    to: NaiveDate,
    source: Option<domain::AccountId>,
    destination: Option<domain::AccountId>,
}

impl Filter {
    pub fn new(from: NaiveDate, to: NaiveDate) -> Self {
        Self {
            from,
            to,
            source: None,
            destination: None,
        }
    }

    pub fn iter_days(&self) -> impl Iterator<Item = NaiveDate> {
        self.from.iter_days().take_while(|&d| d <= self.to)
    }
}

#[async_trait]
pub trait AccountService: Send + Sync {
    async fn find_accounts(
        &self,
        accounts: &[domain::AccountId],
        filter: &Filter,
    ) -> Result<Vec<AccountReport>>;
}

pub struct TransactionReport {
    pub id: domain::TransactionId,
    pub date: NaiveDate,
    pub note: String,
    pub memo: String,
    pub debit: domain::RawUint,
    pub credit: domain::RawUint,
    pub unit_id: domain::UnitId,
    pub unit_code: String,
    pub unit_name: String,
    pub unit_symbol: String,
    pub precision: u8,
    pub account_id: domain::AccountId,
    pub account_name: String,
}

#[async_trait]
pub trait TransactionService: Send + Sync {
    async fn find_transactions(
        &self,
        account: &[domain::AccountId],
        filter: &Filter,
    ) -> Result<Vec<TransactionReport>>;
}

pub struct Entry {
    pub id: domain::EntryId,
    pub date: Option<NaiveDate>,
    pub memo: Option<String>,
    pub debit: domain::RawUint,
    pub credit: domain::RawUint,
    pub unit_id: domain::UnitId,
    pub unit_code: String,
    pub unit_name: String,
    pub unit_symbol: String,
    pub precision: u8,
    pub account_id: domain::AccountId,
    pub account_name: String,
    pub transaction_id: domain::TransactionId,
    pub transaction_note: String,
    pub transaction_date: NaiveDate,
}

#[async_trait]
pub trait EntryService: Send + Sync {
    async fn find_entries(&self, book: &domain::TransactionId) -> Result<Vec<Entry>>;
}

pub struct UnitEntry {
    pub id: domain::UnitId,
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub precision: u8,
    pub accounts: Vec<domain::AccountId>,
    pub base_unit: bool,
    pub deprecated: bool,
}

#[async_trait]
pub trait UnitService: Send + Sync {
    async fn list_units(&self, book: &domain::BookId) -> Result<Vec<UnitEntry>>;
}

pub enum Message {
    Unit(Vec<domain::Unit>),
    Account(Vec<domain::Account>),
    Transaction(domain::Journal),
}

pub struct Log {
    book_id: domain::BookId,
    revision: u32,
    edited_by: uuid::Uuid,
    update: Message,
}

#[async_trait]
pub trait BookStore: Send + Sync {
    async fn register_book(&self, server: &ServerInfo, book: &domain::Book) -> Result<()>;
    async fn remote_books(&self, server: &ServerInfo) -> Result<Vec<BookEntry>>;
    /// fetch_updateは共有サーバーの最新の更新を取得する
    async fn fetch_update(&self, book: &BookEntry) -> Result<Vec<Log>>;
    /// push_updateは共有サーバーの最新の更新を取得する
    async fn push_update(&self, book: &BookEntry, update: Message) -> Result<Log>;
    /// begin_updateは共有サーバーの更新をローカルに反映するためのロックを取得する
    ///
    /// *基本的な使い方*
    /// リモートの更新を反映：fetch_update() -> begin_update() -> save_*() -> commit()
    /// ローカルの更新を反映：push_update() -> begin_update() -> save_*() -> commit()
    async fn begin_update(
        &self,
    ) -> Result<(Box<dyn domain::RevisionId>, Box<dyn UpdateTransaction>)>;
}

#[async_trait]
pub trait UpdateTransaction: Send + Sync {
    async fn unit_repository(&self) -> &dyn domain::UnitRepository;
    async fn account_repository(&self) -> &dyn domain::AccountRepository;
    async fn transaction_repository(&self) -> &dyn domain::TransactionRepository;
    /// commitはローカルに反映できた更新を記録する。
    ///
    /// *基本的な使い方*
    /// リモートの更新を反映：fetch_update() -> begin_update() -> save_*() -> commit()
    /// ローカルの更新を反映：push_update() -> begin_update() -> save_*() -> commit()
    ///
    /// 反映できていない更新をcommit()した場合、CommitPreconditionFailedエラーが返される。
    /// このときローカルDBは直前の`begin_update()`の時点までロールバックされる。
    async fn commit(self, book: &BookEntry, updates: Log) -> Result<()>;
    /// rollbackはローカルDBを直前の`begin_update()`の時点までロールバックする。
    async fn rollback(self) -> Result<()>;
}

type UserName = domain::CabinetName;

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn device_id(&self) -> Result<uuid::Uuid>;
    async fn get_server(&self) -> Result<ServerInfo>;
    async fn reserve_username(&self, server: &ServerInfo, user_name: &UserName) -> Result<()>;
    async fn register_user(&self, server: &ServerInfo, user_info: &UserInfo) -> Result<()>;
    async fn get_user(&self, server: &ServerInfo) -> Result<UserInfo>;
    //async fn invite_viewer(&self, book: &BookEntry, email: EMail) -> Result<()>;
    async fn invite_editor(&self, book: &BookEntry, email: EMailAddress) -> Result<()>;
    async fn list_editor(&self, book: &BookEntry) -> Result<Vec<UserInfo>>;
    async fn remove_editor(&self, book: &BookEntry, editor: &UserInfo) -> Result<Vec<UserInfo>>;
}
