use crate::domain::{CabinetId, DomainError, Entity, Result};
use async_trait::async_trait;
use regex::Regex;
use std::fmt;
use std::sync::LazyLock;
use uuid::Uuid;

/// CabinetType はキャビネットの種類を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CabinetType {
    /// 自身が所有するキャビネット
    Owned,
    /// 他者から共有されたキャビネット
    Shared,
}

/// CabinetName はキャビネットの名称を表す値オブジェクトです。
/// 3文字以上20文字以下の英数字とハイフンのみを許容します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabinetName(String);

static NAME_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

impl CabinetName {
    pub fn new(name: String) -> Result<Self> {
        if NAME_PATTERN.is_match(&name) && name.len() >= 3 && name.len() <= 20 {
            Ok(Self(name))
        } else {
            Err(DomainError::System)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CabinetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Cabinet は帳簿（Book）の集まりを管理するエンティティです。
#[derive(Debug)]
pub struct Cabinet {
    id: CabinetId,
    name: CabinetName,
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

    /// Owned タイプの Cabinet であれば true を返します。
    pub fn is_owned(&self) -> bool {
        self.cabinet_type == CabinetType::Owned
    }

    /// Shared タイプの Cabinet であれば true を返します。
    pub fn is_shared(&self) -> bool {
        self.cabinet_type == CabinetType::Shared
    }
}

#[async_trait]
pub trait CabinetRepository: Send + Sync {
    /// 指定された Id を持つ Cabinet を返します。
    async fn get_cabinet(&self, id: CabinetId) -> Result<Cabinet>;
    /// Cabinet 情報を保存します。
    async fn save_cabinet(&self, cabinet: &Cabinet) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_cabinet_creation() {
        let name = CabinetName::new("my-cab".to_string()).unwrap();
        let id = CabinetId::new(Uuid::now_v7());

        // 正常系：new() は Shared タイプのキャビネットを作成することを確認
        let cab = Cabinet::new(id, name.clone()).unwrap();
        assert_eq!(cab.cabinet_type, CabinetType::Shared);
        assert!(cab.has_id(id));

        // 正常系：with_ownership() は Owned タイプのキャビネットを作成し、新しい ID が発行されることを確認
        let cab_owned = Cabinet::with_ownership(name).unwrap();
        assert_eq!(cab_owned.cabinet_type, CabinetType::Owned);
        assert!(!cab_owned.has_id(id));
    }

    #[test]
    fn test_cabinet_reconstruct() {
        let name = CabinetName::new("my-cab".to_string()).unwrap();
        let id = CabinetId::new(Uuid::now_v7());
        // 正常系：インフラ層からの復元（reconstruct）が正しく行われることを確認
        let cab = Cabinet::reconstruct(id, name, CabinetType::Owned).unwrap();
        assert_eq!(cab.cabinet_type, CabinetType::Owned);
        assert!(cab.has_id(id));
    }

    #[test]
    fn test_cabinet_name_validation() {
        // 正常系：許可される文字（英数字、ハイフン）と長さ（3-20）
        assert!(CabinetName::new("my-cabinet".to_string()).is_ok());
        assert!(CabinetName::new("cab123".to_string()).is_ok());
        assert!(CabinetName::new("a-b-c".to_string()).is_ok());

        // 異常系：短すぎる
        assert!(CabinetName::new("ab".to_string()).is_err());
        // 異常系：長すぎる（21文字）
        assert!(CabinetName::new("a".repeat(21)).is_err());
        // 異常系：不正な文字（大文字、アンダースコア、スペース）
        assert!(CabinetName::new("MyCabinet".to_string()).is_err());
        assert!(CabinetName::new("my_cabinet".to_string()).is_err());
        assert!(CabinetName::new("my cabinet".to_string()).is_err());
        // 異常系：ハイフンの使用法（先頭、末尾、連続）
        assert!(CabinetName::new("-abc".to_string()).is_err());
        assert!(CabinetName::new("abc-".to_string()).is_err());
        assert!(CabinetName::new("abc--def".to_string()).is_err());
    }
}
