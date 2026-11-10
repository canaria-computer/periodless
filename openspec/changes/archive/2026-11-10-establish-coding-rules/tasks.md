# Tasks

## 1. 勘定科目一覧のLLM参照用日英対訳テーブル化

- [x] 1.1 `docs/勘定科目一覧/資産.md` を「日本語科目名・型/バリアント名(`UpperCamelCase`)・変数/メソッド名(`snake_case`)・区分/備考」のMarkdownテーブル形式に更新し、既存の全27科目が漏れなく定義されていることを確認する
- [x] 1.2 `docs/勘定科目一覧/負債.md` を同様のMarkdownテーブル形式に更新し、既存の全19科目が漏れなく定義されていることを確認する
- [x] 1.3 `docs/勘定科目一覧/純資産.md`、`docs/勘定科目一覧/収益.md`、`docs/勘定科目一覧/費用.md`、`docs/勘定科目一覧/その他・評価勘定.md` を同様のMarkdownテーブル形式に更新し、既存の全科目（純資産3・収益7・費用18・その他4）が漏れなく定義されていることを確認する

## 2. コーディング規約（CONTRIBUTING.md）の作成

- [x] 2.1 リポジトリ直下に `CONTRIBUTING.md` を作成し、基本原則、コメント（`//`原則禁止・背景事情のみ例外）と`///`（rustdoc/doctest推奨）の境界、公開APIにおけるフルーエントインターフェース/メソッドチェーン、基本ドメイン用語の日英対訳と`docs/勘定科目一覧/`への参照、型設計と`unwrap`/`expect`/`panic!`禁止、Agentic TDDと日本語テスト関数名、Git Commitルール（CC・1行完結・Bodyなし・日本語subject）、PRルール（1関心事・Whyのみの最小本文）を明記して内容を確認する

## 3. AIエージェント向け指針（AGENTS.md）と OpenSpec コンテキストの整備

- [x] 3.1 リポジトリ直下に `AGENTS.md` を作成し、AIエージェントが遵守すべき厳選ルール・ドメイン不変条件・ドキュメントマップ（`README.md`, `docs/README.md`, `CONTRIBUTING.md`, `docs/勘定科目一覧/*.md`）を記載して内容を確認する
- [x] 3.2 `openspec/config.yaml` の `context` フィールドにプロジェクトの技術スタック・ドメイン原則・コーディング規約の要点を設定し、`openspec validate establish-coding-rules --strict` が成功することを確認する
