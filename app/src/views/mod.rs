use hypertext::prelude::*;

pub mod account;
pub mod history;
pub mod pnl;
pub mod shop;

pub const ACCOUNTS_TITLE: &str = "資産と負債";
pub const ACCOUNTS_SHORT_TITLE: &str = "口座";
pub const PNL_TITLE: &str = "1ヶ月の収支";
pub const PNL_SHORT_TITLE: &str = "収支";
pub const SHOP_TITLE: &str = "連携ショップ";
pub const SHOP_SHORT_TITLE: &str = "ショップ";
pub const HISTORY_TITLE: &str = "取引履歴";
pub const HISTORY_SHORT_TITLE: &str = "履歴";

pub fn account_url(owner: &str, book: &str, r#type: &str) -> String {
    format!("/{}/{}/accounts/{}", owner, book, r#type)
}

pub fn pnl_url(owner: &str, book: &str, year: i32, month: u32) -> String {
    format!("/{}/{}/pnl/{}/{:02}", owner, book, year, month)
}

pub fn shop_url(owner: &str, book: &str, year: i32, month: u32) -> String {
    format!("/{}/{}/shop/{}/{:02}", owner, book, year, month)
}

pub fn history_url(owner: &str, book: &str, account: &str, params: &str) -> String {
    if params.is_empty() {
        format!("/{}/{}/history/{}", owner, book, account)
    } else {
        format!("/{}/{}/history/{}?{}", owner, book, account, params)
    }
}

pub fn full_page(
    title: &str,
    content: impl Renderable,
    sidebar: impl Renderable,
    header: impl Renderable,
    bottom_nav: impl Renderable,
) -> impl Renderable {
    rsx! {
        <!DOCTYPE html>
        <html lang="ja">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />
                <title>(title) " - Hachimoku"</title>
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css" />
                <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css" />
                <link rel="stylesheet" href="/assets/style.css" />
                <script src="https://unpkg.com/htmx.org@1.9.10"></script>
                <script defer src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js"></script>
            </head>
            <body x-data="kakeiboUI">
                <div class="columns is-gapless is-fullheight">
                    <div class="column is-2-desktop is-hidden-touch sidebar">
                        (sidebar)
                    </div>
                    <div class="column is-10-desktop" style="position: relative;">
                        (header)
                        <main class="section px-3 py-5" x-bind:style="isSelectionMode ? 'padding-bottom: 100px;' : ''">
                            <div class="container" id="main-content">
                                (content)
                            </div>
                        </main>
                    </div>
                </div>
                (bottom_nav)
            </body>
        </html>
    }
}

pub fn partial_page(
    title: &str,
    content: impl Renderable,
    sidebar: impl Renderable,
    header: impl Renderable,
    bottom_nav: impl Renderable,
) -> impl Renderable {
    rsx! {
        (content)
        <title hx-swap-oob="true">(title) " - Hachimoku"</title>
        (sidebar)
        (header)
        (bottom_nav)
    }
}
