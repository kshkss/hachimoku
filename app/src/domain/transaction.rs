use crate::domain::{TransactionId, EntryId, BookId, AccountId, UnitId, Amount, Result, DomainError, Relation, MajorCategory};
use crate::domain::account::Account;
use crate::domain::unit::Unit;
use crate::domain::book::Book;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::{Uuid, Timestamp, NoContext};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Journal {
    transactions: Vec<Transaction>,
    book: BookId,
}

#[derive(Debug)]
pub struct TransactionParam {
    pub date: DateTime<Utc>,
    pub note: String,
    pub entries: Vec<EntryId>,
}

#[derive(Debug)]
pub struct Transaction {
    id: TransactionId,
    date: DateTime<Utc>,
    note: String,
    entries: Vec<Entry>,
    book: BookId,
    is_deleted: bool,
}

#[derive(Debug)]
pub struct Entry {
    id: EntryId,
    date: Option<DateTime<Utc>>,
    note: String,
    amount: Amount,
    unit: Relation<UnitId, Unit>,
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

impl Entry {
    pub fn has_id(&self, id: EntryId) -> bool {
        self.id == id
    }

    pub fn reconstruct(
        id: EntryId,
        date: Option<DateTime<Utc>>,
        note: String,
        amount: Amount,
        unit: Relation<UnitId, Unit>,
        account: Relation<AccountId, Account>,
        is_protected: bool,
        provider_id: Option<String>,
    ) -> Self {
        Self { id, date, note, amount, unit, account, is_protected, provider_id }
    }

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

    pub fn mark_protected(&mut self, date: DateTime<Utc>, note: String, provider_id: String) -> Result<()> {
        if self.is_protected {
            return Err(DomainError::InvalidEntry);
        }
        if note.is_empty() || provider_id.is_empty() {
            return Err(DomainError::InvalidEntry);
        }
        self.date = Some(date);
        self.note = note;
        self.provider_id = Some(provider_id);
        self.is_protected = true;
        Ok(())
    }
}

impl Transaction {
    pub fn has_id(&self, id: TransactionId) -> bool {
        self.id == id
    }

    pub fn reconstruct(
        id: TransactionId,
        date: DateTime<Utc>,
        note: String,
        entries: Vec<Entry>,
        book: BookId,
        is_deleted: bool,
    ) -> Self {
        Self { id, date, note, entries, book, is_deleted }
    }

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
                return Err(DomainError::InvalidTransactionOperation);
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
    pub fn reconstruct(transactions: Vec<Transaction>, book: BookId) -> Self {
        Self { transactions, book }
    }

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

    pub fn create_transaction(&mut self, date: DateTime<Utc>, note: String, entries: Vec<Entry>) -> Result<TransactionId> {
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

    pub fn update(&mut self, txid: TransactionId, date: DateTime<Utc>, note: String) -> Result<TransactionId> {
        let idx = self.transactions.iter().position(|t| t.has_id(txid)).ok_or(DomainError::InvalidTransactionOperation)?;
        
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

    pub fn merge(&mut self, date: DateTime<Utc>, note: String, transaction_ids: Vec<TransactionId>) -> Result<TransactionId> {
        if transaction_ids.len() < 2 || note.is_empty() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        let mut indices = Vec::new();
        for tid in &transaction_ids {
            let idx = self.transactions.iter().position(|t| t.has_id(*tid)).ok_or(DomainError::InvalidTransactionOperation)?;
            if self.transactions[idx].is_deleted() {
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

    pub fn divide(&mut self, source: TransactionId, params: Vec<TransactionParam>) -> Result<Vec<TransactionId>> {
        if params.len() < 2 {
            return Err(DomainError::InvalidTransactionOperation);
        }
        let idx = self.transactions.iter().position(|t| t.has_id(source)).ok_or(DomainError::InvalidTransactionOperation)?;
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
                let e_idx = source_tx.iter().position(|e| e.has_id(eid)).ok_or(DomainError::InvalidTransactionOperation)?;
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

    pub fn update_entries(&mut self, source: TransactionId, reuse: Vec<EntryId>, created: Vec<Entry>) -> Result<TransactionId> {
        let idx = self.transactions.iter().position(|t| t.has_id(source)).ok_or(DomainError::InvalidTransactionOperation)?;
        
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
            let e_idx = source_tx.iter().position(|e| e.has_id(eid)).ok_or(DomainError::InvalidTransactionOperation)?;
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
    pub fn create_entry(&self, date: DateTime<Utc>, note: String, amount: Amount) -> Result<Entry> {
        let (major, account_id) = {
            let cat = match self.category() {
                Relation::Loaded(c) => c,
                Relation::Unloaded(_) => return Err(DomainError::System),
            };
            use crate::domain::HasId;
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
    pub fn create_journal(&self) -> Journal {
        use crate::domain::HasId;
        Journal {
            transactions: Vec::with_capacity(2),
            book: self.id(),
        }
    }
}

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn get_transactions(&self, transaction_ids: Vec<TransactionId>) -> Result<Journal>;
    async fn update_transactions(&self, journal: &Journal) -> Result<()>;
}
