use crate::domain::{UnitId, BookId, Result, DomainError, HasId};
use crate::domain::book::Book;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub struct Unit {
    pub(super) id: UnitId,
    pub(super) code: String,
    pub(super) symbol: String,
    pub(super) name: String,
    pub(super) precision: i32,
    pub(super) book: BookId,
}

impl HasId for Unit {
    type Id = UnitId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Unit {
    pub fn has_id(&self, id: UnitId) -> bool {
        self.id == id
    }

    pub fn reconstruct(id: UnitId, code: String, symbol: String, name: String, precision: i32, book: BookId) -> Result<Self> {
        Ok(Self { id, code, symbol, name, precision, book })
    }
}

impl Book {
    pub fn create_unit(&self, code: String, symbol: String, name: String, precision: i32) -> Result<Unit> {
        if precision < 0 {
            return Err(DomainError::InvalidUnitPrecision);
        }
        if code.is_empty() || name.is_empty() {
             return Err(DomainError::System);
        }
        use crate::domain::HasId;
        Ok(Unit {
            id: UnitId::new(Uuid::now_v7()),
            code,
            symbol,
            name,
            precision,
            book: self.id(),
        })
    }
}

#[async_trait]
pub trait UnitRepository: Send + Sync {
    async fn get_units(&self, book: BookId) -> Result<Vec<Unit>>;
    async fn save_unit(&self, unit: &Unit) -> Result<()>;
}
