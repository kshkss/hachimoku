use crate::domain::{CategoryId, MajorCategory, Result, Entity};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Category {
    id: CategoryId,
    major: MajorCategory,
    minor: String,
    is_active: bool,
}

impl Entity for Category {
    type Id = CategoryId;
    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl Category {
    pub fn reconstruct(id: CategoryId, major: MajorCategory, minor: String, is_active: bool) -> Result<Self> {
        Ok(Self { id, major, minor, is_active })
    }

    pub fn major(&self) -> MajorCategory {
        self.major
    }
}

#[async_trait]
pub trait CategoryUpdater: Send + Sync {
    async fn update_categories(&self, config: &[Category]) -> Result<()>;
}
