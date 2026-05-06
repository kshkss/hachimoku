use crate::domain::{BookId, BookSlug, CabinetId, CabinetName, UnitId, Result, DomainError, HasId, AccountId, CategoryId, Relation};
use crate::domain::unit::Unit;
use crate::domain::account::Account;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub struct Book {
    id: BookId,
    slug: BookSlug,
    base_unit: UnitId,
    name: String,
    cabinet: CabinetId,
}

impl HasId for Book {
    type Id = BookId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Book {
    pub fn has_id(&self, id: BookId) -> bool {
        self.id == id
    }

    pub fn reconstruct(id: BookId, slug: BookSlug, base_unit: UnitId, name: String, cabinet: CabinetId) -> Self {
        Self { id, slug, base_unit, name, cabinet }
    }

    pub fn create_unit(&self, code: String, symbol: String, name: String, precision: i32) -> Result<Unit> {
        if precision < 0 {
            return Err(DomainError::InvalidUnitPrecision);
        }
        if code.is_empty() || name.is_empty() {
             return Err(DomainError::System);
        }
        Ok(Unit::reconstruct(
            UnitId::new(Uuid::now_v7()),
            code,
            symbol,
            name,
            precision,
            self.id,
        ))
    }

    pub fn create_account(&self, name: String, category_id: CategoryId, unit_id: UnitId) -> Result<Account> {
        if name.is_empty() || category_id.as_str().is_empty() || unit_id.as_uuid().is_nil() {
             return Err(DomainError::System);
        }
        Ok(Account::reconstruct(
            AccountId::new(Uuid::now_v7()),
            name,
            Relation::Unloaded(category_id),
            unit_id,
            self.id,
        ))
    }
}

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn get_book_by_slug(&self, cabinet: &CabinetName, slug: &BookSlug) -> Result<Book>;
    async fn get_book_by_id(&self, book: BookId) -> Result<Book>;
    async fn save_book(&self, book: &Book) -> Result<()>;
    async fn get_books(&self, cabinet: CabinetId) -> Result<Vec<Book>>;
}
