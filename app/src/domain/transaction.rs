use crate::domain::account::Account;
use crate::domain::book::Book;
use crate::domain::unit::Unit;
use crate::domain::{
    AccountId, Amount, BookId, DomainError, Entity, EntryId, MajorCategory, Relation, Result,
    TransactionId, UnitId,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::{NoContext, Timestamp, Uuid};

/// Journal は帳簿内の取引（Transaction）の集合を管理する集約ルートです。
#[derive(Debug)]
pub struct Journal {
    transactions: Vec<Transaction>,
    book: BookId,
}

/// TransactionParam は取引を分割する際などに使用されるパラメータです。
#[derive(Debug)]
pub struct TransactionParam {
    pub date: DateTime<Utc>,
    pub note: String,
    pub entries: Vec<EntryId>,
}

/// Transaction は一回分の会計取引を表すドメインエンティティです。
#[derive(Debug)]
pub struct Transaction {
    id: TransactionId,
    date: DateTime<Utc>,
    note: String,
    entries: Vec<Entry>,
    book: BookId,
    is_deleted: bool,
}

/// Entry は取引を構成する各勘定科目への記帳（エントリー）を表すドメインエンティティです。
#[derive(Debug)]
pub struct Entry {
    id: EntryId,
    /// Date は Entry が保護されるときにセットされます。保護されていない場合は None です。
    date: Option<DateTime<Utc>>,
    note: String,
    amount: Amount,
    /// Unit は記帳に使用される単位です。
    unit: Relation<UnitId, Unit>,
    /// Account は記帳対象の勘定科目です。
    account: Relation<AccountId, Account>,
    is_protected: bool,
    provider_id: Option<String>,
}

fn generate_id_from_date(date: DateTime<Utc>) -> Uuid {
    let ts = Timestamp::from_unix(
        NoContext,
        date.timestamp() as u64,
        date.timestamp_subsec_nanos(),
    );
    Uuid::new_v7(ts)
}

impl Entity for Entry {
    type Id = EntryId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Entry {
    /// `Account.create_entry()` で作成した Entry を保護するメソッドです。
    ///
    /// # 正常終了時
    /// 保護済み状態となった自身の所有権（`Ok(Self)`）を返します。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidEntry` を返します。
    /// - すでに保護されている場合。
    /// - `note` が空の場合。
    /// - `provider_id` が空の場合。
    ///
    /// # 注意
    /// Journal 内の Entry に対して直接呼び出してはいけません。
    pub fn mark_protected(
        self,
        date: DateTime<Utc>,
        note: String,
        provider_id: String,
    ) -> Result<Self> {
        if self.is_protected {
            return Err(DomainError::InvalidEntry);
        }
        if note.is_empty() || provider_id.is_empty() {
            Err(DomainError::InvalidEntry)
        } else {
            Ok(Self {
                date: Some(date),
                note,
                is_protected: true,
                provider_id: Some(provider_id),
                ..self
            })
        }
    }

    /// インフラ層などで既存のデータから Entry を復元します。
    pub fn reconstruct(
        id: EntryId,
        date: Option<DateTime<Utc>>,
        note: String,
        amount: Amount,
        unit: Relation<UnitId, Unit>,
        account: Relation<AccountId, Account>,
        is_protected: bool,
        provider_id: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            date,
            note,
            amount,
            unit,
            account,
            is_protected,
            provider_id,
        })
    }

    /// 指定された日付が、エントリーの日付（セットされている場合）と一致するかどうかを返します。
    pub fn is_date_match(&self, date: DateTime<Utc>) -> bool {
        match self.date {
            Some(d) => d == date,
            None => true,
        }
    }

    pub fn is_protected(&self) -> bool {
        self.is_protected
    }

    pub fn amount(&self) -> Amount {
        self.amount
    }

    pub fn unit(&self) -> UnitId {
        self.amount.unit()
    }
}

impl Entity for Transaction {
    type Id = TransactionId;
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Transaction {
    /// インフラ層などで既存のデータから Transaction を復元します。
    pub fn reconstruct(
        id: TransactionId,
        date: DateTime<Utc>,
        note: String,
        entries: Vec<Entry>,
        book: BookId,
        is_deleted: bool,
    ) -> Result<Self> {
        Ok(Self {
            id,
            date,
            note,
            entries,
            book,
            is_deleted,
        })
    }

    /// 取引の内容が妥当性を検証します。
    ///
    /// # 正常終了時
    /// 内容が妥当であることを示す `Ok(())` を返します。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidTransactionOperation` を返します。
    /// - エントリーが2つ未満の場合。
    /// - エントリーの日付と取引の日付が一貫していない場合。
    /// - 借方と貸方の合計（貸借）が一致していない場合。
    ///
    /// 以下の場合は `DomainError::InvalidEntry` を返します。
    /// - エントリーの金額がゼロの場合。
    pub fn validate(&self) -> Result<()> {
        if self.entries.len() < 2 {
            return Err(DomainError::InvalidTransactionOperation);
        }
        let mut sum: HashMap<UnitId, Amount> = HashMap::new();
        for e in &self.entries {
            if !e.is_date_match(self.date) {
                return Err(DomainError::InvalidTransactionOperation);
            }
            let amount = e.amount();
            if amount.is_zero() {
                return Err(DomainError::InvalidEntry);
            }
            match sum.get_mut(&amount.unit()) {
                Some(existing) => {
                    *existing = *existing + amount;
                }
                None => {
                    sum.insert(amount.unit(), amount);
                }
            }
        }
        for amount in sum.values() {
            if !amount.is_balanced() {
                return Err(DomainError::InvalidTransactionOperation);
            }
        }
        Ok(())
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// 取引内に保護されたエントリーが含まれているかどうかを返します。
    pub fn has_protected_entries(&self) -> bool {
        self.entries.iter().any(|e| e.is_protected())
    }

    pub fn push_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn pop_entry(&mut self) -> Option<Entry> {
        self.entries.pop()
    }

    pub fn remove(&mut self, index: usize) -> Entry {
        self.entries.remove(index)
    }

    /// 他の取引からすべてのエントリーをこの取引に移動させます。
    pub fn append(&mut self, other: &mut Transaction) {
        self.entries.append(&mut other.entries);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Entry> {
        self.entries.iter()
    }
}

impl Journal {
    /// インフラ層などで既存のデータから Journal を復元します。
    /// 復元時に、各取引の所属帳簿 ID が一致しているか、および各取引が独立して妥当（`Transaction::validate`）かを検証します。
    pub fn reconstruct(transactions: Vec<Transaction>, book: BookId) -> Result<Self> {
        for tx in &transactions {
            if tx.book != book {
                return Err(DomainError::System);
            }
            tx.validate()?;
        }
        Ok(Self { transactions, book })
    }

    /// `UpdateTransactions` を呼び出す前に、不変条件を満たすか検査する関数です。
    ///
    /// # 正常終了時
    /// 不変条件が満たされていることを示す `Ok(())` を返します。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidTransactionOperation` を返します。
    /// - 取引の所属帳簿 ID が Journal の帳簿 ID と一致しない場合。
    /// - 削除される取引（`is_deleted` が true）に保護されたエントリーが含まれている場合。
    /// - 削除されない取引において：
    ///   - 取引にエントリーが含まれていない、または1つしか含まれていない場合。
    ///   - 貸借の合計が一致していない場合。
    ///   - 取引とそれに含まれるエントリーの日付が一貫していない場合。
    ///   - 金額がゼロのエントリーが含まれている場合。
    pub fn validate(&self) -> Result<()> {
        for tx in &self.transactions {
            if tx.book != self.book {
                return Err(DomainError::InvalidTransactionOperation);
            }
            if tx.is_deleted() {
                if tx.has_protected_entries() {
                    return Err(DomainError::InvalidTransactionOperation);
                }
            } else {
                tx.validate()?;
            }
        }
        Ok(())
    }

    /// 新しい取引を作成して Journal に追加します。
    ///
    /// # 正常終了時
    /// 生成された新しい取引の ID（`Ok(TransactionId)`）を返します。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidTransactionOperation` を返します。
    /// - `note` が空の場合。
    /// - `date` がゼロ（未指定）の場合。
    /// - 貸借が一致しない、または日付に整合性がない場合（`Transaction::validate` に準ずる）。
    pub fn create_transaction(
        &mut self,
        date: DateTime<Utc>,
        note: String,
        entries: Vec<Entry>,
    ) -> Result<TransactionId> {
        if note.is_empty() {
            return Err(DomainError::InvalidTransactionOperation);
        }
        let id = TransactionId::new(generate_id_from_date(date));
        let tx = Transaction {
            id,
            date,
            note,
            entries,
            book: self.book,
            is_deleted: false,
        };
        tx.validate()?;
        self.transactions.push(tx);
        Ok(id)
    }

    /// 取引の日付と説明を更新します。
    ///
    /// # 正常終了時
    /// 生成された新しい取引の ID（`Ok(TransactionId)`）を返します。
    ///
    /// # 挙動
    /// 元の取引は削除済み（`is_deleted` = true）となり、エントリーはすべて新しい取引へ移動します。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidTransactionOperation` を返します。
    /// - 取引 ID が見つからない場合。
    /// - 指定された取引がすでに削除済みの場合。
    /// - `note` が空、または `date` がゼロの場合。
    /// - 更新後の日付が既存エントリーの日付と矛盾する場合。
    pub fn update(
        &mut self,
        txid: TransactionId,
        date: DateTime<Utc>,
        note: String,
    ) -> Result<TransactionId> {
        let idx = self
            .transactions
            .iter()
            .position(|t| t.has_id(txid))
            .ok_or(DomainError::InvalidTransactionOperation)?;

        if self.transactions[idx].is_deleted() || note.is_empty() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        let mut old_tx = &mut self.transactions[idx];
        let id = TransactionId::new(generate_id_from_date(date));
        let mut new_tx = Transaction {
            id,
            date,
            note,
            entries: Vec::new(),
            book: old_tx.book,
            is_deleted: false,
        };
        new_tx.append(&mut old_tx);
        new_tx.validate()?;

        old_tx.is_deleted = true;
        self.transactions.push(new_tx);

        Ok(id)
    }

    /// 複数の取引を一つにマージします。
    ///
    /// # 正常終了時
    /// マージによって生成された新しい取引の ID（`Ok(TransactionId)`）を返します。
    ///
    /// # 挙動
    /// マージ後は元のすべての取引が削除済み（`is_deleted` = true）となり、全エントリーが新しい取引へ移動します。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidTransactionOperation` を返します。
    /// - 指定された取引が2つ未満の場合。
    /// - `note` が空、または `date` がゼロの場合。
    /// - 取引 ID が見つからない、または既に削除済みの場合。
    /// - 同じ取引 ID が重複して指定されている場合。
    /// - マージ後の日付と全エントリーの日付が一貫していない場合。
    /// - 統合後のエントリーの貸借が一致しない場合。
    pub fn merge(
        &mut self,
        date: DateTime<Utc>,
        note: String,
        transaction_ids: Vec<TransactionId>,
    ) -> Result<TransactionId> {
        if transaction_ids.len() < 2 || note.is_empty() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        let mut indices = Vec::new();
        for tid in &transaction_ids {
            let idx = self
                .transactions
                .iter()
                .position(|t| t.has_id(*tid))
                .ok_or(DomainError::InvalidTransactionOperation)?;
            if self.transactions[idx].is_deleted {
                return Err(DomainError::InvalidTransactionOperation);
            }
            if indices.contains(&idx) {
                return Err(DomainError::InvalidTransactionOperation);
            }
            indices.push(idx);
        }

        let id = TransactionId::new(generate_id_from_date(date));
        let mut new_tx = Transaction {
            id,
            date,
            note,
            entries: Vec::new(),
            book: self.book,
            is_deleted: false,
        };

        for &idx in &indices {
            let mut old_tx = &mut self.transactions[idx];
            new_tx.append(&mut old_tx);
            old_tx.is_deleted = true;
        }

        new_tx.validate()?;
        self.transactions.push(new_tx);
        Ok(id)
    }

    /// 取引を複数の取引に分割します。
    ///
    /// # 正常終了時
    /// 分割によって生成された新しい取引の ID リスト（`Ok(Vec<TransactionId>)`）を返します。
    ///
    /// # 挙動
    /// 分割後、元の取引は削除済み（`is_deleted` = true）となります。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidTransactionOperation` を返します。
    /// - 分割先パラメータ（`params`）が2つ未満の場合。
    /// - `note` が空、または `date` がゼロのパラメータが含まれる場合。
    /// - ソースとなる取引が見つからない、または既に削除済みの場合。
    /// - 分割先全体でエントリーが重複して指定されている場合。
    /// - 分割後の各取引において、日付矛盾または貸借不一致が発生する場合。
    /// - ソース取引のすべてのエントリーが、分割先のいずれにも含まれていない（未消化）場合。
    pub fn divide(
        &mut self,
        source: TransactionId,
        params: Vec<TransactionParam>,
    ) -> Result<Vec<TransactionId>> {
        if params.len() < 2 {
            return Err(DomainError::InvalidTransactionOperation);
        }
        let idx = self
            .transactions
            .iter()
            .position(|t| t.has_id(source))
            .ok_or(DomainError::InvalidTransactionOperation)?;
        if self.transactions[idx].is_deleted() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        let source_tx = &mut self.transactions[idx];
        let mut new_transactions = Vec::new();
        let mut result_ids = Vec::new();

        for p in params {
            if p.note.is_empty() {
                return Err(DomainError::InvalidTransactionOperation);
            }
            let id = TransactionId::new(generate_id_from_date(p.date));
            let mut tx = Transaction {
                id,
                date: p.date,
                note: p.note,
                entries: Vec::new(),
                book: self.book,
                is_deleted: false,
            };
            for eid in p.entries {
                let e_idx = source_tx
                    .iter()
                    .position(|e| e.has_id(eid))
                    .ok_or(DomainError::InvalidTransactionOperation)?;
                let entry = source_tx.remove(e_idx);
                tx.push_entry(entry);
            }
            tx.validate()?;
            new_transactions.push(tx);
            result_ids.push(id);
        }

        if !source_tx.is_empty() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        source_tx.is_deleted = true;
        self.transactions.append(&mut new_transactions);

        Ok(result_ids)
    }

    /// 取引に含まれるエントリーを更新（追加・削除・再利用）します。
    ///
    /// # 正常終了時
    /// 更新された内容を持つ新しい取引の ID（`Ok(TransactionId)`）を返します。
    ///
    /// # 挙動
    /// 元の取引は削除済み（`is_deleted` = true）となります。
    /// `reuse` に含まれない（再利用されない）エントリーは、元の取引に残され、最終的にソース取引と共に削除されます。
    /// `created` （新規作成されたエントリー）は新しい取引に追加されます。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidTransactionOperation` を返します。
    /// - ソースとなる取引が見つからない、または既に削除済みの場合。
    /// - 再利用されない（削除予定の）エントリーの中に、保護された（`is_protected`）エントリーが含まれている場合。
    /// - 再利用指定されたエントリー ID がソース取引に含まれていない場合。
    /// - 新規作成されたエントリーの日付が、取引の日付と一貫していない場合。
    /// - 最終的な新しい取引において、貸借が一致しない、またはエントリー数が不足している場合。
    pub fn update_entries(
        &mut self,
        source: TransactionId,
        reuse: Vec<EntryId>,
        created: Vec<Entry>,
    ) -> Result<TransactionId> {
        let idx = self
            .transactions
            .iter()
            .position(|t| t.has_id(source))
            .ok_or(DomainError::InvalidTransactionOperation)?;

        if self.transactions[idx].is_deleted() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        let source_tx = &mut self.transactions[idx];
        let id = TransactionId::new(generate_id_from_date(source_tx.date));
        let mut new_tx = Transaction {
            id,
            date: source_tx.date,
            note: source_tx.note.clone(),
            entries: Vec::new(),
            book: self.book,
            is_deleted: false,
        };

        // Reuse entries
        for eid in reuse {
            let e_idx = source_tx
                .iter()
                .position(|e| e.has_id(eid))
                .ok_or(DomainError::InvalidTransactionOperation)?;
            new_tx.push_entry(source_tx.remove(e_idx));
        }

        // Add created entries
        for e in created {
            new_tx.push_entry(e);
        }

        // Check if remaining entries in source are protected
        if source_tx.has_protected_entries() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        new_tx.validate()?;
        source_tx.is_deleted = true;
        self.transactions.push(new_tx);

        Ok(id)
    }
}

impl Account {
    /// 新しい Entry を作成します。
    ///
    /// # 正常終了時
    /// 作成された `Entry` の所有権を返します。
    ///
    /// # 注意
    /// 取引に追加するには別途 `Journal::create_transaction` や `Journal::update_entries` を呼び出す必要があります。
    ///
    /// # エラー
    /// 以下の場合は `DomainError::InvalidEntry` を返します。
    /// - 金額が 0 の場合。
    /// - 金額の単位が科目の基本単位と異なり、かつ科目の大分類が「純資産」以外の場合。
    pub fn create_entry(&self, date: DateTime<Utc>, note: String, amount: Amount) -> Result<Entry> {
        let (major, account_id) = {
            let cat = match self.category() {
                Relation::Loaded(c) => c,
                Relation::Unloaded(_) => return Err(DomainError::System),
            };
            use crate::domain::Entity;
            (cat.major(), self.id())
        };

        if amount.is_zero() {
            return Err(DomainError::InvalidEntry);
        }

        if amount.unit() != self.unit() && major != MajorCategory::Equity {
            return Err(DomainError::InvalidEntry);
        }

        Ok(Entry {
            id: EntryId::new(generate_id_from_date(date)),
            date: None,
            note,
            amount,
            unit: Relation::Unloaded(amount.unit()),
            account: Relation::Unloaded(account_id),
            is_protected: false,
            provider_id: None,
        })
    }
}

impl Book {
    /// この帳簿に関連付けられた空の Journal を作成します。
    pub fn create_journal(&self) -> Journal {
        use crate::domain::Entity;
        Journal {
            transactions: Vec::with_capacity(2),
            book: self.id(),
        }
    }
}

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    /// 指定された複数の TransactionId に対応する取引を取得し、Journal として返します。
    async fn get_transactions(&self, transaction_ids: &[TransactionId]) -> Result<Journal>;
    /// 取引とエントリーを追加・削除・更新します。
    async fn update_transactions(&self, journal: &Journal) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BookSlug, Cabinet, CabinetName, Category, CategoryId, UnitId};
    use uuid::Uuid;

    fn setup_book() -> (Book, UnitId) {
        let cab = Cabinet::with_ownership(CabinetName::new("cab".to_string()).unwrap()).unwrap();
        let unit_id = UnitId::new(Uuid::now_v7());
        let book = cab
            .create_book(
                BookSlug::new("book".to_string()).unwrap(),
                "Book".to_string(),
                unit_id,
            )
            .unwrap();
        (book, unit_id)
    }

    fn setup_account(book: &Book, unit: UnitId) -> Account {
        let cat_id = CategoryId::new("cat".to_string());
        let cat = Category::reconstruct(
            cat_id.clone(),
            MajorCategory::Asset,
            "Minor".to_string(),
            true,
        )
        .unwrap();
        Account::reconstruct(
            AccountId::new(Uuid::now_v7()),
            "Acc".to_string(),
            Relation::Loaded(cat),
            unit,
            book.id(),
            true,
        )
        .unwrap()
    }

    #[test]
    fn test_entry_protection() {
        let (book, unit) = setup_book();
        let acc = setup_account(&book, unit);
        let date = Utc::now();
        let entry = acc
            .create_entry(
                date,
                "Note".to_string(),
                Amount::new_debit(100, unit).unwrap(),
            )
            .unwrap();

        // 正常系：初期状態では保護されていないことを確認
        assert!(!entry.is_protected());

        // 正常系：正しく保護状態に移行できることを確認
        let protected = entry
            .mark_protected(date, "Protected Note".to_string(), "provider-1".to_string())
            .unwrap();
        assert!(protected.is_protected());
        assert!(protected.is_date_match(date));

        // 異常系：二重に保護しようとするとエラーになることを確認
        assert!(
            protected
                .mark_protected(date, "fail".to_string(), "id".to_string())
                .is_err()
        );
    }

    #[test]
    fn test_transaction_balancing() {
        let (book, unit) = setup_book();
        let acc = setup_account(&book, unit);
        let date = Utc::now();

        let e1 = acc
            .create_entry(date, "D".to_string(), Amount::new_debit(100, unit).unwrap())
            .unwrap();
        let e2 = acc
            .create_entry(
                date,
                "C".to_string(),
                Amount::new_credit(100, unit).unwrap(),
            )
            .unwrap();

        // 正常系：貸借が一致し、日付が整合している取引が妥当（validate OK）であることを確認
        let mut tx = Transaction::reconstruct(
            TransactionId::new(Uuid::now_v7()),
            date,
            "Note".to_string(),
            vec![e1, e2],
            book.id(),
            false,
        )
        .unwrap();
        assert!(tx.validate().is_ok());

        // 異常系：貸借が一致しなくなった取引がバリデーションエラー（InvalidTransactionOperation）になることを確認
        let e3 = acc
            .create_entry(date, "D".to_string(), Amount::new_debit(10, unit).unwrap())
            .unwrap();
        tx.push_entry(e3);
        assert!(matches!(
            tx.validate(),
            Err(DomainError::InvalidTransactionOperation)
        ));
    }

    #[test]
    fn test_journal_update_ownership_move() {
        let (book, unit) = setup_book();
        let acc = setup_account(&book, unit);
        let date = Utc::now();
        let mut journal = book.create_journal();

        let e1 = acc
            .create_entry(date, "D".to_string(), Amount::new_debit(100, unit).unwrap())
            .unwrap();
        let e2 = acc
            .create_entry(
                date,
                "C".to_string(),
                Amount::new_credit(100, unit).unwrap(),
            )
            .unwrap();

        let tx_id = journal
            .create_transaction(date, "Old Note".to_string(), vec![e1, e2])
            .unwrap();

        // 正常系：取引の更新（update）により、エントリーの所有権が新しい取引へ移動することを確認
        let new_date = date + chrono::Duration::days(1);
        let new_tx_id = journal
            .update(tx_id, new_date, "New Note".to_string())
            .unwrap();

        assert_ne!(tx_id, new_tx_id);

        // 旧取引は削除済みかつ空（エントリーが移動済み）であるべき
        let old_tx = journal
            .transactions
            .iter()
            .find(|t| t.has_id(tx_id))
            .unwrap();
        assert!(old_tx.is_deleted());
        assert!(old_tx.is_empty());

        // 新取引は有効で、エントリーを保持しているべき
        let new_tx = journal
            .transactions
            .iter()
            .find(|t| t.has_id(new_tx_id))
            .unwrap();
        assert!(!new_tx.is_deleted());
        assert_eq!(new_tx.len(), 2);
        assert_eq!(new_tx.date, new_date);
    }

    #[test]
    fn test_journal_merge() {
        let (book, unit) = setup_book();
        let acc = setup_account(&book, unit);
        let date = Utc::now();
        let mut journal = book.create_journal();

        let tx1_id = journal
            .create_transaction(
                date,
                "T1".to_string(),
                vec![
                    acc.create_entry(date, "D".to_string(), Amount::new_debit(100, unit).unwrap())
                        .unwrap(),
                    acc.create_entry(
                        date,
                        "C".to_string(),
                        Amount::new_credit(100, unit).unwrap(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap();

        let tx2_id = journal
            .create_transaction(
                date,
                "T2".to_string(),
                vec![
                    acc.create_entry(date, "D".to_string(), Amount::new_debit(50, unit).unwrap())
                        .unwrap(),
                    acc.create_entry(date, "C".to_string(), Amount::new_credit(50, unit).unwrap())
                        .unwrap(),
                ],
            )
            .unwrap();

        // 正常系：2つの取引をマージし、エントリーが統合されることを確認
        let merged_id = journal
            .merge(date, "Merged".to_string(), vec![tx1_id, tx2_id])
            .unwrap();

        let merged_tx = journal
            .transactions
            .iter()
            .find(|t| t.has_id(merged_id))
            .unwrap();
        assert_eq!(merged_tx.len(), 4);

        // 元の取引はすべて削除済みとなっているべき
        assert!(
            journal
                .transactions
                .iter()
                .find(|t| t.has_id(tx1_id))
                .unwrap()
                .is_deleted()
        );
        assert!(
            journal
                .transactions
                .iter()
                .find(|t| t.has_id(tx2_id))
                .unwrap()
                .is_deleted()
        );
    }

    #[test]
    fn test_journal_divide() {
        let (book, unit) = setup_book();
        let acc = setup_account(&book, unit);
        let date = Utc::now();
        let mut journal = book.create_journal();

        let e1 = acc
            .create_entry(
                date,
                "D1".to_string(),
                Amount::new_debit(100, unit).unwrap(),
            )
            .unwrap();
        let e1_id = e1.id();
        let e2 = acc
            .create_entry(
                date,
                "C1".to_string(),
                Amount::new_credit(100, unit).unwrap(),
            )
            .unwrap();
        let e2_id = e2.id();
        let e3 = acc
            .create_entry(date, "D2".to_string(), Amount::new_debit(50, unit).unwrap())
            .unwrap();
        let e3_id = e3.id();
        let e4 = acc
            .create_entry(
                date,
                "C2".to_string(),
                Amount::new_credit(50, unit).unwrap(),
            )
            .unwrap();
        let e4_id = e4.id();

        let source_id = journal
            .create_transaction(date, "Source".to_string(), vec![e1, e2, e3, e4])
            .unwrap();

        // 正常系：1つの取引を2つに分割し、それぞれが独立して貸借一致することを確認
        let params = vec![
            TransactionParam {
                date,
                note: "Part 1".to_string(),
                entries: vec![e1_id, e2_id],
            },
            TransactionParam {
                date,
                note: "Part 2".to_string(),
                entries: vec![e3_id, e4_id],
            },
        ];

        let res = journal.divide(source_id, params).unwrap();
        assert_eq!(res.len(), 2);

        let source_tx = journal
            .transactions
            .iter()
            .find(|t| t.has_id(source_id))
            .unwrap();
        assert!(source_tx.is_deleted());
        assert!(source_tx.is_empty());

        assert_eq!(
            journal
                .transactions
                .iter()
                .find(|t| t.has_id(res[0]))
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            journal
                .transactions
                .iter()
                .find(|t| t.has_id(res[1]))
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn test_journal_update_entries() {
        let (book, unit) = setup_book();
        let acc = setup_account(&book, unit);
        let date = Utc::now();
        let mut journal = book.create_journal();

        let e1 = acc
            .create_entry(
                date,
                "Reuse".to_string(),
                Amount::new_debit(100, unit).unwrap(),
            )
            .unwrap();
        let e1_id = e1.id();
        let e2 = acc
            .create_entry(
                date,
                "Delete".to_string(),
                Amount::new_credit(100, unit).unwrap(),
            )
            .unwrap();

        let source_id = journal
            .create_transaction(date, "Source".to_string(), vec![e1, e2])
            .unwrap();

        // 正常系：特定のエントリーを保持しつつ、新しいエントリーを追加して取引を更新できることを確認
        let e3 = acc
            .create_entry(
                date,
                "New".to_string(),
                Amount::new_credit(100, unit).unwrap(),
            )
            .unwrap();

        let new_tx_id = journal
            .update_entries(source_id, vec![e1_id], vec![e3])
            .unwrap();

        let new_tx = journal
            .transactions
            .iter()
            .find(|t| t.has_id(new_tx_id))
            .unwrap();
        assert_eq!(new_tx.len(), 2);
        assert!(new_tx.iter().any(|e| e.has_id(e1_id)));

        assert!(
            journal
                .transactions
                .iter()
                .find(|t| t.has_id(source_id))
                .unwrap()
                .is_deleted()
        );
    }

    #[test]
    fn test_journal_validate_protected_in_deleted() {
        let (book, unit) = setup_book();
        let acc = setup_account(&book, unit);
        let date = Utc::now();

        let e1 = acc
            .create_entry(date, "D".to_string(), Amount::new_debit(100, unit).unwrap())
            .unwrap();
        let e2 = acc
            .create_entry(
                date,
                "C".to_string(),
                Amount::new_credit(100, unit).unwrap(),
            )
            .unwrap();
        let e1_protected = e1
            .mark_protected(date, "P".to_string(), "id".to_string())
            .unwrap();

        // 異常系：削除対象の取引に保護済みエントリーが含まれている場合、Journal::validate がエラーを返すことを確認
        let tx = Transaction::reconstruct(
            TransactionId::new(Uuid::now_v7()),
            date,
            "Note".to_string(),
            vec![e1_protected, e2],
            book.id(),
            true,
        )
        .unwrap();
        let journal = Journal::reconstruct(vec![tx], book.id()).unwrap();

        assert!(matches!(
            journal.validate(),
            Err(DomainError::InvalidTransactionOperation)
        ));
    }
}
