use chrono::NaiveDate;

use crate::domain;
use crate::usecase::{
    AccountEntry, AccountReport, AccountService, BookEntry, BookService, Filter, Result,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PNLType {
    Expense,
    Income,
}

pub struct PNLArgs {
    pub cabinet: domain::CabinetName,
    pub book: domain::BookSlug,
    pub type_: PNLType,
    pub year: i32,
    pub month: u32,
}

pub struct PageArgs {
    pub book: BookEntry,
    pub accounts: HashMap<domain::CategoryId, Vec<AccountEntry>>,
    pub summary: Vec<AccountReport>,
    pub sum_income: domain::RawInt,
    pub sum_expense: domain::RawInt,
    pub year: i32,
    pub month: u32,
    pub next_year: i32,
    pub next_month: u32,
    pub prev_year: i32,
    pub prev_month: u32,
}

pub async fn pnl(service: impl AccountService + BookService, args: PNLArgs) -> Result<PageArgs> {
    let book = service
        .find_book(args.cabinet.into(), args.book.into())
        .await?;

    let accounts = service.list_accounts(&book.id).await?;
    let mut expense = Vec::<domain::AccountId>::new();
    let mut income = Vec::<domain::AccountId>::new();
    let mut account_map = HashMap::<domain::CategoryId, Vec<AccountEntry>>::new();
    for ac in accounts.into_iter() {
        match ac.major_category {
            domain::MajorCategory::Expense => {
                expense.push(ac.id);
            }
            domain::MajorCategory::Income => {
                income.push(ac.id);
            }
            _others => (),
        }
        match account_map.get_mut(&ac.category_id) {
            None => {
                if ac.num_entries > 0 {
                    account_map.insert(ac.category_id.clone(), vec![ac]);
                }
            }
            Some(at) => {
                if ac.num_entries > 0 {
                    at.push(ac);
                }
            }
        }
    }

    let filter = Filter::new(
        NaiveDate::from_ymd_opt(args.year, args.month, 1).unwrap(),
        NaiveDate::from_ymd_opt(args.year, args.month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap(),
    );

    let expense = service.find_accounts(&expense, &filter).await?;
    let mut sum_expense = 0;
    for a in expense.iter() {
        if a.unit_id == book.unit_id {
            sum_expense += a.debit as domain::RawInt - a.credit as domain::RawInt;
        }
    }
    let income = service.find_accounts(&income, &filter).await?;
    let mut sum_income = 0;
    for a in income.iter() {
        if a.unit_id == book.unit_id {
            sum_income += a.debit as domain::RawInt - a.credit as domain::RawInt;
        }
    }

    let summary;
    match args.type_ {
        PNLType::Expense => {
            summary = expense;
        }
        PNLType::Income => {
            summary = income;
        }
    }

    Ok(PageArgs {
        book,
        accounts: account_map,
        summary,
        sum_income,
        sum_expense,
        year: args.year,
        month: args.month,
        next_year: if args.month == 12 {
            args.year + 1
        } else {
            args.year
        },
        next_month: if args.month == 12 { 1 } else { args.month + 1 },
        prev_year: if args.month == 1 {
            args.year - 1
        } else {
            args.year
        },
        prev_month: if args.month == 1 { 12 } else { args.month - 1 },
    })
}
