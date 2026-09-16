use crate::usecase::history::PageArgs;
use crate::views::{
    ACCOUNTS_SHORT_TITLE, ACCOUNTS_TITLE, HISTORY_SHORT_TITLE, HISTORY_TITLE, PNL_SHORT_TITLE,
    PNL_TITLE, SHOP_SHORT_TITLE, SHOP_TITLE, account_url, history_url, pnl_url, shop_url,
};
use hypertext::prelude::*;

pub fn sidebar(args: &PageArgs, current_account: &str) -> impl Renderable {
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
            <p class="menu-label mt-5">"口座別履歴"</p>
            <ul class="menu-list">
                @for ac in args.accounts.values().flatten().filter(|ac| ac.num_entries > 0) {
                    <li>
                        <a
                            class=(if ac.id.as_uuid().to_string() == current_account { "is-active" } else { "" })
                            hx-get=(history_url(owner, book_slug, &ac.id.as_uuid().to_string(), ""))
                            hx-target="#main-content"
                            hx-push-url="true"
                        >(&ac.name)</a>
                    </li>
                }
            </ul>
        </aside>
    }
}

pub fn mobile_header() -> impl Renderable {
    rsx! {
        <header id="mobile-header" class="app-header p-3 is-hidden-desktop" hx-swap-oob="true">
            <div class="container is-flex is-justify-content-space-between is-align-items-center">
                <h1 class="title is-5 mb-0">
                    <span>(HISTORY_TITLE)</span>
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
                class="nav-item"
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
                class="nav-item is-active"
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

pub fn content(
    args: &PageArgs,
    current_account: &str,
    current_account_name: &str,
) -> impl Renderable {
    let owner = "demo";
    let book_slug = &args.book.slug;

    rsx! {
        <div x-data="{ show_filter_options: false, isSelectionMode: false }">
            <div class="level mb-4 is-mobile">
                <div class="level-left">
                    <h2 class="title is-4 is-hidden-touch mb-0">
                        @if current_account.is_empty() { (HISTORY_TITLE) }
                        @else { (format!("{} - {}", HISTORY_TITLE, current_account_name)) }
                    </h2>
                    <h2 class="title is-5 is-hidden-desktop mb-0">
                        @if current_account_name.is_empty() { "すべて" }
                        @else { (current_account_name) }
                    </h2>
                </div>
                <div class="level-right">
                    <div class="buttons">
                        <button
                            class="button is-small is-hidden-desktop"
                            x-on:click="show_filter_options = !show_filter_options"
                        >
                            <span class="icon is-small"><i class="fas fa-filter"></i></span>
                            <span>"絞り込み"</span>
                        </button>
                    </div>
                </div>
            </div>

            <div class="box p-0 is-hidden-touch" style="overflow: visible;">
                <table class="table is-fullwidth is-hoverable">
                    <thead>
                        <tr>
                            <th style="width: 40px;"></th>
                            <th style="width: 130px;">"日付"</th>
                            <th>"内容"</th>
                            <th style="width: 18%;">"元(減)"</th>
                            <th style="width: 18%;">"先(増)"</th>
                            <th class="has-text-right">"金額"</th>
                            <th style="width: 80px;"></th>
                        </tr>
                    </thead>
                    <tbody>
                        @for tx in &args.transactions {
                            <tr x-on:click=(format!("handleRowClick('{}', $event)", tx.id.as_uuid().to_string())) style="cursor: pointer;">
                                <td style="text-align: center; vertical-align: middle;">
                                    <input type="checkbox" class="checkbox" value=(tx.id.as_uuid().to_string()) x-model="selectedTransactionIds" />
                                </td>
                                <td style="vertical-align: middle;">(tx.date.to_string())</td>
                                <td style="vertical-align: middle;"><span>(&tx.note)</span></td>
                                <td style="vertical-align: middle;"><div class="tag is-light is-danger mb-1"><span>(&tx.account_name)</span></div></td>
                                <td style="vertical-align: middle;"><div class="tag is-light is-success mb-1"><span>(&tx.account_name)</span></div></td>
                                <td class="has-text-right has-text-weight-bold" style="vertical-align: middle;">
                                    (format!("¥{}", tx.debit - tx.credit))
                                </td>
                                <td style="text-align: center; vertical-align: middle;">
                                    <button class="button is-small is-rounded is-white"><span class="icon is-small"><i class="fas fa-ellipsis-h"></i></span></button>
                                </td>
                            </tr>
                        }
                    </tbody>
                </table>
            </div>

            <div class="selection-action-bar" x-show="isSelectionMode" x-transition>
                <div class="container is-flex is-justify-content-space-between is-align-items-center">
                    <div>
                        <span class="has-text-weight-bold is-size-5" x-text="selectedTransactionIds.length + '件'"></span>
                        <span class="has-text-grey ml-1">"を選択中"</span>
                    </div>
                    <button class="button is-primary is-rounded" x-bind:disabled="selectedTransactionIds.length < 2">
                        <span class="icon"><i class="fas fa-object-group"></i></span>
                        <span>"まとめる"</span>
                    </button>
                </div>
            </div>
        </div>
    }
}
