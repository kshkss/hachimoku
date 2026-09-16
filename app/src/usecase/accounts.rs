use chrono::NaiveDate;

use crate::domain;
use crate::usecase::{
    AccountEntry, AccountReport, AccountService, BookEntry, BookService, EPOCH_DATE, Filter, Result,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    All,
    Asset,
    Liability,
}

pub struct AccountArgs {
    pub cabinet: domain::CabinetName,
    pub book: domain::BookSlug,
    pub type_: AccountType,
    pub year: i32,
    pub month: u32,
}

pub struct PageArgs {
    pub book: BookEntry,
    pub accounts: HashMap<domain::CategoryId, Vec<AccountEntry>>,
    pub summary: Vec<AccountReport>,
    pub sum_asset: domain::RawInt,
    pub sum_liability: domain::RawInt,
}

pub async fn accounts(
    service: impl AccountService + BookService,
    args: AccountArgs,
) -> Result<PageArgs> {
    let book = service
        .find_book(args.cabinet.into(), args.book.into())
        .await?;

    let accounts = service.list_accounts(&book.id).await?;
    let mut assets = Vec::<domain::AccountId>::new();
    let mut liabilities = Vec::<domain::AccountId>::new();
    let mut account_map = HashMap::<domain::CategoryId, Vec<AccountEntry>>::new();
    for ac in accounts.into_iter() {
        match ac.major_category {
            domain::MajorCategory::Asset => {
                assets.push(ac.id);
            }
            domain::MajorCategory::Liability if ac.category_id != *domain::category::SHOP => {
                liabilities.push(ac.id);
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
        from: EPOCH_DATE,
        to: NaiveDate::from_ymd_opt(args.year, args.month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap(),
        source: None,
        destination: None,
    };

    let assets = service.find_accounts(&assets, &filter).await?;
    let mut sum_asset = 0;
    for a in assets.iter() {
        if a.unit_id == book.unit_id {
            sum_asset += a.debit as domain::RawInt - a.credit as domain::RawInt;
        }
    }
    let liabilities = service.find_accounts(&liabilities, &filter).await?;
    let mut sum_liability = 0;
    for a in liabilities.iter() {
        if a.unit_id == book.unit_id {
            sum_liability += a.debit as domain::RawInt - a.credit as domain::RawInt;
        }
    }

    let mut summary;
    match args.type_ {
        AccountType::All => {
            summary = assets;
            summary.extend(liabilities);
        }
        AccountType::Asset => {
            summary = assets;
        }
        AccountType::Liability => {
            summary = liabilities;
        }
    }

    Ok(PageArgs {
        book,
        accounts: account_map,
        summary,
        sum_asset,
        sum_liability,
    })
}
