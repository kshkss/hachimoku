use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{Datelike, Local};
use hypertext::Renderable;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain;
use crate::mock::MockServices;
use crate::usecase;
use crate::usecase::accounts::AccountType;
use crate::usecase::pnl::PNLType;
use crate::views;

pub fn build_router() -> Router {
    Router::new()
        .route("/{owner}/{book}/accounts/{type}", get(accounts_handler))
        .route("/{owner}/{book}/pnl/{year}/{month}", get(pnl_handler))
        .route("/{owner}/{book}/shop/{year}/{month}", get(shop_handler))
        .route("/{owner}/{book}/history", get(history_list_handler))
        .route(
            "/{owner}/{book}/history/{account}",
            get(history_detail_handler),
        )
}

fn bad_request(err: impl std::fmt::Display) -> Response {
    (StatusCode::BAD_REQUEST, err.to_string()).into_response()
}

fn internal_error(err: impl std::fmt::Display) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
}

fn is_hx_request(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}

fn respond(
    headers: &HeaderMap,
    title: &str,
    content: impl Renderable,
    sidebar: impl Renderable,
    header: impl Renderable,
    bottom_nav: impl Renderable,
) -> Response {
    let html = if is_hx_request(headers) {
        views::partial_page(title, content, sidebar, header, bottom_nav)
            .render()
            .into_inner()
    } else {
        views::full_page(title, content, sidebar, header, bottom_nav)
            .render()
            .into_inner()
    };
    Html(html).into_response()
}

async fn accounts_handler(
    Path((owner, book, type_)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Response {
    let cabinet = match domain::CabinetName::new(owner) {
        Ok(c) => c,
        Err(e) => return bad_request(e),
    };
    let book_slug = match domain::BookSlug::new(book) {
        Ok(b) => b,
        Err(e) => return bad_request(e),
    };
    let type_ = match type_.as_str() {
        "asset" => AccountType::Asset,
        "liability" => AccountType::Liability,
        _ => AccountType::All,
    };
    let today = Local::now().date_naive();
    let year = today.year();
    let month = today.month();

    let args = usecase::accounts::AccountArgs {
        cabinet,
        book: book_slug,
        type_,
        year,
        month,
    };
    let page = match usecase::accounts::accounts(MockServices, args).await {
        Ok(p) => p,
        Err(e) => return internal_error(e),
    };

    respond(
        &headers,
        views::ACCOUNTS_TITLE,
        views::account::content(&page, type_),
        views::account::sidebar(&page, type_, year, month),
        views::account::mobile_header(),
        views::account::bottom_nav(&page, type_, year, month),
    )
}

async fn pnl_handler(
    Path((owner, book, year, month)): Path<(String, String, i32, u32)>,
    headers: HeaderMap,
) -> Response {
    let cabinet = match domain::CabinetName::new(owner) {
        Ok(c) => c,
        Err(e) => return bad_request(e),
    };
    let book_slug = match domain::BookSlug::new(book) {
        Ok(b) => b,
        Err(e) => return bad_request(e),
    };

    // NOTE: usecase::pnl::pnl は PNLType::Expense/Income のどちらか片方の
    // summary しか返さないが、views::pnl::content は収入・支出両方を
    // 表示する前提になっている（既存の不整合）。ここでは暫定的に
    // Expense を渡す。
    let args = usecase::pnl::PNLArgs {
        cabinet,
        book: book_slug,
        type_: PNLType::Expense,
        year,
        month,
    };
    let page = match usecase::pnl::pnl(MockServices, args).await {
        Ok(p) => p,
        Err(e) => return internal_error(e),
    };

    respond(
        &headers,
        views::PNL_TITLE,
        views::pnl::content(&page),
        views::pnl::sidebar(&page),
        views::pnl::mobile_header(),
        views::pnl::bottom_nav(&page),
    )
}

async fn shop_handler(
    Path((owner, book, year, month)): Path<(String, String, i32, u32)>,
    headers: HeaderMap,
) -> Response {
    let cabinet = match domain::CabinetName::new(owner) {
        Ok(c) => c,
        Err(e) => return bad_request(e),
    };
    let book_slug = match domain::BookSlug::new(book) {
        Ok(b) => b,
        Err(e) => return bad_request(e),
    };

    let args = usecase::shop::ShopArgs {
        cabinet,
        book: book_slug,
        year,
        month,
    };
    let page = match usecase::shop::accounts(MockServices, args).await {
        Ok(p) => p,
        Err(e) => return internal_error(e),
    };

    respond(
        &headers,
        views::SHOP_TITLE,
        views::shop::content(&page),
        views::shop::sidebar(&page),
        views::shop::mobile_header(),
        views::shop::bottom_nav(&page),
    )
}

#[derive(Deserialize)]
struct HistoryQuery {
    month: Option<String>,
}

async fn history_list_handler(
    Path((owner, book)): Path<(String, String)>,
    Query(query): Query<HistoryQuery>,
    headers: HeaderMap,
) -> Response {
    render_history(owner, book, None, query.month, &headers).await
}

async fn history_detail_handler(
    Path((owner, book, account)): Path<(String, String, String)>,
    Query(query): Query<HistoryQuery>,
    headers: HeaderMap,
) -> Response {
    render_history(owner, book, Some(account), query.month, &headers).await
}

async fn render_history(
    owner: String,
    book: String,
    account: Option<String>,
    month: Option<String>,
    headers: &HeaderMap,
) -> Response {
    let cabinet = match domain::CabinetName::new(owner) {
        Ok(c) => c,
        Err(e) => return bad_request(e),
    };
    let book_slug = match domain::BookSlug::new(book) {
        Ok(b) => b,
        Err(e) => return bad_request(e),
    };

    let account_id = account.as_deref().and_then(|s| match s {
        "" | "all" | "0" => None,
        s => Uuid::parse_str(s).ok().map(domain::AccountId::new),
    });

    let today = Local::now().date_naive();
    let (year, month) = month
        .as_deref()
        .and_then(parse_year_month)
        .unwrap_or((today.year(), today.month()));

    let args = usecase::history::HistoryArgs {
        cabinet,
        book: book_slug,
        account: account_id,
        year,
        month,
        day: None,
    };
    let page = match usecase::history::history(MockServices, args).await {
        Ok(p) => p,
        Err(e) => return internal_error(e),
    };

    let current_account = account.as_deref().unwrap_or("");
    let current_account_name = account_id
        .and_then(|id| {
            page.accounts
                .values()
                .flatten()
                .find(|ac| ac.id == id)
                .map(|ac| ac.name.clone())
        })
        .unwrap_or_default();

    respond(
        headers,
        views::HISTORY_TITLE,
        views::history::content(&page, current_account, &current_account_name),
        views::history::sidebar(&page, current_account),
        views::history::mobile_header(),
        views::history::bottom_nav(&page),
    )
}

/// "YYYY-MM" 形式の文字列を (year, month) にパースする。Go 版の
/// `Sscanf` 検証と同等の緩いバリデーション（範囲外は None を返すのみ）。
fn parse_year_month(s: &str) -> Option<(i32, u32)> {
    let (y, m) = s.split_once('-')?;
    let year: i32 = y.parse().ok()?;
    let month: u32 = m.parse().ok()?;
    if (1..=12).contains(&month) {
        Some((year, month))
    } else {
        None
    }
}
