use crate::domain::{UnitId, BookId, Result, DomainError, HasId};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: UnitId,
    pub code: String,
    pub symbol: String,
    pub name: String,
    pub precision: i32,
    pub book: BookId,
}

impl HasId for Unit {
    type Id = UnitId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Unit {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_nil() || self.code.is_empty() || self.name.is_empty() || self.book.0.is_nil() {
            return Err(DomainError::System);
        }
        if self.precision < 0 {
            return Err(DomainError::System);
        }
        Ok(())
    }
}

#[async_trait]
pub trait UnitRepository: Send + Sync {
    async fn get_units(&self, book: BookId) -> Result<Vec<Unit>>;
    async fn save_unit(&self, unit: &Unit) -> Result<()>;
}
