use crate::domain::book::Book;
use crate::domain::{BookId, DomainError, Entity, Result, RevisionId, UnitId};
use async_trait::async_trait;
use uuid::Uuid;

pub const MAX_PRECISION: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawInt(pub i32);

impl RawInt {
    pub const MAX: Self = Self(i32::MAX);
    pub fn as_int(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawUint(pub u32);

impl RawUint {
    pub const MAX: Self = Self(i32::MAX as u32);
    pub fn as_uint(&self) -> u32 {
        self.0
    }
}

pub const MAX_RAW_INT: RawInt = RawInt::MAX;
pub const MAX_RAW_UINT: RawUint = RawUint::MAX;

/// Amount は取引の金額と単位を保持する値オブジェクトです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount {
    value: RawInt,
    unit: UnitId,
}

impl Amount {
    /// 新しい Amount を作成します。金額が0の場合はエラーを返します。
    pub fn new_debit(amount: RawUint, unit: UnitId) -> Result<Self> {
        if amount.as_uint() == 0 || amount > MAX_RAW_UINT {
            return Err(DomainError::InvalidEntry);
        }
        Ok(Self {
            value: RawInt(amount.as_uint() as i32),
            unit,
        })
    }

    /// 新しい Amount を作成します。金額が0の場合はエラーを返します。
    pub fn new_credit(amount: RawUint, unit: UnitId) -> Result<Self> {
        if amount.as_uint() == 0 || amount > MAX_RAW_UINT {
            return Err(DomainError::InvalidEntry);
        }
        Ok(Self {
            value: RawInt(-(amount.as_uint() as i32)),
            unit,
        })
    }

    /// インフラ層などで既存のデータから Amount を復元します。
    pub fn reconstruct(debit: RawUint, credit: RawUint, unit: UnitId) -> Result<Self> {
        if debit > MAX_RAW_UINT || credit > MAX_RAW_UINT {
            Err(DomainError::System)
        } else {
            let value = RawInt(debit.as_uint() as i32 - credit.as_uint() as i32);
            Ok(Self { value, unit })
        }
    }

    /// 単位の ID を返します。
    pub fn unit(&self) -> UnitId {
        self.unit
    }

    /// 借方の金額（正の値）を返します。
    pub fn debit(&self) -> RawUint {
        RawUint(self.value.as_int().max(0) as u32)
    }

    /// 貸方の金額（正の値）を返します。
    pub fn credit(&self) -> RawUint {
        RawUint((-self.value.as_int()).max(0) as u32)
    }

    /// 金額の符号を反転させた Amount を返します。
    pub fn neg(&self) -> Self {
        Self {
            value: RawInt(-self.value.as_int()),
            unit: self.unit,
        }
    }

    /// 金額がバランスしている（0である）かどうかを返します。
    pub fn is_balanced(&self) -> bool {
        self.value.as_int() == 0
    }

    /// 金額がバランスしている（0である）かどうかを返します。
    pub fn is_zero(&self) -> bool {
        self.value.as_int() == 0
    }
}

impl std::ops::Add for Amount {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if self.unit != other.unit {
            panic!("cannot add amounts with different units");
        }
        Self {
            value: RawInt(self.value.as_int() + other.value.as_int()),
            unit: self.unit,
        }
    }
}

/// Unit は取引で使用される単位（通貨など）を表すドメインエンティティです。
#[derive(Debug)]
pub struct Unit {
    id: UnitId,
    /// Code は単位を識別するコードで、帳簿内で一意である必要があります。
    code: String,
    symbol: String,
    /// Name は単位の名称で、帳簿内で一意である必要があります。
    name: String,
    precision: u8,
    /// Book はこの単位を所有する帳簿の ID です。
    book: BookId,
    is_active: bool,
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
        precision: u8,
        book: BookId,
        is_active: bool,
    ) -> Result<Self> {
        if precision > crate::domain::MAX_PRECISION {
            Err(DomainError::InvalidUnitPrecision)
        } else {
            Ok(Self {
                id,
                code,
                symbol,
                name,
                precision,
                book,
                is_active,
            })
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Book {
    /// 帳簿が所有する新しい Unit を作成します。
    /// precision は小数点以下の表示桁数を表し、0から8の範囲である必要があります。
    pub fn create_unit(
        &self,
        code: String,
        symbol: String,
        name: String,
        precision: u8,
    ) -> Result<Unit> {
        if precision > crate::domain::MAX_PRECISION {
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
            is_active: true,
        })
    }
}

#[async_trait]
pub trait UnitRepository: Send + Sync {
    /// 指定された Id の Unit を返します。
    async fn get_unit(&self, unit: UnitId) -> Result<Unit>;
    /// Unit 情報を作成または更新します。
    async fn save_unit(&self, revision: &dyn RevisionId, unit: &Unit) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BookSlug;

    #[test]
    fn test_create_unit_validation() {
        let cab_id = crate::domain::CabinetId::new(Uuid::now_v7());
        let book_id = BookId::new(Uuid::new_v4());
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

        // 正常系：精密度の境界値(8)が許可されることを確認
        assert!(
            book.create_unit("BTC".to_string(), "₿".to_string(), "Bitcoin".to_string(), 8)
                .is_ok()
        );

        // 異常系：精密度の最大値(8)を超える場合はエラーとなることを確認
        let res = book.create_unit(
            "LONG".to_string(),
            "L".to_string(),
            "Too long".to_string(),
            9,
        );
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
        let book_id = BookId::new(Uuid::new_v4());
        // 正常系：インフラ層からの復元（reconstruct）が正しく行われることを確認
        let unit = Unit::reconstruct(
            id,
            "JPY".to_string(),
            "¥".to_string(),
            "Yen".to_string(),
            0,
            book_id,
            true,
        )
        .unwrap();
        assert!(unit.has_id(id));
        assert!(unit.is_active());
    }

    #[test]
    fn test_amount_operations() {
        let unit = UnitId::new(Uuid::now_v7());

        // 正常系：借方(debit)と貸方(credit)の生成と値の整合性
        let d = Amount::new_debit(RawUint(100), unit).unwrap();
        assert_eq!(d.debit().as_uint(), 100);
        assert_eq!(d.credit().as_uint(), 0);

        let c = Amount::new_credit(RawUint(50), unit).unwrap();
        assert_eq!(c.debit().as_uint(), 0);
        assert_eq!(c.credit().as_uint(), 50);

        // 正常系：インフラ層を想定した復元（reconstruct）とバランス判定
        let r = Amount::reconstruct(RawUint(100), RawUint(150), unit).unwrap();
        assert_eq!(r.debit().as_uint(), 0);
        assert_eq!(r.credit().as_uint(), 50);
        assert_eq!(r.is_balanced(), false);

        // 正常系：符号反転（neg）
        let n = d.neg();
        assert_eq!(n.credit().as_uint(), 100);

        // 正常系：金額の加算（バランスの推移）
        let sum = d + c;
        assert_eq!(sum.debit().as_uint(), 50);
        assert_eq!(sum.credit().as_uint(), 0);

        let balanced = d + d.neg();
        assert!(balanced.is_balanced());
        assert!(balanced.is_zero());
    }

    #[test]
    #[should_panic(expected = "cannot add amounts with different units")]
    fn test_amount_add_different_units_panic() {
        // 異常系：異なる単位間での加算は panic! することを確認
        let u1 = UnitId::new(Uuid::now_v7());
        let u2 = UnitId::new(Uuid::now_v7());
        let a1 = Amount::new_debit(RawUint(100), u1).unwrap();
        let a2 = Amount::new_debit(RawUint(100), u2).unwrap();
        let _ = a1 + a2;
    }
}
