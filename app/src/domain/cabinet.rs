use crate::domain::{CabinetId, CabinetName, CabinetType, Result, DomainError, HasId};
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub struct Cabinet {
    pub(super) id: CabinetId,
    pub(super) name: CabinetName,
    pub(super) cabinet_type: CabinetType,
}

impl HasId for Cabinet {
    type Id = CabinetId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Cabinet {
    pub fn has_id(&self, id: CabinetId) -> bool {
        self.id == id
    }

    pub fn new(id: CabinetId, name: CabinetName) -> Result<Self> {
        if id.as_uuid().is_nil() {
            return Err(DomainError::System);
        }
        Ok(Self { 
            id, 
            name, 
            cabinet_type: CabinetType::Shared,
        })
    }

    pub fn with_ownership(name: CabinetName) -> Result<Self> {
        Ok(Self {
            id: CabinetId::new(Uuid::now_v7()),
            name,
            cabinet_type: CabinetType::Owned,
        })
    }

    pub fn reconstruct(id: CabinetId, name: CabinetName, cabinet_type: CabinetType) -> Result<Self> {
        Ok(Self { id, name, cabinet_type })
    }
}

#[async_trait]
pub trait CabinetRepository: Send + Sync {
    async fn get_cabinet(&self, id: CabinetId) -> Result<Cabinet>;
    async fn save_cabinet(&self, cabinet: &Cabinet) -> Result<()>;
}
