use chrono::NaiveDate;

use crate::domain;
use crate::usecase::{
    AccountEntry, AccountReport, AccountService, BookEntry, BookService, Filter, Result,
};
use std::collections::HashMap;

pub struct ShopArgs {
    pub cabinet: domain::CabinetName,
    pub book: domain::BookSlug,
    pub year: i32,
    pub month: u32,
}

pub struct PageArgs {
    pub book: BookEntry,
    pub accounts: HashMap<domain::CategoryId, Vec<AccountEntry>>,
    pub summary: Vec<AccountReport>,
    pub sum: domain::RawInt,
    pub year: i32,
    pub month: u32,
    pub next_year: i32,
    pub next_month: u32,
    pub prev_year: i32,
    pub prev_month: u32,
}

pub async fn accounts(
    service: impl AccountService + BookService,
    args: ShopArgs,
) -> Result<PageArgs> {
    let book = service
        .find_book(args.cabinet.into(), args.book.into())
        .await?;

    let accounts = service.list_accounts(&book.id).await?;
    let mut shop = Vec::<domain::AccountId>::new();
    let mut account_map = HashMap::<domain::CategoryId, Vec<AccountEntry>>::new();
    for ac in accounts.into_iter() {
        match ac.major_category {
            domain::MajorCategory::Liability if ac.category_id == *domain::category::SHOP => {
                shop.push(ac.id);
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

    let filter = Filter {
        from: NaiveDate::from_ymd_opt(args.year, args.month, 1).unwrap(),
        to: NaiveDate::from_ymd_opt(args.year, args.month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap(),
        source: None,
        destination: None,
    };

    let summaries = service.find_accounts(&shop, &filter).await?;
    let mut sum = 0;
    for a in summaries.iter() {
        if a.unit_id == book.unit_id {
            sum += a.debit as domain::RawInt - a.credit as domain::RawInt;
        }
    }

    Ok(PageArgs {
        book,
        accounts: account_map,
        summary: summaries,
        sum: sum,
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
