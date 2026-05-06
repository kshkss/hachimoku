use crate::domain::{AccountId, CategoryId, UnitId, BookId, Result, HasId, Relation};
use crate::domain::category::Category;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Account {
    id: AccountId,
    name: String,
    category: Relation<CategoryId, Category>,
    unit: UnitId,
    book: BookId,
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
    ) -> Self {
        Self { id, name, category, unit, book }
    }
    
    pub fn unit(&self) -> UnitId { self.unit }
    pub fn category(&self) -> &Relation<CategoryId, Category> { &self.category }
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn get_account_by_id(&self, account: AccountId) -> Result<Account>;
    async fn get_accounts(&self, book: BookId) -> Result<Vec<Account>>;
    async fn get_accounts_of_category(&self, book: BookId, category: CategoryId) -> Result<Vec<AccountId>>;
    async fn save_account(&self, account: &Account) -> Result<()>;
}
