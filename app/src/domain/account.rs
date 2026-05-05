use crate::domain::{AccountId, CategoryId, UnitId, BookId, Result, DomainError, HasId, Relation};
use crate::domain::category::Category;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: AccountId,
    pub name: String,
    pub category: Relation<CategoryId, Category>,
    pub unit: UnitId,
    pub book: BookId,
}

impl HasId for Account {
    type Id = AccountId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Account {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_nil() || self.name.is_empty() || self.unit.0.is_nil() || self.book.0.is_nil() {
            return Err(DomainError::System);
        }
        match &self.category {
            Relation::Unloaded(id) => {
                if id.0.is_empty() {
                    return Err(DomainError::System);
                }
            }
            Relation::Loaded(cat) => {
                cat.validate()?;
            }
        }
        Ok(())
    }
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn get_account_by_id(&self, account: AccountId) -> Result<Account>;
    async fn get_accounts(&self, book: BookId) -> Result<Vec<Account>>;
    async fn get_accounts_of_category(&self, book: BookId, category: CategoryId) -> Result<Vec<AccountId>>;
    async fn save_account(&self, account: &Account) -> Result<()>;
}
