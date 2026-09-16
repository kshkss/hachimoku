use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain;
use crate::usecase::{
    self, AccountEntry, AccountReport, AccountService, BookEntry, BookRole, BookService, Filter,
    TransactionReport, TransactionService,
};

/// 状態を一切持たないインメモリ・モック実装。
///
/// usecase のサービストレイトを実装するインフラ層がまだ存在しないため、
/// axum のハンドラが実際に HTML を返せるように固定のフィクスチャデータを返す。
/// 日付フィルタ（`Filter`）は `usecase` モジュールの外からは中身を読めない
/// （フィールドが非公開のため）ので、渡された口座ID一覧でのみ絞り込む。
pub struct MockServices;

fn jpy_unit_id() -> domain::UnitId {
    domain::UnitId::new(Uuid::from_u128(200))
}

fn account_ids() -> [domain::AccountId; 8] {
    [1, 2, 3, 4, 5, 6, 7, 8].map(|n| domain::AccountId::new(Uuid::from_u128(n)))
}

fn fixture_accounts() -> Vec<AccountEntry> {
    let ids = account_ids();
    let book_id = domain::BookId::new(Uuid::from_u128(100));
    let book_slug = domain::BookSlug::new("demo-book".to_string()).unwrap();

    let entry = |id, name: &str, category_id: domain::CategoryId, major, minor: &str, num_entries| {
        AccountEntry {
            id,
            name: name.to_string(),
            category_id,
            major_category: major,
            minor_category: minor.to_string(),
            num_entries,
            num_raw_transactions: 0,
            deprecated: false,
            book_id,
            book_slug: book_slug.clone(),
            book_name: "デモ帳簿".to_string(),
            book_role: BookRole::Writable,
        }
    };

    vec![
        entry(
            ids[0],
            "財布(現金)",
            domain::category::CASH.clone(),
            domain::MajorCategory::Asset,
            "現金",
            5,
        ),
        entry(
            ids[1],
            "三菱UFJ銀行",
            domain::category::BANK.clone(),
            domain::MajorCategory::Asset,
            "銀行",
            8,
        ),
        entry(
            ids[2],
            "楽天カード",
            domain::category::SHOP.clone(),
            domain::MajorCategory::Liability,
            "カード",
            3,
        ),
        entry(
            ids[3],
            "給与",
            domain::CategoryId::new("in_salary".to_string()),
            domain::MajorCategory::Income,
            "給与",
            2,
        ),
        entry(
            ids[4],
            "副業収入",
            domain::CategoryId::new("in_side".to_string()),
            domain::MajorCategory::Income,
            "副業",
            1,
        ),
        entry(
            ids[5],
            "食費",
            domain::CategoryId::new("ex_food".to_string()),
            domain::MajorCategory::Expense,
            "食費",
            4,
        ),
        entry(
            ids[6],
            "日用品",
            domain::CategoryId::new("ex_daily".to_string()),
            domain::MajorCategory::Expense,
            "日用品",
            2,
        ),
        entry(
            ids[7],
            "家賃",
            domain::CategoryId::new("ex_rent".to_string()),
            domain::MajorCategory::Expense,
            "住居",
            1,
        ),
    ]
}

fn fixture_reports() -> Vec<AccountReport> {
    let ids = account_ids();
    let unit_id = jpy_unit_id();

    let report =
        |id, name: &str, category_id: domain::CategoryId, major, minor: &str, debit, credit| {
            AccountReport {
                id,
                name: name.to_string(),
                category_id,
                major_category: major,
                minor_category: minor.to_string(),
                unit_id,
                unit_code: "JPY".to_string(),
                unit_name: "円".to_string(),
                unit_symbol: "¥".to_string(),
                precision: 0,
                debit,
                credit,
            }
        };

    vec![
        report(
            ids[0],
            "財布(現金)",
            domain::category::CASH.clone(),
            domain::MajorCategory::Asset,
            "現金",
            12_500,
            0,
        ),
        report(
            ids[1],
            "三菱UFJ銀行",
            domain::category::BANK.clone(),
            domain::MajorCategory::Asset,
            "銀行",
            185_000,
            0,
        ),
        report(
            ids[2],
            "楽天カード",
            domain::category::SHOP.clone(),
            domain::MajorCategory::Liability,
            "カード",
            0,
            45_000,
        ),
        report(
            ids[3],
            "給与",
            domain::CategoryId::new("in_salary".to_string()),
            domain::MajorCategory::Income,
            "給与",
            0,
            245_000,
        ),
        report(
            ids[4],
            "副業収入",
            domain::CategoryId::new("in_side".to_string()),
            domain::MajorCategory::Income,
            "副業",
            0,
            5_000,
        ),
        report(
            ids[5],
            "食費",
            domain::CategoryId::new("ex_food".to_string()),
            domain::MajorCategory::Expense,
            "食費",
            15_000,
            0,
        ),
        report(
            ids[6],
            "日用品",
            domain::CategoryId::new("ex_daily".to_string()),
            domain::MajorCategory::Expense,
            "日用品",
            5_000,
            0,
        ),
        report(
            ids[7],
            "家賃",
            domain::CategoryId::new("ex_rent".to_string()),
            domain::MajorCategory::Expense,
            "住居",
            10_000,
            0,
        ),
    ]
}

fn fixture_transactions() -> Vec<TransactionReport> {
    let ids = account_ids();
    let unit_id = jpy_unit_id();

    let tx = |id, date, note: &str, debit, credit, account_id, account_name: &str| {
        TransactionReport {
            id,
            date,
            note: note.to_string(),
            memo: String::new(),
            debit,
            credit,
            unit_id,
            unit_code: "JPY".to_string(),
            unit_name: "円".to_string(),
            unit_symbol: "¥".to_string(),
            precision: 0,
            account_id,
            account_name: account_name.to_string(),
        }
    };

    // NOTE: usecase::history::history() は一覧モード（account指定なし）では
    // 収益・費用・未分類カテゴリの口座IDだけを対象にフィルタする（資産・負債の
    // 口座は対象外）。このモックも同じ視点（収支カテゴリ側の口座）で
    // account_id を付与する。資産・負債口座（財布/銀行/カード）を指定した
    // 個別履歴では今のところ何も表示されない（不完全な点として既知）。
    vec![
        tx(
            domain::TransactionId::new(Uuid::from_u128(1)),
            NaiveDate::from_ymd_opt(2026, 9, 20).unwrap(),
            "今月分給与",
            245_000,
            0,
            ids[3],
            "給与",
        ),
        tx(
            domain::TransactionId::new(Uuid::from_u128(2)),
            NaiveDate::from_ymd_opt(2026, 9, 22).unwrap(),
            "スーパー(食品と日用品)",
            4_500,
            0,
            ids[5],
            "食費",
        ),
        tx(
            domain::TransactionId::new(Uuid::from_u128(3)),
            NaiveDate::from_ymd_opt(2026, 9, 25).unwrap(),
            "家賃引き落とし",
            10_000,
            0,
            ids[7],
            "家賃",
        ),
        tx(
            domain::TransactionId::new(Uuid::from_u128(4)),
            NaiveDate::from_ymd_opt(2026, 9, 26).unwrap(),
            "コンビニ",
            980,
            0,
            ids[5],
            "食費",
        ),
    ]
}

#[async_trait]
impl BookService for MockServices {
    async fn list_books(&self) -> domain::Result<Vec<BookEntry>> {
        Ok(vec![])
    }

    async fn find_book(
        &self,
        cabinet_name: domain::CabinetName,
        book_slug: domain::BookSlug,
    ) -> domain::Result<BookEntry> {
        Ok(BookEntry {
            id: domain::BookId::new(Uuid::from_u128(100)),
            slug: book_slug.as_str().to_string(),
            name: "デモ帳簿".to_string(),
            role: BookRole::Writable,
            cabinet_id: domain::CabinetId::new(Uuid::from_u128(1)),
            cabinet_name: cabinet_name.as_str().to_string(),
            cabinet_type: domain::CabinetType::Owned,
            unit_id: jpy_unit_id(),
            unit_code: "JPY".to_string(),
            unit_name: "円".to_string(),
            unit_symbol: "¥".to_string(),
            revision: 1,
            sync_status: None,
        })
    }

    async fn list_accounts(&self, _book: &domain::BookId) -> usecase::Result<Vec<AccountEntry>> {
        Ok(fixture_accounts())
    }
}

#[async_trait]
impl AccountService for MockServices {
    async fn find_accounts(
        &self,
        accounts: &[domain::AccountId],
        _filter: &Filter,
    ) -> usecase::Result<Vec<AccountReport>> {
        Ok(fixture_reports()
            .into_iter()
            .filter(|r| accounts.contains(&r.id))
            .collect())
    }
}

#[async_trait]
impl TransactionService for MockServices {
    async fn find_transactions(
        &self,
        account: &[domain::AccountId],
        _filter: &Filter,
    ) -> usecase::Result<Vec<TransactionReport>> {
        Ok(fixture_transactions()
            .into_iter()
            .filter(|t| account.is_empty() || account.contains(&t.account_id))
            .collect())
    }
}
