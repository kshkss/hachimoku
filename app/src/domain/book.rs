use crate::domain::cabinet::Cabinet;
use crate::domain::{
    BookId, BookSlug, CabinetId, CabinetName, CabinetType, DomainError, Entity, Result, UnitId,
};
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
    pub fn reconstruct(
        id: BookId,
        slug: BookSlug,
        base_unit: UnitId,
        name: String,
        cabinet: CabinetId,
    ) -> Result<Self> {
        Ok(Self {
            id,
            slug,
            base_unit,
            name,
            cabinet,
        })
    }
}

impl Cabinet {
    /// 指定された Cabinet が所有する新しい Book を作成します。
    /// CabinetType が Owned でない場合、権限エラーを返します。
    pub fn create_book(&self, slug: BookSlug, name: String, base_unit: UnitId) -> Result<Book> {
        if self.is_shared() {
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
            cabinet: self.id(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_create_book_permissions() {
        let name = CabinetName::new("cab".to_string()).unwrap();
        let owned_cab = Cabinet::with_ownership(name.clone()).unwrap();
        let shared_cab = Cabinet::new(CabinetId::new(Uuid::now_v7()), name).unwrap();

        let slug = BookSlug::new("my-book".to_string()).unwrap();
        let unit = UnitId::new(Uuid::now_v7());

        // 正常系：所有権を持つキャビネットからは帳簿を作成できることを確認
        assert!(
            owned_cab
                .create_book(slug.clone(), "Book Name".to_string(), unit)
                .is_ok()
        );

        // 異常系：共有されたキャビネットからは帳簿を作成できず、PermissionDenied となることを確認
        let res = shared_cab.create_book(slug, "Book Name".to_string(), unit);
        assert!(matches!(res, Err(DomainError::PermissionDenied)));
    }

    #[test]
    fn test_book_reconstruct() {
        let id = BookId::new(Uuid::now_v7());
        let slug = BookSlug::new("slug".to_string()).unwrap();
        let unit = UnitId::new(Uuid::now_v7());
        let cab_id = CabinetId::new(Uuid::now_v7());
        // 正常系：インフラ層からの復元（reconstruct）が正しく行われることを確認
        let book = Book::reconstruct(id, slug, unit, "Name".to_string(), cab_id).unwrap();
        assert!(book.has_id(id));
    }
}
