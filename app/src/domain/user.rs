use crate::domain::{UserId, Result, DomainError, HasId, BookId, BookSlug, UnitId};
use crate::domain::book::Book;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
}

impl HasId for User {
    type Id = UserId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl User {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_nil() {
            return Err(DomainError::System);
        }
        Ok(())
    }

    pub fn create(id: UserId) -> Result<Self> {
        let user = Self { id };
        user.validate()?;
        Ok(user)
    }

    pub fn create_book(&self, slug: BookSlug, name: String, base_unit: UnitId) -> Result<Book> {
        let book = Book {
            id: BookId(Uuid::now_v7()),
            slug,
            base_unit,
            name,
            owner: self.id,
        };
        book.validate()?;
        Ok(book)
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_user(&self, id: UserId) -> Result<User>;
    async fn save_user(&self, user: &User) -> Result<()>;
    async fn get_editors(&self, book: BookId) -> Result<Vec<User>>;
}
