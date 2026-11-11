# AGENTS.md

Periodless
は、紙の複式簿記（日商簿記3級範囲・2022年度改定基準）をそのままデジタル化する
Rust 製会計コアライブラリです。 AI
エージェントは設計・実装・テスト・コミットを行う際、常に以下の原則と規約を厳守してください。

---

## 1. ドキュメントマップ（作業前に必ず参照）

- [`README.md`](README.md) :
  アーキテクチャ思想（不変履歴、導出ビュー、Port/Adapter分離、Agentic TDD）
- [`docs/README.md`](docs/README.md) :
  製品仕様・準拠範囲（日商簿記3級）・受入条件の骨子
- [`CONTRIBUTING.md`](CONTRIBUTING.md) :
  **コーディング規約・基本用語の日英対訳・Git/PRルールの正式リファレンス**
- `docs/勘定科目一覧/` : **勘定科目の英語識別子（`UpperCamelCase` /
  `snake_case`）の唯一の真実の源泉**
  - [`資産.md`](docs/勘定科目一覧/資産.md) /
    [`負債.md`](docs/勘定科目一覧/負債.md) /
    [`純資産.md`](docs/勘定科目一覧/純資産.md) /
    [`収益.md`](docs/勘定科目一覧/収益.md) /
    [`費用.md`](docs/勘定科目一覧/費用.md) /
    [`その他・評価勘定.md`](docs/勘定科目一覧/その他・評価勘定.md)

---

## 2. ドメインの絶対不変条件

1. **追記専用（Append-Only）と不変性**:
   - 記帳済みの仕訳（`JournalEntry`）および伝票（`Slip`）に対する `UPDATE` /
     `DELETE` は一切存在しない。訂正・取消は新しい仕訳や伝票の追加で表現する。
   - 下書き状態は持たない。
2. **貸借一致の強制**:
   - 一仕訳の借方（`Debit`）合計と貸方（`Credit`）合計が一致しない記録は必ず型または
     `Result::Err` で拒否する。
3. **時計を持たない単調増加の時系列**:
   - 日時は外部入力として受け取る。最後に受け付けた仕訳より前の日時の仕訳は必ず拒否する。同一日時は到達順序で順序づける。
4. **財務諸表はすべて導出ビュー**:
   - 仕訳帳（`GeneralJournal`）、損益計算書（`IncomeStatement`）、貸借対照表（`BalanceSheet`）は不変の仕訳履歴から任意の期間・到達位置を指定して計算するビューであり、帳簿を変更しない。
5. **コアの純粋性**:
   - ドメインコアに特定の DB・ストレージ・フレームワーク依存を持ち込まない。

---

## 3. コーディング規約（厳選ルール）

詳細は [`CONTRIBUTING.md`](CONTRIBUTING.md) を参照してください。

- **行コメント（`//`）の原則禁止**:
  - コード自体（型・関数抽出・変数名）で意図を示す。`//`
    は外部制約など説明しないと背景事情（Why）が伝わらない例外的な場合のみ許可し、処理内容（What
    / How）の説明には書かない。
- **`///`（rustdoc）と doctest の推進**:
  - 公開 API（`pub`）には `///` を推奨し、`cargo test --doc` で実行可能な
    doctest を充実させる。
- **公開 API のフルーエントインターフェース**:
  - 公開
    API（仕訳・伝票の構築、記帳、期間・位置指定クエリ等）はフルーエントインターフェースとメソッドチェーンを中心に設計する。
- **日英識別子の厳格な統一**:
  - プロダクションコードの識別子は英語に統一する。
  - `Data`, `Record`, `Item`, `Manager`, `Util`
    等の汎用名を禁止し、`CONTRIBUTING.md` および `docs/勘定科目一覧/*.md`
    に定義された英語識別子をそのまま使う（独自の別訳を作らない）。
- **パニックの完全禁止と Newtype**:
  - プロダクションコードでの `unwrap()`, `expect()`, `panic!()`, `todo!()`,
    `unimplemented!()`, スライス直添字アクセスは禁止。
  - 金額に浮動小数点（`f32`, `f64`）は使用禁止。ドメイン制約違反はすべて明示的な
    `Result<T, E>` と Newtype で表現する。
- **Rust 変更時のローカル検証と CI 必須チェック**:
  - Rust コードを変更したら、コミット前に
    `cargo test --workspace --all-targets`, `cargo test --doc`,
    `cargo clippy --workspace --all-targets -- -D warnings`,
    `cargo fmt --all -- --check` の4コマンドをすべて成功させる。
  - これらは `.github/workflows/rust.yml` の `cargo test` / `cargo clippy` /
    `cargo fmt --check` ジョブとして `main` 向け PR
    の必須チェックに登録されており、 失敗するとマージできない。clippy の警告は
    `allow` で抑制せずコードを修正する。
- **Agentic TDD と日本語テスト関数名**:
  - `OpenSpec → 失敗するテスト（Red） → 最小実装（Green） → リファクタリング → rustdoc / doctest`
    の順序を守る。
  - `#[test]` 関数名には日本語（例:
    `fn 貸借が不一致の仕訳は拒否される()`）を推奨・許容する（テストモジュールに
    `#[allow(non_ascii_idents)]` を付与）。

---

## 4. Git Commit / PR ルール

- **Commit**: スコープ付き Conventional Commits
  形式（`<type>(<scope>): <subject>`）の **1行のみ（Body・Footer
  なし）**。`<subject>` は日本語で記述する。
  - 例: `feat(journal): 貸借一致を検証する仕訳ビルダーを追加`
- **PR**: **1 PR = 1関心事**。コード差分で変更を自明にし、PR
  本文には「背景・目的（Why）」と「関連 Spec / Issue」のみを簡潔に書く。
- **Formatter**: Markdown / YAML / JSON は `deno fmt` で整形する（コミット前に
  `deno fmt --check` が通ることを確認する）。
