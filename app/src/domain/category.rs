use crate::domain::{CategoryId, DomainError, Entity, Result};
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::LazyLock;

/// MajorCategory は勘定科目の大分類を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MajorCategory {
    /// 資産
    Asset,
    /// 負債
    Liability,
    /// 純資産（資本）
    Equity,
    /// 収益
    Income,
    /// 費用
    Expense,
}

impl MajorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Liability => "liability",
            Self::Equity => "equity",
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }
}

impl FromStr for MajorCategory {
    type Err = DomainError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "asset" => Ok(Self::Asset),
            "liability" => Ok(Self::Liability),
            "equity" => Ok(Self::Equity),
            "income" => Ok(Self::Income),
            "expense" => Ok(Self::Expense),
            _ => Err(DomainError::System),
        }
    }
}

macro_rules! define_categories {
    ($($(#[$attr:meta])* $name:ident => $val:expr),* $(,)?) => {
        $(
            $(#[$attr])*
            pub static $name: LazyLock<CategoryId> = LazyLock::new(|| CategoryId::new($val.to_string()));
        )*
    };
}

define_categories! {
    CATEGORY_CASH => "as_cash",
    CATEGORY_BANK => "as_bank",
    CATEGORY_EL_MONEY => "as_electronic_money",
    CATEGORY_POINT => "as_point",
    CATEGORY_AD_PAYMENT => "as_advance_payment",
    CATEGORY_CURRENCY => "as_currency",
    CATEGORY_GROWTH_PORTFOLIO => "as_growth_portfolio",
    CATEGORY_INCOME_PORTFOLIO => "as_income_portfolio",
    CATEGORY_INFLATION_HEDGE_PORTFOLIO => "as_inflation_hedge_portfolio",
    CATEGORY_CARD => "li_card",
    CATEGORY_LOAN => "li_loan",
    CATEGORY_INSTALLMENT => "li_installment",
    CATEGORY_SHOP => "li_shop",
    CATEGORY_CARRYOVER => "eq_carryover_balance",
    /// 未分類のカテゴリー ID 定数です。
    CATEGORY_UNCLASSIFIED => "eq_unclassified",
    CATEGORY_FOREIGN_EXCHANGE => "eq_foreign_exchange",
    CATEGORY_VALUATION_GAIN_LOSS => "eq_valuation_gain_loss",
}

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
    pub fn reconstruct(
        id: CategoryId,
        major: MajorCategory,
        minor: String,
        is_active: bool,
    ) -> Result<Self> {
        Ok(Self {
            id,
            major,
            minor,
            is_active,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_reconstruct() {
        let id = CategoryId::new("cat-id".to_string());
        // 正常系：インフラ層からの復元（reconstruct）が正しく行われることを確認
        let cat =
            Category::reconstruct(id.clone(), MajorCategory::Asset, "Minor".to_string(), true)
                .unwrap();
        assert!(cat.has_id(id));
        assert_eq!(cat.major(), MajorCategory::Asset);
    }
}
