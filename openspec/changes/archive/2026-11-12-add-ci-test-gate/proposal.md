# Proposal

## Why

現在のCIは `deno fmt --check`
によるドキュメント・設定ファイルの整形検証のみで、Rustコードのビルド・テストが一度も検証されていない。そのためテストが失敗する状態やコンパイルできない状態のままでも
`main` へのPRをマージできてしまい、Agentic TDD・パニック禁止・rustdoc/doctest
充実といった既存規約が人間とAIエージェントの自己申告に委ねられている。

## What Changes

- 既存のフォーマット系ワークフロー（`format-check.yml` /
  `format-pr.yml`）とは独立した、Rust専用のGitHub Actionsワークフロー
  `.github/workflows/rust.yml` を新設する。
- Rustワークフローは読み取り専用権限（`contents: read`）で、`test`・`clippy`・`fmt`
  の3ジョブを実行する。
  - `test`: `cargo test --workspace --all-targets` および `cargo test --doc`
  - `clippy`: `cargo clippy --workspace --all-targets -- -D warnings`
  - `fmt`: `cargo fmt --all -- --check`
- `main`
  ブランチのマージ前チェックとして、Rustワークフローの各ジョブと既存のフォーマット検証CIを必須ステータスチェックに登録する運用手順を
  `CONTRIBUTING.md`
  に定義する（GitHub側のルールセット設定自体はリポジトリ管理者が実施する）。
- 既存のワークフローファイル（`format-check.yml`, `format-pr.yml`,
  `greetings.yml`, `label.yml`）は変更しない。

## Capabilities

### New Capabilities

- `rust-verification-ci`: Rustワークスペースのテスト・doctest・lint・整形をPull
  Requestおよび `main` への push
  で検証する、既存ワークフローから独立した必須チェックCIの仕様と、`main`
  ブランチ保護における必須ステータスチェック登録の運用規定

### Modified Capabilities

- `coding-standards`:
  Rustコードの変更時にローカルで実行すべき事前検証コマンド（`cargo test` /
  `cargo test --doc` / `cargo clippy` /
  `cargo fmt --check`）と、それらがCIの必須チェックとして強制されることの規約化

## Impact

- 新規ファイル:
  - `.github/workflows/rust.yml`
- 更新ファイル:
  - `CONTRIBUTING.md`（CI必須チェックと `main` ルールセット設定手順の追記）
  - `AGENTS.md`（CI必須チェックの要点追記）
  - `openspec/config.yaml`（プロジェクトコンテキストへのCI必須チェック追記）
- 運用への影響:
  - `main` 向けPRは `test` / `clippy` / `fmt` / `deno fmt --check`
    がすべて成功しないとマージできなくなる。
  - 既存コードが `cargo clippy -D warnings` または `cargo fmt --check`
    に違反している場合、CI導入時点で失敗するため本変更内で解消する必要がある。
- 依存関係: GitHub Actions の外部アクション（`dtolnay/rust-toolchain`,
  `Swatinem/rust-cache`）をCIに追加する。Rustクレートの依存関係は追加しない。
