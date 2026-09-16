use chrono::{Datelike, NaiveDate};

use crate::domain;
use crate::usecase::{
    AccountEntry, BookEntry, BookService, Filter, Result, TransactionReport, TransactionService,
};
use std::collections::HashMap;

pub struct HistoryArgs {
    pub cabinet: domain::CabinetName,
    pub book: domain::BookSlug,
    pub account: Option<domain::AccountId>,
    pub year: i32,
    pub month: u32,
    pub day: Option<u32>,
}

pub struct PageArgs {
    pub book: BookEntry,
    pub accounts: HashMap<domain::CategoryId, Vec<AccountEntry>>,
    pub transactions: Vec<TransactionReport>,
    pub dates: Vec<domain::RawInt>,
    pub year: i32,
    pub month: u32,
    pub next_year: i32,
    pub next_month: u32,
    pub prev_year: i32,
    pub prev_month: u32,
    pub day: Option<u32>,
}

pub async fn history(
    service: impl TransactionService + BookService,
    args: HistoryArgs,
) -> Result<PageArgs> {
    let book = service
        .find_book(args.cabinet.into(), args.book.into())
        .await?;

    let accounts = service.list_accounts(&book.id).await?;
    let mut account_ids = if let Some(id) = args.account {
        vec![id]
    } else {
        Vec::new()
    };
    let mut account_map = HashMap::<domain::CategoryId, Vec<AccountEntry>>::new();
    for ac in accounts.into_iter() {
        if args.account.is_none() {
            if ac.major_category == domain::MajorCategory::Expense
                || ac.major_category == domain::MajorCategory::Income
                || ac.category_id == *domain::category::UNCLASSIFIED
            {
                account_ids.push(ac.id);
            }
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

    let filter = if let Some(day) = args.day {
        let date = NaiveDate::from_ymd_opt(args.year as i32, args.month, day).unwrap();
        Filter::new(date, date)
    } else {
        Filter::new(
            NaiveDate::from_ymd_opt(args.year, args.month, 1).unwrap(),
            NaiveDate::from_ymd_opt(args.year, args.month + 1, 1)
                .unwrap()
                .pred_opt()
                .unwrap(),
        )
    };

    let transactions = service.find_transactions(&account_ids, &filter).await?;
    let mut sum_dates = Vec::with_capacity(31);
    for _ in filter.iter_days() {
        sum_dates.push(0)
    }
    for t in transactions.iter() {
        if t.unit_id == book.unit_id {
            let d = t.date.day0() as usize;
            sum_dates[d] += t.debit as domain::RawInt - t.credit as domain::RawInt;
        }
    }

    Ok(PageArgs {
        book,
        accounts: account_map,
        transactions,
        dates: sum_dates,
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
        day: args.day,
    })
}
