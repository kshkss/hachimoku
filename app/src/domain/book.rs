use crate::domain::{BookId, BookSlug, CabinetId, CabinetName, UnitId, Result, DomainError, HasId, AccountId, CategoryId, Relation};
use crate::domain::unit::Unit;
use crate::domain::account::Account;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Book {
    pub id: BookId,
    pub slug: BookSlug,
    pub base_unit: UnitId,
    pub name: String,
    pub cabinet: CabinetId,
}

impl HasId for Book {
    type Id = BookId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Book {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_nil() || self.base_unit.0.is_nil() || self.name.is_empty() || self.cabinet.0.is_nil() {
            return Err(DomainError::System);
        }
        Ok(())
    }

    pub fn create_unit(&self, code: String, symbol: String, name: String, precision: i32) -> Result<Unit> {
        if precision < 0 {
            return Err(DomainError::InvalidUnitPrecision);
        }
        let unit = Unit {
            id: UnitId(Uuid::now_v7()),
            code,
            symbol,
            name,
            precision,
            book: self.id,
        };
        unit.validate()?;
        Ok(unit)
    }

    pub fn create_account(&self, name: String, category_id: CategoryId, unit_id: UnitId) -> Result<Account> {
        let account = Account {
            id: AccountId(Uuid::now_v7()),
            name,
            category: Relation::Unloaded(category_id),
            unit: unit_id,
            book: self.id,
        };
        account.validate()?;
        Ok(account)
    }
}

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn get_book_by_slug(&self, cabinet: &CabinetName, slug: &BookSlug) -> Result<Book>;
    async fn get_book_by_id(&self, book: BookId) -> Result<Book>;
    async fn save_book(&self, book: &Book) -> Result<()>;
    async fn get_books(&self, cabinet: CabinetId) -> Result<Vec<Book>>;
}
