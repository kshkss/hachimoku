use crate::usecase::accounts::{AccountType, PageArgs};
use crate::views::{
    ACCOUNTS_SHORT_TITLE, ACCOUNTS_TITLE, HISTORY_SHORT_TITLE, HISTORY_TITLE, PNL_SHORT_TITLE,
    PNL_TITLE, SHOP_SHORT_TITLE, SHOP_TITLE, account_url, history_url, pnl_url, shop_url,
};
use hypertext::prelude::*;

pub fn sidebar(
    args: &PageArgs,
    current_type: AccountType,
    year: i32,
    month: u32,
) -> impl Renderable {
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
                        class=(if current_type == AccountType::All { "is-active" } else { "" })
                    ><i class="fas fa-wallet mr-2"></i>(ACCOUNTS_TITLE)</a>
                </li>
                <li>
                    <a
                        hx-get=(pnl_url(owner, book_slug, year, month))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><i class="fas fa-chart-line mr-2"></i>(PNL_TITLE)</a>
                </li>
                <li>
                    <a
                        hx-get=(shop_url(owner, book_slug, year, month))
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
                    <span>(ACCOUNTS_TITLE)</span>
                </h1>
                <button class="button is-primary is-rounded is-small">
                    <span class="icon"><i class="fas fa-plus"></i></span>
                    <span>"記帳"</span>
                </button>
            </div>
        </header>
    }
}

pub fn bottom_nav(
    args: &PageArgs,
    current_type: AccountType,
    year: i32,
    month: u32,
) -> impl Renderable {
    let owner = "demo";
    let book_slug = &args.book.slug;

    rsx! {
        <nav id="bottom-nav" class="bottom-nav is-hidden-desktop" hx-swap-oob="true" x-show="!isSelectionMode" x-transition>
            <div
                class=(format!("nav-item {}", if current_type == AccountType::All { "is-active" } else { "" }))
                hx-get=(account_url(owner, book_slug, "all"))
                hx-target="#main-content"
                hx-push-url="true"
            ><i class="fas fa-wallet"></i><span>(ACCOUNTS_SHORT_TITLE)</span></div>
            <div
                class="nav-item"
                hx-get=(pnl_url(owner, book_slug, year, month))
                hx-target="#main-content"
                hx-push-url="true"
            ><i class="fas fa-chart-line"></i><span>(PNL_SHORT_TITLE)</span></div>
            <div
                class="nav-item"
                hx-get=(shop_url(owner, book_slug, year, month))
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

pub fn content(args: &PageArgs, current_type: AccountType) -> impl Renderable {
    let owner = "demo";
    let book_slug = &args.book.slug;

    rsx! {
        <div>
            <h2 class="title is-4 is-hidden-touch mb-5">(ACCOUNTS_TITLE)</h2>
            <div class="tabs is-toggle is-fullwidth is-small mb-4">
                <ul>
                    <li
                        class=(if current_type == AccountType::All { "is-active" } else { "" })
                        hx-get=(account_url(owner, book_slug, "all"))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><a>"すべて"</a></li>
                    <li
                        class=(if current_type == AccountType::Asset { "is-active" } else { "" })
                        hx-get=(account_url(owner, book_slug, "asset"))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><a>"資産"</a></li>
                    <li
                        class=(if current_type == AccountType::Liability { "is-active" } else { "" })
                        hx-get=(account_url(owner, book_slug, "liability"))
                        hx-target="#main-content"
                        hx-push-url="true"
                    ><a>"負債"</a></li>
                </ul>
            </div>
            <div class="columns is-multiline">
                @for acc in &args.summary {
                    <div class="column is-12-mobile is-6-tablet is-4-desktop">
                        <div
                            class="account-row"
                            hx-get=(history_url(owner, book_slug, &acc.id.as_uuid().to_string(), ""))
                            hx-target="#main-content"
                            hx-push-url="true"
                        >
                            <div>
                                <p class="has-text-weight-bold">(&acc.name)</p>
                                <p class="is-size-7 has-text-grey">(format!("{:?}", acc.major_category))</p>
                            </div>
                            <div class="has-text-right">
                                <p class=(format!("title is-5 mb-0 {}", if acc.debit >= acc.credit { "has-text-success" } else { "has-text-danger" }))>
                                    (format!("¥{}", (acc.debit as i64 - acc.credit as i64).abs()))
                                </p>
                            </div>
                        </div>
                    </div>
                }
            </div>
        </div>
    }
}
