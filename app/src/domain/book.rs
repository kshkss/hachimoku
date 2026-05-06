use crate::domain::{BookId, BookSlug, CabinetId, CabinetName, CabinetType, UnitId, Result, DomainError, Entity, AccountId, CategoryId, Relation};
use crate::domain::cabinet::Cabinet;
use async_trait::async_trait;
use uuid::Uuid;

/// Book は一冊の帳簿を表すドメインエンティティです。
#[derive(Debug)]
pub struct Book {
    pub(super) id: BookId,
    /// Slug はキャビネット内で一意な文字列で、BookをURLなどで識別するために使用されます。
    pub(super) slug: BookSlug,
    /// BaseUnit は帳簿の基本単位です。
    pub(super) base_unit: UnitId,
    pub(super) name: String,
    /// Cabinet はこの帳簿が所属するキャビネットの ID です。
    pub(super) cabinet: CabinetId,
}

impl Entity for Book {
    type Id = BookId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Book {
    /// インフラ層などで既存のデータから Book を復元します。
    pub fn reconstruct(id: BookId, slug: BookSlug, base_unit: UnitId, name: String, cabinet: CabinetId) -> Result<Self> {
        Ok(Self { id, slug, base_unit, name, cabinet })
    }
}

impl Cabinet {
    /// 指定された Cabinet が所有する新しい Book を作成します。
    /// CabinetType が Owned でない場合、権限エラーを返します。
    pub fn create_book(&self, slug: BookSlug, name: String, base_unit: UnitId) -> Result<Book> {
        if self.cabinet_type != CabinetType::Owned {
            return Err(DomainError::PermissionDenied);
        }
        if name.is_empty() || base_unit.as_uuid().is_nil() {
             return Err(DomainError::System);
        }

        Ok(Book {
            id: BookId::new(Uuid::now_v7()),
            slug,
            base_unit,
            name,
            cabinet: self.id,
        })
    }
}

#[async_trait]
pub trait BookRepository: Send + Sync {
    /// 指定された Id を持つ Book を取得します。
    async fn get_book(&self, book: BookId) -> Result<Book>;
    /// Book 情報を保存（作成または更新）します。
    async fn save_book(&self, book: &Book) -> Result<()>;
}
