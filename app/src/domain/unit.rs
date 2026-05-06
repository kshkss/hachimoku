use crate::domain::{UnitId, BookId, Result, HasId};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Unit {
    id: UnitId,
    code: String,
    symbol: String,
    name: String,
    precision: i32,
    book: BookId,
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

    pub fn reconstruct(id: UnitId, code: String, symbol: String, name: String, precision: i32, book: BookId) -> Self {
        Self { id, code, symbol, name, precision, book }
    }
}

#[async_trait]
pub trait UnitRepository: Send + Sync {
    async fn get_units(&self, book: BookId) -> Result<Vec<Unit>>;
    async fn save_unit(&self, unit: &Unit) -> Result<()>;
}
