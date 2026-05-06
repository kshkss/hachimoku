use crate::domain::{CategoryId, MajorCategory, Result, Entity};
use async_trait::async_trait;

/// Category は勘定科目のカテゴリーを表すドメインエンティティです。
#[derive(Debug)]
pub struct Category {
    /// Id はシステム全体で一意なカテゴリー識別子です。
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
    /// インフラ層などで既存のデータから Category を復元します。
    pub fn reconstruct(id: CategoryId, major: MajorCategory, minor: String, is_active: bool) -> Result<Self> {
        Ok(Self { id, major, minor, is_active })
    }

    pub fn major(&self) -> MajorCategory {
        self.major
    }
}

#[async_trait]
pub trait CategoryUpdater: Send + Sync {
    /// カテゴリーの一覧を更新します。
    /// 存在しない ID の場合は新規作成、存在する場合は更新、指定されなかった既存 ID は無効化されます。
    async fn update_categories(&self, config: &[Category]) -> Result<()>;
}
