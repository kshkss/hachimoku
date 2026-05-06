use crate::domain::{AccountId, CategoryId, UnitId, BookId, Result, DomainError, Entity, Relation};
use crate::domain::book::Book;
use crate::domain::category::Category;
use async_trait::async_trait;
use uuid::Uuid;

/// Account は勘定科目を表すドメインエンティティです。
#[derive(Debug)]
pub struct Account {
    pub(super) id: AccountId,
    pub(super) name: String,
    /// Category はこの科目が属するカテゴリーです。
    pub(super) category: Relation<CategoryId, Category>,
    /// Unit はこの科目の基本単位です。
    pub(super) unit: UnitId,
    /// Book はこの科目を所有する帳簿の ID です。
    pub(super) book: BookId,
}

impl Entity for Account {
    type Id = AccountId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Account {
    /// インフラ層などで既存のデータから Account を復元します。
    pub fn reconstruct(
        id: AccountId,
        name: String,
        category: Relation<CategoryId, Category>,
        unit: UnitId,
        book: BookId,
    ) -> Result<Self> {
        Ok(Self { id, name, category, unit, book })
    }
    
    pub fn unit(&self) -> UnitId { self.unit }
    pub fn category(&self) -> &Relation<CategoryId, Category> { &self.category }
}

impl Book {
    /// 帳簿が所有する新しい Account を作成します。
    pub fn create_account(&self, name: String, category_id: CategoryId, unit_id: UnitId) -> Result<Account> {
        if name.is_empty() || category_id.as_str().is_empty() || unit_id.as_uuid().is_nil() {
             return Err(DomainError::System);
        }
        use crate::domain::Entity;
        Ok(Account {
            id: AccountId::new(Uuid::now_v7()),
            name,
            category: Relation::Unloaded(category_id),
            unit: unit_id,
            book: self.id(),
        })
    }
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// 指定された Id の Account を返します。
    async fn get_account(&self, id: AccountId) -> Result<Account>;
    /// Account 情報を保存（作成または更新）します。
    async fn save_account(&self, account: &Account) -> Result<()>;
}
