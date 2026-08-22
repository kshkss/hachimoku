# ADR Index (app特化型アーキテクチャ決定記録) - v2

このインデックスは、家計簿アプリ **hachimoku** のクライアント/ローカルアプリケーション（`/app`）における技術選定、設計パターン、ドメインモデリングに関するアーキテクチャ決定（ADR）を網羅したものです。

v2へのアップデートに伴い、同期サーバー（`/sync`）や外部スクレイピング（ブラウザ拡張）などの外部境界ドメインを意図的に排除し、**ローカルアプリ（クライアント側）の自律性と堅牢性、開発体験（DX）を最大化するための設計決定**に特化して再編成しました。

## ADR一覧

| ID | ステータス | タイトル | 概要 | 関連レイヤー |
|---|---|---|---|---|
| [ADR-002-v2](./adr-002-frontend-htmx-alpine-maud-v2.md) | **決定** | [軽量フロントエンドUIスタック (Maud + HTMX + Alpine.js)](./adr-002-frontend-htmx-alpine-maud-v2.md) | SPAを避け、サーバーサイド主導のHTMXと、クライアント状態を担うAlpine.js、コンパイル時に型検証されるMaudテンプレートを組み合わせ、優れたDXとユーザー体験を両立する。 | UI・プレゼンテーション層 |
| [ADR-004-v2](./adr-004-ui-four-buckets-v2.md) | **決定** | [「4つのバケツ」とUI表示マッピング](./adr-004-ui-four-buckets-v2.md) | ドメイン内の複式簿記5分類を、UI層で直感的な4つのバケツ（資金・運用中・支払予定・保留中）へ再マップし、おもてなしUI配色ルール（資産/負債のプラス・マイナス表現）を適用する。 | UI・プレゼンテーション層 |
| [ADR-005-v2](./adr-005-cabinet-book-journal-aggregation-v2.md) | **決定** | [Cabinet集約のType State管理とBook/Journal集約の完全分離](./adr-005-cabinet-book-journal-aggregation-v2.md) | Cabinetの所有/参照権限をType Stateパターンでコンパイル時に保証し、仕訳（Journal）データのオンメモリ肥大化を防ぐためにBookとJournalの集約を完全に分離する。 | ドメイン層 (Domain Layer) |
| [ADR-006-v2](./adr-006-unit-of-work-witness-v2.md) | **決定** | [Witnessパターンによる型安全なトランザクション境界制御](./adr-006-unit-of-work-witness-v2.md) | トランザクション外での不用意なデータ永続化（オートコミット）を静的に防ぐため、UowToken（Unit of Work Token）を引数に要求するWitness（Capability）パターンを採用する。 | アプリケーション層 / インフラ層 |
| [ADR-007-v2](./adr-007-wasm-service-worker-opfs-v2.md) | **決定** | [WASM ＋ Service Worker ＋ OPFS SQLite によるブラウザ内ローカル永続化](./adr-007-wasm-service-worker-opfs-v2.md) | クライアントをインストール不要なWebアプリとして提供するためのPoC。Service Workerでfetchを中継し、WASM内のGo/RustからOPFS（Origin Private File System）上のSQLiteへ同期永続化する。 | ローカルインフラ層 (WASM/PoC) |
| [ADR-008-v2](./adr-008-domain-multicurrency-position-account-v2.md) | **決定** | [多通貨・マルチアセットを処理する「1口座1通貨」およびポジション勘定設計](./adr-008-domain-multicurrency-position-account-v2.md) | 外貨や株式等の多通貨・マルチアセット取引を簿記的・数学的に破綻なく処理するため、すべての口座に「1口座1通貨」の制約を強制し、交換時にはPosition Account（ポジション勘定）を経由する。 | ドメイン層 (Domain Layer) |

## アーキテクチャの基本方針（クライアント主導・ローカルファースト）

`/app` は、サーバーが消失したりオフライン状態であっても、手元のデバイスだけで完全に動作（データの閲覧・記帳・計算）できることを絶対の不変条件とします。
ドメイン層にはRust（または検証用WASMとしてのGo/Rust）を据え、インフラ（DB）やUIフレームワークの仕様変更、さらにはネットワークの接続状態に左右されない、ピュアな個人財務ドメインモデルを構築します。
