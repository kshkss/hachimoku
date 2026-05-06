use crate::domain::{CabinetId, CabinetName, CabinetType, DomainError, Entity, Result};
use async_trait::async_trait;
use uuid::Uuid;

/// Cabinet は帳簿（Book）の集まりを管理するエンティティです。
#[derive(Debug)]
pub struct Cabinet {
    pub(super) id: CabinetId,
    pub(super) name: CabinetName,
    pub(super) cabinet_type: CabinetType,
}

impl Entity for Cabinet {
    type Id = CabinetId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Cabinet {
    /// 新しい Cabinet を作成します。他人から共有されたときに使用することを想定しています。
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

    /// 所有権を持つ新しい Cabinet を作成します。ID は自動生成されます。
    pub fn with_ownership(name: CabinetName) -> Result<Self> {
        Ok(Self {
            id: CabinetId::new(Uuid::now_v7()),
            name,
            cabinet_type: CabinetType::Owned,
        })
    }

    /// インフラ層などで既存のデータから Cabinet を復元します。
    pub fn reconstruct(
        id: CabinetId,
        name: CabinetName,
        cabinet_type: CabinetType,
    ) -> Result<Self> {
        Ok(Self {
            id,
            name,
            cabinet_type,
        })
    }
}

#[async_trait]
pub trait CabinetRepository: Send + Sync {
    /// 指定された Id を持つ Cabinet を返します。
    async fn get_cabinet(&self, id: CabinetId) -> Result<Cabinet>;
    /// Cabinet 情報を保存します。
    async fn save_cabinet(&self, cabinet: &Cabinet) -> Result<()>;
}
