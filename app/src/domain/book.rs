use crate::domain::{BookId, BookSlug, CabinetId, CabinetName, CabinetType, UnitId, Result, DomainError, HasId, AccountId, CategoryId, Relation};
use crate::domain::cabinet::Cabinet;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub struct Book {
    pub(super) id: BookId,
    pub(super) slug: BookSlug,
    pub(super) base_unit: UnitId,
    pub(super) name: String,
    pub(super) cabinet: CabinetId,
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

    pub fn reconstruct(id: BookId, slug: BookSlug, base_unit: UnitId, name: String, cabinet: CabinetId) -> Result<Self> {
        Ok(Self { id, slug, base_unit, name, cabinet })
    }
}

impl Cabinet {
    pub fn create_book(&self, slug: BookSlug, name: String, base_unit: UnitId) -> Result<Book> {
        if self.cabinet_type != CabinetType::Owned {
            return Err(DomainError::PermissionDenied);
        }
        if name.is_empty() || base_unit.as_uuid().is_nil() {
             return Err(DomainError::System);
        }

        Ok(Book {
            id: BookId::new(Uuid::now_v7()),
            slug,
            base_unit,
            name,
            cabinet: self.id,
        })
    }
}

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn get_book_by_slug(&self, cabinet: &CabinetName, slug: &BookSlug) -> Result<Book>;
    async fn get_book_by_id(&self, book: BookId) -> Result<Book>;
    async fn save_book(&self, book: &Book) -> Result<()>;
    async fn get_books(&self, cabinet: CabinetId) -> Result<Vec<Book>>;
}
