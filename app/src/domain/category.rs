use crate::domain::{CategoryId, MajorCategory, Result, HasId};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Category {
    id: CategoryId,
    major: MajorCategory,
    minor: String,
    is_active: bool,
}

impl HasId for Category {
    type Id = CategoryId;
    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl Category {
    pub fn has_id(&self, id: &CategoryId) -> bool {
        &self.id == id
    }

    pub fn reconstruct(id: CategoryId, major: MajorCategory, minor: String, is_active: bool) -> Result<Self> {
        Ok(Self { id, major, minor, is_active })
    }

    pub fn major(&self) -> MajorCategory {
        self.major
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
