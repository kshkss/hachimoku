use crate::domain::book::Book;
use crate::domain::category::Category;
use crate::domain::{AccountId, BookId, CategoryId, DomainError, Entity, Relation, Result, UnitId};
use async_trait::async_trait;
use uuid::Uuid;

/// Account は勘定科目を表すドメインエンティティです。
#[derive(Debug)]
pub struct Account {
    id: AccountId,
    name: String,
    /// Category はこの科目が属するカテゴリーです。
    category: Relation<CategoryId, Category>,
    /// Unit はこの科目の基本単位です。
    unit: UnitId,
    /// Book はこの科目を所有する帳簿の ID です。
    book: BookId,
    is_active: bool,
}

impl Entity for Account {
    type Id = AccountId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Account {
    /// インフラ層などで既存のデータから Account を復元します。
    pub fn reconstruct(
        id: AccountId,
        name: String,
        category: Relation<CategoryId, Category>,
        unit: UnitId,
        book: BookId,
        is_active: bool,
    ) -> Result<Self> {
        Ok(Self {
            id,
            name,
            category,
            unit,
            book,
            is_active,
        })
    }

    pub fn unit(&self) -> UnitId {
        self.unit
    }
    pub fn category(&self) -> &Relation<CategoryId, Category> {
        &self.category
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Book {
    /// 帳簿が所有する新しい Account を作成します。
    pub fn create_account(
        &self,
        name: String,
        category_id: CategoryId,
        unit_id: UnitId,
    ) -> Result<Account> {
        if name.is_empty() || category_id.as_str().is_empty() || unit_id.as_uuid().is_nil() {
            return Err(DomainError::System);
        }
        use crate::domain::Entity;
        Ok(Account {
            id: AccountId::new(Uuid::now_v7()),
            name,
            category: Relation::Unloaded(category_id),
            unit: unit_id,
            book: self.id(),
            is_active: true,
        })
    }
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// 指定された Id の Account を返します。
    async fn get_account(&self, id: AccountId) -> Result<Account>;
    /// Account 情報を保存（作成または更新）します。
    async fn save_account(&self, account: &Account) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BookSlug;

    #[test]
    fn test_create_account_validation() {
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

        let cat_id = CategoryId::new("cat".to_string());
        let unit_id = UnitId::new(Uuid::now_v7());

        // 正常系：有効な口座が作成できることを確認
        assert!(
            book.create_account("Asset Account".to_string(), cat_id.clone(), unit_id)
                .is_ok()
        );

        // 異常系：名称が空の場合はエラーとなることを確認
        assert!(
            book.create_account("".to_string(), cat_id.clone(), unit_id)
                .is_err()
        );
        // 異常系：カテゴリーIDが空の場合はエラーとなることを確認
        assert!(
            book.create_account("Name".to_string(), CategoryId::new("".to_string()), unit_id)
                .is_err()
        );
    }

    #[test]
    fn test_account_reconstruct() {
        let id = AccountId::new(Uuid::now_v7());
        let cat_id = CategoryId::new("cat".to_string());
        let unit_id = UnitId::new(Uuid::now_v7());
        let book_id = BookId::new(Uuid::now_v7());
        // 正常系：インフラ層からの復元（reconstruct）が正しく行われることを確認
        let account = Account::reconstruct(
            id,
            "Account".to_string(),
            Relation::Unloaded(cat_id),
            unit_id,
            book_id,
            true,
        )
        .unwrap();
        assert!(account.has_id(id));
        assert!(account.is_active());
    }
}
