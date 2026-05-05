use crate::domain::{TransactionId, EntryId, BookId, AccountId, UnitId, Amount, Result, DomainError, Relation, MajorCategory};
use crate::domain::account::Account;
use crate::domain::unit::Unit;
use crate::domain::book::Book;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::{Uuid, Timestamp, NoContext};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Journal {
    pub transactions: Vec<Transaction>,
    pub book: BookId,
}

#[derive(Debug, Clone)]
pub struct TransactionParam {
    pub date: DateTime<Utc>,
    pub note: String,
    pub entries: Vec<EntryId>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub date: DateTime<Utc>,
    pub note: String,
    pub entries: Vec<Entry>,
    pub book: BookId,
    pub is_deleted: bool,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: EntryId,
    pub date: Option<DateTime<Utc>>,
    pub note: String,
    pub amount: Amount,
    pub unit: Relation<UnitId, Unit>,
    pub account: Relation<AccountId, Account>,
    pub is_protected: bool,
    pub provider_id: Option<String>,
}

fn generate_id_from_date(date: DateTime<Utc>) -> Uuid {
    let ts = Timestamp::from_unix(
        NoContext,
        date.timestamp() as u64,
        date.timestamp_subsec_nanos(),
    );
    Uuid::new_v7(ts)
}

impl Transaction {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_nil() || self.note.is_empty() || self.book.0.is_nil() {
            return Err(DomainError::System);
        }
        for e in &self.entries {
            e.validate()?;
        }
        Ok(())
    }
}

impl Entry {
    pub fn validate(&self) -> Result<()> {
        if self.id.0.is_nil() {
            return Err(DomainError::System);
        }
        self.amount.validate()?;
        
        // Unit validation
        match &self.unit {
            Relation::Unloaded(id) => {
                if *id != self.amount.unit {
                    return Err(DomainError::System);
                }
            }
            Relation::Loaded(u) => {
                u.validate()?;
                if u.id != self.amount.unit {
                    return Err(DomainError::System);
                }
            }
        }

        // Account validation
        match &self.account {
            Relation::Unloaded(id) => {
                if id.0.is_nil() {
                    return Err(DomainError::System);
                }
            }
            Relation::Loaded(a) => {
                a.validate()?;
            }
        }

        if self.is_protected && (self.date.is_none() || self.note.is_empty() || self.provider_id.is_none() || self.provider_id.as_ref().unwrap().is_empty()) {
            return Err(DomainError::System);
        }
        Ok(())
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

fn validate_date(date: DateTime<Utc>, entries: &[Entry]) -> Result<()> {
    for e in entries {
        if let Some(e_date) = e.date {
            if e_date != date {
                return Err(DomainError::InvalidTransactionOperation);
            }
        }
    }
    Ok(())
}

fn validate_balance(entries: &[Entry]) -> Result<()> {
    if entries.len() < 2 {
        return Err(DomainError::InvalidTransactionOperation);
    }
    let mut sum: HashMap<UnitId, Amount> = HashMap::new();
    for e in entries {
        if e.amount.is_zero() {
            return Err(DomainError::InvalidTransactionOperation);
        }
        match sum.get_mut(&e.amount.unit) {
            Some(existing) => {
                *existing = (*existing + e.amount)?;
            }
            None => {
                sum.insert(e.amount.unit, e.amount);
            }
        }
    }
    for amount in sum.values() {
        if !amount.is_balance() {
            return Err(DomainError::InvalidTransactionOperation);
        }
    }
    Ok(())
}

impl Journal {
    pub fn validate(&self) -> Result<()> {
        let mut used_entry_ids = HashMap::new();
        for tx in &self.transactions {
            tx.validate()?;
            if tx.book != self.book {
                return Err(DomainError::InvalidTransactionOperation);
            }
            if tx.is_deleted {
                for e in &tx.entries {
                    e.validate()?;
                    if e.is_protected {
                        return Err(DomainError::InvalidTransactionOperation);
                    }
                    if used_entry_ids.insert(e.id, true).is_some() {
                        return Err(DomainError::InvalidTransactionOperation);
                    }
                }
            } else {
                for e in &tx.entries {
                    e.validate()?;
                    if used_entry_ids.insert(e.id, true).is_some() {
                        return Err(DomainError::InvalidTransactionOperation);
                    }
                }
                validate_date(tx.date, &tx.entries)?;
                validate_balance(&tx.entries)?;
            }
        }
        Ok(())
    }

    pub fn create_transaction(&mut self, date: DateTime<Utc>, note: String, entries: Vec<Entry>) -> Result<TransactionId> {
        if note.is_empty() {
            return Err(DomainError::InvalidTransactionOperation);
        }
        for e in &entries {
            e.validate()?;
        }
        validate_date(date, &entries)?;
        validate_balance(&entries)?;

        let id = TransactionId(generate_id_from_date(date));
        let tx = Transaction {
            id,
            date,
            note,
            entries,
            book: self.book,
            is_deleted: false,
        };
        self.transactions.push(tx);
        Ok(id)
    }

    pub fn update(&mut self, txid: TransactionId, date: DateTime<Utc>, note: String) -> Result<TransactionId> {
        let idx = self.transactions.iter().position(|t| t.id == txid).ok_or(DomainError::InvalidTransactionOperation)?;
        
        {
            let tx = &self.transactions[idx];
            if tx.is_deleted || note.is_empty() {
                return Err(DomainError::InvalidTransactionOperation);
            }
            validate_date(date, &tx.entries)?;
        }

        let entries = self.transactions[idx].entries.clone();
        let book = self.transactions[idx].book;

        let id = TransactionId(generate_id_from_date(date));
        let new_tx = Transaction {
            id,
            date,
            note,
            entries,
            book,
            is_deleted: false,
        };
        
        self.transactions[idx].is_deleted = true;
        self.transactions[idx].entries = Vec::new();
        self.transactions.push(new_tx);
        
        Ok(id)
    }

    pub fn merge(&mut self, date: DateTime<Utc>, note: String, transaction_ids: Vec<TransactionId>) -> Result<TransactionId> {
        if transaction_ids.len() < 2 || note.is_empty() {
            return Err(DomainError::InvalidTransactionOperation);
        }

        let mut entries = Vec::new();
        let mut indices = Vec::new();

        for tid in &transaction_ids {
            let idx = self.transactions.iter().position(|t| t.id == *tid).ok_or(DomainError::InvalidTransactionOperation)?;
            if self.transactions[idx].is_deleted {
                return Err(DomainError::InvalidTransactionOperation);
            }
            indices.push(idx);
        }

        for &idx in &indices {
            entries.extend(self.transactions[idx].entries.clone());
        }

        validate_date(date, &entries)?;
        validate_balance(&entries)?;

        for &idx in &indices {
            self.transactions[idx].is_deleted = true;
            self.transactions[idx].entries = Vec::new();
        }

        let id = TransactionId(generate_id_from_date(date));
        let tx = Transaction {
            id,
            date,
            note,
            entries,
            book: self.book,
            is_deleted: false,
        };
        self.transactions.push(tx);
        Ok(id)
    }

    pub fn divide(&mut self, source: TransactionId, params: Vec<TransactionParam>) -> Result<Vec<TransactionId>> {
        if params.len() < 2 {
            return Err(DomainError::InvalidTransactionOperation);
        }
        let idx = self.transactions.iter().position(|t| t.id == source).ok_or(DomainError::InvalidTransactionOperation)?;
        if self.transactions[idx].is_deleted {
            return Err(DomainError::InvalidTransactionOperation);
        }

        let mut dest_entries = Vec::new();
        let mut consumed_entries = HashMap::new();

        {
            let tx = &self.transactions[idx];
            for p in &params {
                if p.note.is_empty() {
                    return Err(DomainError::InvalidTransactionOperation);
                }
                let mut entries = Vec::new();
                for &eid in &p.entries {
                    if consumed_entries.insert(eid, true).is_some() {
                        return Err(DomainError::InvalidTransactionOperation);
                    }
                    let e_idx = tx.entries.iter().position(|e| e.id == eid).ok_or(DomainError::InvalidTransactionOperation)?;
                    entries.push(tx.entries[e_idx].clone());
                }
                validate_date(p.date, &entries)?;
                validate_balance(&entries)?;
                dest_entries.push(entries);
            }

            for e in &tx.entries {
                if !consumed_entries.contains_key(&e.id) {
                    return Err(DomainError::InvalidTransactionOperation);
                }
            }
        }

        let mut res = Vec::new();
        for (i, p) in params.iter().enumerate() {
            let id = TransactionId(generate_id_from_date(p.date));
            let new_tx = Transaction {
                id,
                date: p.date,
                note: p.note.clone(),
                entries: dest_entries[i].clone(),
                book: self.book,
                is_deleted: false,
            };
            self.transactions.push(new_tx);
            res.push(id);
        }

        self.transactions[idx].is_deleted = true;
        self.transactions[idx].entries = Vec::new();

        Ok(res)
    }

    pub fn update_entries(&mut self, source: TransactionId, reuse: Vec<EntryId>, created: Vec<Entry>) -> Result<TransactionId> {
        let idx = self.transactions.iter().position(|t| t.id == source).ok_or(DomainError::InvalidTransactionOperation)?;
        
        let (date, note, new_entries) = {
            let tx = &self.transactions[idx];
            if tx.is_deleted {
                return Err(DomainError::InvalidTransactionOperation);
            }

            let mut new_entries = Vec::new();
            for e in &tx.entries {
                if reuse.contains(&e.id) {
                    new_entries.push(e.clone());
                } else {
                    if e.is_protected {
                        return Err(DomainError::InvalidTransactionOperation);
                    }
                }
            }

            for eid in &reuse {
                if !tx.entries.iter().any(|e| e.id == *eid) {
                    return Err(DomainError::InvalidTransactionOperation);
                }
            }

            for e in &created {
                e.validate()?;
            }
            validate_date(tx.date, &created)?;
            new_entries.extend(created);

            validate_balance(&new_entries)?;
            (tx.date, tx.note.clone(), new_entries)
        };

        let id = TransactionId(generate_id_from_date(date));
        
        // Final modification
        let mut delete_entries = Vec::new();
        for e in self.transactions[idx].entries.drain(..) {
            if !reuse.contains(&e.id) {
                delete_entries.push(e);
            }
        }
        
        self.transactions[idx].entries = delete_entries;
        self.transactions[idx].is_deleted = true;

        let dest = Transaction {
            id,
            date,
            note,
            entries: new_entries,
            book: self.book,
            is_deleted: false,
        };
        self.transactions.push(dest);

        Ok(id)
    }
}

impl Account {
    pub fn create_entry(&self, date: DateTime<Utc>, note: String, amount: Amount) -> Result<Entry> {
        let cat = match &self.category {
            Relation::Loaded(c) => c,
            Relation::Unloaded(_) => return Err(DomainError::System),
        };

        if amount.is_zero() {
            return Err(DomainError::InvalidEntry);
        }
        amount.validate()?;

        if amount.unit != self.unit && cat.major != MajorCategory::Equity {
            return Err(DomainError::InvalidEntry);
        }

        Ok(Entry {
            id: EntryId(generate_id_from_date(date)),
            date: None,
            note,
            amount,
            unit: Relation::Unloaded(amount.unit),
            account: Relation::Unloaded(self.id),
            is_protected: false,
            provider_id: None,
        })
    }
}

impl Book {
    pub fn create_journal(&self) -> Journal {
        Journal {
            transactions: Vec::with_capacity(2),
            book: self.id,
        }
    }
}

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn get_transactions(&self, transaction_ids: Vec<TransactionId>) -> Result<Journal>;
    async fn update_transactions(&self, journal: &Journal) -> Result<()>;
}
