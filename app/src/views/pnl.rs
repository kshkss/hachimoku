use crate::usecase::pnl::PageArgs;
use crate::views::{
    ACCOUNTS_SHORT_TITLE, ACCOUNTS_TITLE, HISTORY_SHORT_TITLE, HISTORY_TITLE, PNL_SHORT_TITLE,
    PNL_TITLE, SHOP_SHORT_TITLE, SHOP_TITLE, account_url, history_url, pnl_url, shop_url,
};
use hypertext::prelude::*;

pub fn sidebar(args: &PageArgs) -> impl Renderable {
    let owner = "demo";
    let book_slug = &args.book.slug;

    rsx! {
        <aside id="sidebar-menu" class="menu sidebar-menu" hx-swap-oob="true">
            <div class="is-flex is-align-items-center mb-5 px-2">
                <span class="icon has-text-primary is-size-4 mr-2"><i class="fas fa-wallet"></i></span>
                <span class="has-text-weight-bold is-size-5">"Hachimoku"</span>
            </div>
            <p class="menu-label">"サマリー"</p>
            <ul class="menu-list">
                <li>
                    <a
                        hx-get=(account_url(owner, book_slug, "all"))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><i class="fas fa-wallet mr-2"></i>(ACCOUNTS_TITLE)</a>
                </li>
                <li>
                    <a
                        class="is-active"
                        hx-get=(pnl_url(owner, book_slug, args.year, args.month))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><i class="fas fa-chart-line mr-2"></i>(PNL_TITLE)</a>
                </li>
                <li>
                    <a
                        hx-get=(shop_url(owner, book_slug, args.year, args.month))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><i class="fas fa-store mr-2"></i>(SHOP_TITLE)</a>
                </li>
            </ul>
            <p class="menu-label">
                <i class="fas fa-history mr-2"></i>(HISTORY_TITLE)
            </p>
            <p class="menu-label mt-5">"現金"</p>
            <ul class="menu-list">
                <li>
                    <a
                        hx-get=(history_url(owner, book_slug, "1", ""))
                        hx-target="#main-content"
                        hx-push-url="true"
                    >"財布"</a>
                </li>
            </ul>
        </aside>
    }
}

pub fn mobile_header() -> impl Renderable {
    rsx! {
        <header id="mobile-header" class="app-header p-3 is-hidden-desktop" hx-swap-oob="true">
            <div class="container is-flex is-justify-content-space-between is-align-items-center">
                <h1 class="title is-5 mb-0">
                    <span>(PNL_TITLE)</span>
                </h1>
                <button class="button is-primary is-rounded is-small">
                    <span class="icon"><i class="fas fa-plus"></i></span>
                    <span>"記帳"</span>
                </button>
            </div>
        </header>
    }
}

pub fn bottom_nav(args: &PageArgs) -> impl Renderable {
    let owner = "demo";
    let book_slug = &args.book.slug;

    rsx! {
        <nav id="bottom-nav" class="bottom-nav is-hidden-desktop" hx-swap-oob="true" x-show="!isSelectionMode" x-transition>
            <div
                class="nav-item"
                hx-get=(account_url(owner, book_slug, "all"))
                hx-target="#main-content"
                hx-push-url="true"
            ><i class="fas fa-wallet"></i><span>(ACCOUNTS_SHORT_TITLE)</span></div>
            <div
                class="nav-item is-active"
                hx-get=(pnl_url(owner, book_slug, args.year, args.month))
                hx-target="#main-content"
                hx-push-url="true"
            ><i class="fas fa-chart-line"></i><span>(PNL_SHORT_TITLE)</span></div>
            <div
                class="nav-item"
                hx-get=(shop_url(owner, book_slug, args.year, args.month))
                hx-target="#main-content"
                hx-push-url="true"
            ><i class="fas fa-store"></i><span>(SHOP_SHORT_TITLE)</span></div>
            <div
                class="nav-item"
                hx-get=(history_url(owner, book_slug, "", ""))
                hx-target="#main-content"
                hx-push-url="true"
            >
                <div style="position: relative;">
                    <i class="fas fa-history"></i>
                    <span class="tag is-danger is-rounded" style="position: absolute; top: -5px; right: -10px; height: 16px; min-width: 16px; font-size: 0.6rem; padding: 0 4px;">"2"</span>
                </div>
                <span>(HISTORY_SHORT_TITLE)</span>
            </div>
        </nav>
    }
}

pub fn content(args: &PageArgs) -> impl Renderable {
    let owner = "demo";
    let book_slug = &args.book.slug;

    rsx! {
        <div>
            <div class="level is-mobile mb-5">
                <div class="level-left">
                    <button
                        class="button is-white"
                        hx-get=(pnl_url(owner, book_slug, args.prev_year, args.prev_month))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><span class="icon"><i class="fas fa-chevron-left"></i></span></button>
                </div>
                <div class="level-item has-text-centered">
                    <h2 class="title is-5 mb-0">(format!("{}年{}月の収支", args.year, args.month))</h2>
                </div>
                <div class="level-right">
                    <button
                        class="button is-white"
                        hx-get=(pnl_url(owner, book_slug, args.next_year, args.next_month))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><span class="icon"><i class="fas fa-chevron-right"></i></span></button>
                </div>
            </div>
            <div class="box mb-5">
                <div class="is-flex is-justify-content-space-between is-align-items-center mb-3">
                    <span class="has-text-grey">"収入合計"</span>
                    <span class="has-text-weight-bold is-size-5 has-text-success">(format!("¥{}", args.sum_income))</span>
                </div>
                <div class="is-flex is-justify-content-space-between is-align-items-center mb-3">
                    <span class="has-text-grey">"支出合計"</span>
                    <span class="has-text-weight-bold is-size-5 has-text-danger">(format!("¥{}", args.sum_expense))</span>
                </div>
                <hr class="my-3" />
                <div class="is-flex is-justify-content-space-between is-align-items-center">
                    <span class="has-text-weight-bold">"収支差額"</span>
                    <span class="title is-4 mb-0 has-text-info">(format!("¥{}", args.sum_income - args.sum_expense))</span>
                </div>
            </div>
            <div class="columns">
                <div class="column is-6">
                    <h3 class="label">"収入"</h3>
                    @for acc in args.summary.iter().filter(|acc| acc.debit < acc.credit) {
                        <div
                            class="account-row"
                            hx-get=(history_url(owner, book_slug, &acc.id.as_uuid().to_string(), &format!("month={:04}-{:02}", args.year, args.month)))
                            hx-target="#main-content"
                            hx-push-url="true"
                        >
                            <span>(&acc.name)</span>
                            <span class="has-text-weight-bold has-text-success">(format!("¥{}", (acc.debit as i64 - acc.credit as i64).abs()))</span>
                        </div>
                    }
                </div>
                <div class="column is-6">
                    <h3 class="label">"支出"</h3>
                    @for acc in args.summary.iter().filter(|acc| acc.debit >= acc.credit) {
                        <div
                            class="account-row"
                            hx-get=(history_url(owner, book_slug, &acc.id.as_uuid().to_string(), &format!("month={:04}-{:02}", args.year, args.month)))
                            hx-target="#main-content"
                            hx-push-url="true"
                        >
                            <span>(&acc.name)</span>
                            <span class="has-text-weight-bold has-text-danger">(format!("¥{}", (acc.debit as i64 - acc.credit as i64).abs()))</span>
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}
