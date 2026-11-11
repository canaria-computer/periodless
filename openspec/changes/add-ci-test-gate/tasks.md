# Tasks

## 1. ベースライン検証コマンドの確認

- [x] 1.1 現在のブランチ基底（`origin/main` 相当）で
      `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace --all-targets`、`cargo test --doc`
      の4コマンドをローカル実行し、すべて終了コード0で成功することを確認する
- [x] 1.2 1.1
      で失敗する検証があった場合、本変更内で最小限の修正を行うか、該当ジョブを後続変更へ切り出すかを判断し、判断結果を
      `design.md` の Risks に追記する

## 2. Rust専用検証ワークフローの実装

- [x] 2.1 `.github/workflows/rust.yml`
      を新設し、`name: Rust`、`permissions: contents: read`、トリガー
      `push: branches: [main]` および
      `pull_request: [opened, synchronize, reopened]`、`concurrency: rust-${{ github.ref }}`（`cancel-in-progress: true`）を定義して、`paths`
      フィルタを含まないことを確認する
- [x] 2.2 `test` ジョブ（`name: cargo test`）に
      `actions/checkout@v4`、`dtolnay/rust-toolchain@stable`、`Swatinem/rust-cache@v2`
      を構成し、`cargo test --workspace --all-targets` と `cargo test --doc`
      を実行するステップを定義して、ローカルで同一コマンドが成功することを確認する
- [x] 2.3 `clippy` ジョブ（`name: cargo clippy`）に
      `components: rustfmt, clippy`
      付きのツールチェインを構成し、`cargo clippy --workspace --all-targets -- -D warnings`
      を実行するステップを定義して、ローカルで同一コマンドが成功することを確認する
- [x] 2.4 `fmt` ジョブ（`name: cargo fmt --check`）で
      `cargo fmt --all -- --check`
      を実行するステップを定義し、ローカルで同一コマンドが成功することを確認する
- [x] 2.5 既存の
      `.github/workflows/format-check.yml`、`format-pr.yml`、`greetings.yml`、`label.yml`
      に差分がないことを `git diff --stat .github/workflows` で確認する
- [x] 2.6 `deno fmt .github` を適用し、`deno fmt --check`
      が成功することを確認する

## 3. 規約ドキュメントと必須チェック運用手順の整備

- [x] 3.1 `CONTRIBUTING.md` の CI/PR
      運用節に「Rust検証CI（必須チェック）」の項を追記し、`cargo test` /
      `cargo clippy` / `cargo fmt --check`
      の各ジョブの責務と、Rustコード変更時にローカルで実行すべき4コマンドを明記する
- [x] 3.2 `CONTRIBUTING.md` に `main` のルールセット設定手順（「Require a pull
      request before merging」と「Require status checks to pass before
      merging」を有効化し、必須チェックとして `cargo test` / `cargo clippy` /
      `cargo fmt --check` / `deno fmt --check`
      の4件を登録する）と、登録済みジョブ名を変更しない旨を追記する
- [x] 3.3 `AGENTS.md`
      のコーディング規約節に、Rustコードの変更時にローカルで4コマンドを通す必要があることとCIで必須チェックされる旨を1〜2行で追記する
- [x] 3.4 `openspec/config.yaml` の `context` に
      Rust検証CIの必須チェックを1行追記し、`openspec validate add-ci-test-gate --strict`
      が成功することを確認する
- [x] 3.5 `deno fmt` を適用し、`deno fmt --check` が成功することを確認する

## 4. 統合検証

- [x] 4.1 変更をブランチへ push して `main` 向けのPull Requestを作成し、`Rust`
      ワークフローの `cargo test` / `cargo clippy` / `cargo fmt --check`
      の3チェックと既存の `deno fmt --check`
      がすべて成功することをPR画面で確認する
- [x] 4.2
      意図的にテストを失敗させるコミット（または一時ブランチ）でRust検証CIが失敗（Fail）を返すことを確認し、確認後に元へ戻す
- [ ] 4.3 リポジトリ管理者が 3.2 の手順に従って `main`
      のルールセットへ4件の必須チェックを登録し、チェック未完了のPRでマージボタンがブロックされることを確認する
