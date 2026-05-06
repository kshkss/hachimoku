use crate::domain::book::Book;
use crate::domain::{BookId, DomainError, Entity, Result, UnitId};
use async_trait::async_trait;
use uuid::Uuid;

/// Unit は取引で使用される単位（通貨など）を表すドメインエンティティです。
#[derive(Debug)]
pub struct Unit {
    pub(super) id: UnitId,
    /// Code は単位を識別するコードで、帳簿内で一意である必要があります。
    pub(super) code: String,
    pub(super) symbol: String,
    /// Name は単位の名称で、帳簿内で一意である必要があります。
    pub(super) name: String,
    pub(super) precision: i32,
    /// Book はこの単位を所有する帳簿の ID です。
    pub(super) book: BookId,
}

impl Entity for Unit {
    type Id = UnitId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Unit {
    /// インフラ層などで既存のデータから Unit を復元します。
    pub fn reconstruct(
        id: UnitId,
        code: String,
        symbol: String,
        name: String,
        precision: i32,
        book: BookId,
    ) -> Result<Self> {
        Ok(Self {
            id,
            code,
            symbol,
            name,
            precision,
            book,
        })
    }
}

impl Book {
    /// 帳簿が所有する新しい Unit を作成します。
    /// precision は小数点以下の表示桁数を表します。負の値が指定された場合はエラーを返します。
    pub fn create_unit(
        &self,
        code: String,
        symbol: String,
        name: String,
        precision: i32,
    ) -> Result<Unit> {
        if precision < 0 {
            return Err(DomainError::InvalidUnitPrecision);
        }
        if code.is_empty() || name.is_empty() {
            return Err(DomainError::System);
        }
        use crate::domain::Entity;
        Ok(Unit {
            id: UnitId::new(Uuid::now_v7()),
            code,
            symbol,
            name,
            precision,
            book: self.id(),
        })
    }
}

#[async_trait]
pub trait UnitRepository: Send + Sync {
    /// 指定された Id の Unit を返します。
    async fn get_unit(&self, book: UnitId) -> Result<Vec<Unit>>;
    /// Unit 情報を作成または更新します。
    async fn save_unit(&self, unit: &Unit) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BookSlug;

    #[test]
    fn test_create_unit_validation() {
        let cab_id = crate::domain::CabinetId::new(Uuid::now_v7());
        let book_id = BookId::new(Uuid::now_v7());
        let book = Book::reconstruct(
            book_id,
            BookSlug::new("slug".to_string()).unwrap(),
            UnitId::new(Uuid::now_v7()),
            "Book".to_string(),
            cab_id,
        )
        .unwrap();

        // 正常系：有効な単位が作成できることを確認
        assert!(
            book.create_unit("USD".to_string(), "$".to_string(), "Dollar".to_string(), 2)
                .is_ok()
        );

        // 異常系：精密度の負数は許可されないことを確認
        let res = book.create_unit("USD".to_string(), "$".to_string(), "Dollar".to_string(), -1);
        assert!(matches!(res, Err(DomainError::InvalidUnitPrecision)));

        // 異常系：コードや名称が空の場合はエラーとなることを確認
        assert!(
            book.create_unit("".to_string(), "$".to_string(), "Dollar".to_string(), 2)
                .is_err()
        );
        assert!(
            book.create_unit("USD".to_string(), "$".to_string(), "".to_string(), 2)
                .is_err()
        );
    }

    #[test]
    fn test_unit_reconstruct() {
        let id = UnitId::new(Uuid::now_v7());
        let book_id = BookId::new(Uuid::now_v7());
        // 正常系：インフラ層からの復元（reconstruct）が正しく行われることを確認
        let unit = Unit::reconstruct(
            id,
            "JPY".to_string(),
            "¥".to_string(),
            "Yen".to_string(),
            0,
            book_id,
        )
        .unwrap();
        assert!(unit.has_id(id));
    }
}
