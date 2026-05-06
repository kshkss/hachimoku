use crate::domain::{AccountId, CategoryId, UnitId, BookId, Result, DomainError, HasId, Relation};
use crate::domain::book::Book;
use crate::domain::category::Category;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub struct Account {
    pub(super) id: AccountId,
    pub(super) name: String,
    pub(super) category: Relation<CategoryId, Category>,
    pub(super) unit: UnitId,
    pub(super) book: BookId,
}

impl HasId for Account {
    type Id = AccountId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Account {
    pub fn has_id(&self, id: AccountId) -> bool {
        self.id == id
    }

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
    pub fn create_account(&self, name: String, category_id: CategoryId, unit_id: UnitId) -> Result<Account> {
        if name.is_empty() || category_id.as_str().is_empty() || unit_id.as_uuid().is_nil() {
             return Err(DomainError::System);
        }
        use crate::domain::HasId;
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
    async fn get_account_by_id(&self, account: AccountId) -> Result<Account>;
    async fn get_accounts(&self, book: BookId) -> Result<Vec<Account>>;
    async fn get_accounts_of_category(&self, book: BookId, category: CategoryId) -> Result<Vec<AccountId>>;
    async fn save_account(&self, account: &Account) -> Result<()>;
}
