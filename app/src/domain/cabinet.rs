use crate::domain::book::Book;
use crate::domain::{
    BookId, BookSlug, CabinetId, CabinetName, CabinetType, DomainError, HasId, Result, UnitId,
};
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Cabinet {
    pub id: CabinetId,
    pub name: CabinetName,
    pub cabinet_type: CabinetType,
}

impl HasId for Cabinet {
    type Id = CabinetId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Cabinet {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_nil() {
            return Err(DomainError::System);
        }
        Ok(())
    }

    pub fn new(id: CabinetId, name: CabinetName) -> Result<Self> {
        let cabinet = Self {
            id,
            name,
            cabinet_type: CabinetType::Shared,
        };
        cabinet.validate()?;
        Ok(cabinet)
    }

    pub fn with_ownership(name: CabinetName) -> Result<Self> {
        let cabinet = Self {
            id: CabinetId(Uuid::now_v7()),
            name,
            cabinet_type: CabinetType::Owned,
        };
        cabinet.validate()?;
        Ok(cabinet)
    }

    pub fn create_book(&self, slug: BookSlug, name: String, base_unit: UnitId) -> Result<Book> {
        if self.cabinet_type != CabinetType::Owned {
            return Err(DomainError::PermissionDenied);
        }

        let book = Book {
            id: BookId(Uuid::now_v7()),
            slug,
            base_unit,
            name,
            cabinet: self.id,
        };
        book.validate()?;
        Ok(book)
    }
}

#[async_trait]
pub trait CabinetRepository: Send + Sync {
    async fn get_cabinet(&self, id: CabinetId) -> Result<Cabinet>;
    async fn save_cabinet(&self, cabinet: &Cabinet) -> Result<()>;
}
