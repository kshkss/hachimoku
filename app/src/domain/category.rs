use crate::domain::{CategoryId, MajorCategory, Result, DomainError, HasId};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Category {
    pub id: CategoryId,
    pub major: MajorCategory,
    pub minor: String,
    pub is_active: bool,
}

impl HasId for Category {
    type Id = CategoryId;
    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl Category {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_empty() || self.minor.is_empty() {
            return Err(DomainError::System);
        }
        Ok(())
    }
}

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn get_categories(&self) -> Result<Vec<Category>>;
    async fn get_active_categories(&self) -> Result<Vec<Category>>;
}

#[async_trait]
pub trait CategoryUpdater: Send + Sync {
    async fn update_categories(&self, config: Vec<Category>) -> Result<()>;
}
