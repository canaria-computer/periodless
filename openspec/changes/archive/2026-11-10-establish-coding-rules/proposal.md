# Proposal

## Why

Periodlessの実装を開始するにあたり、人間とLLM（AIエージェント）が共通の前提でAgentic
TDDを進められるよう、コーディング規約・Git/PR運用・日商簿記3級のユビキタス言語（勘定科目およびドメイン用語の日英識別子対訳）を明文化して確立する必要がある。

## What Changes

- `CONTRIBUTING.md`
  を新設し、以下のコーディング規約および開発プロセスを定義する：
  - コード自体で意図を示し、行コメント（`//`）は特殊な背景事情の補足以外では原則禁止するルール
  - 公開APIへの
    `///`（rustdoc）推奨と実行可能なドキュメントテスト（doctest）の充実
  - 公開API設計を中心としたフルーエントインターフェースおよびメソッドチェーンの採用
  - プロダクションコードにおける `unwrap` / `expect` / `panic!`
    の禁止と、Newtypeおよび `Result` による不変条件の型表現
  - Agentic TDD のサイクルと、テスト関数名における日本語識別子の推奨・許容
  - Git Commit ルール（Conventional
    Commits、ヘッダ1行完結、Bodyなし、日本語subject）
  - Pull Request ルール（1 PR =
    1関心事、コード差分で変更を自明にし、本文はWhyと関連Spec/Issueのみ）
- `docs/勘定科目一覧/` 配下の6ファイル（`資産.md`, `負債.md`, `純資産.md`,
  `収益.md`, `費用.md`,
  `その他・評価勘定.md`）を改変し、単なる日本語の羅列から、LLMがコード生成・レビュー時に直接参照できる「日本語科目名・英語型/バリアント名（`UpperCamelCase`）・英語変数/メソッド名（`snake_case`）・貸借区分/性質」の構造化テーブルへ刷新する。
- リポジトリ直下に `AGENTS.md`
  を新設し、AIエージェントが常に遵守すべき最重要原則・コーディング規約・参照先ドキュメントへのポインタを厳選して記載する。
- `openspec/config.yaml` の `context` を更新し、OpenSpec
  の各フェーズでも同じ規約と用語体系が自動的に適用されるようにする。

## Capabilities

### New Capabilities

- `coding-standards`:
  プロジェクト全体のコーディング規約、API設計方針、型安全・エラー処理、Agentic
  TDD・テスト規約、Git/PR運用ルール、およびAIエージェント向けガイドライン（`CONTRIBUTING.md`,
  `AGENTS.md`）
- `account-terminology`:
  日商簿記3級準拠の勘定科目および会計ドメイン用語における日英識別子対訳仕様（`docs/勘定科目一覧/*.md`）

### Modified Capabilities

## Impact

- 新規ファイル: `CONTRIBUTING.md`, `AGENTS.md`
- 更新ファイル:
  - `docs/勘定科目一覧/資産.md`
  - `docs/勘定科目一覧/負債.md`
  - `docs/勘定科目一覧/純資産.md`
  - `docs/勘定科目一覧/収益.md`
  - `docs/勘定科目一覧/費用.md`
  - `docs/勘定科目一覧/その他・評価勘定.md`
  - `openspec/config.yaml`
- 今後のすべてのRustクレート実装（ドメインコア、アダプタ、テスト、PR/コミット運用）の基準となる。
