# Spec Delta

## ADDED Requirements

### Requirement: Rustコード変更時のローカル事前検証

開発者およびAIエージェントは、Rustコードを変更した場合、コミットまたはPull
Request作成の前に、ワークスペース全体のテスト・ドキュメントテスト・lint・整形の検証をローカルで実行し、すべてが成功する状態を維持しなければならない（SHALL）。これらの検証は
`main` へのマージ前の必須チェックとしてCIでも強制される（SHALL）。

#### Scenario: Rustコードの変更をコミットする前

- **WHEN** 開発者またはAIエージェントがRustソースを追加または変更した場合
- **THEN**
  `cargo test --workspace --all-targets`、`cargo test --doc`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`
  をローカルで実行し、すべてエラーなく成功することを確認する

#### Scenario: lint警告が残っている場合

- **WHEN** `cargo clippy` が警告を1件以上報告する場合
- **THEN**
  警告を抑制属性で隠さずコード側を修正し、警告が0件の状態にしてからコミットする
