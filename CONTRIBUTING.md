# Contributing & Coding Rules

Periodless
は、紙で行う複式簿記（日商簿記3級範囲）をそのままデジタル化する会計コアライブラリです。
本ドキュメントは、人間および LLM（AI
エージェント）が一貫した品質と設計思想でコードを書くためのコーディング規約および開発プロセスを定義します。

---

## 1. ドメイン基本原則

1. **主役は複式簿記そのもの**:
   - 汎用的な CRUD アプリケーションとして設計しません。
   - 記帳済みの仕訳（`JournalEntry`）や伝票（`Slip`）を上書き・物理削除（`UPDATE`
     /
     `DELETE`）する意味論をドメインに持ち込まないでください。訂正や取消は、新しい仕訳・伝票の追記（`append`）によって表現します。
2. **現在の状態は履歴から導出されるビュー**:
   - 仕訳帳、損益計算書、貸借対照表、残高は、可変な状態として保持するのではなく、不変な仕訳履歴から任意の期間・到達順序上の位置を指定して導出します。
3. **ドメインコアとアダプタの分離**:
   - 会計上の不変条件（貸借一致、日時の非逆行、勘定科目の妥当性など）はすべてドメインコアが責任を持ちます。
   - 永続化やキャッシュは小さな
     `trait`（port）を介して外部アダプタに委譲し、特定の DB
     やストレージ仕様をコアに持ち込みません。

---

## 2. コード表現とドキュメント（コメント・rustdoc・doctest）

### コード自体で意図を示す（`//` 行コメントの原則禁止）

- **何をしているか（What）・どうやっているか（How）を説明する `//`
  コメントは書かないでください。**
- コメントを書きたくなった場合は、まず以下のリファクタリングによってコード自体に意図を語らせます：
  - ドメインの概念を表す型（Newtype / Enum / Struct）を導入する
  - 意図を表す名前の小関数・メソッドへ抽出する
  - 説明的な変数名をつける
- **例外**:
  外部仕様の制約や非自明な回避策など、**コード構造だけでは「なぜそうせざるを得なかったか（Why）」が伝わらない特殊な事情がある場合にのみ**
  `//` の使用を認めます。

### `///`（rustdoc）の推奨とドキュメントテスト（doctest）の充実

- 公開 API（`pub` なクレート、モジュール、型、トレイト、メソッド、関数）には
  `///` による rustdoc の記述を推奨します。
- 単なる文章説明にとどめず、**実行可能な仕様（Executable Specification）として
  `///` 内のコード例（doctest）を充実させてください。**
- すべての doctest は `cargo test --doc`
  で常時検証され、生きた仕様書として機能します。

---

## 3. API 設計：フルーエントインターフェースとメソッドチェーン

- **公開 API の設計を中心に、フルーエントインターフェース（Fluent
  Interface）とメソッドチェーンを推奨します。**
- 適用対象の例：
  - 仕訳（`JournalEntry`）および伝票（`Slip`）の組み立て（借方・貸方の追加、日時の指定、確定）
  - 帳簿（`Ledger` / `GeneralJournal`）への記帳操作
  - 集計期間（開始・終了日時）や到達順序上の位置を指定した財務諸表（`IncomeStatement`,
    `BalanceSheet` 等）のクエリ構築
- メソッドチェーンの途中で不正な状態を素通しさせず、型状態（Typestate）または終端メソッドでの検証によって不変条件を強制してください。

---

## 4. 命名規則とユビキタス言語（日英識別子対訳）

- **プロダクションコードの識別子（型名・バリアント名・関数名・変数名・モジュール名）はすべて英語で統一します**（Rust
  標準の `UpperCamelCase` / `snake_case`）。
- `Data`, `Record`, `Item`, `Info`, `Manager`, `Processor`, `Util`, `Helper`
  といった会計上の意味を消してしまう汎用名は使用しないでください。

### 基本ドメイン用語の標準対訳

| 日本語用語          | 型・トレイト名 (`UpperCamelCase`)    | 変数・メソッド名 (`snake_case`)        | 備考                                         |
| :------------------ | :----------------------------------- | :------------------------------------- | :------------------------------------------- |
| 仕訳                | `JournalEntry`                       | `journal_entry`                        | 貸借一致する一単位の会計記録                 |
| 仕訳行（貸借要素）  | `JournalLine` / `EntryLine`          | `journal_line` / `entry_line`          | 勘定科目と金額の組                           |
| 借方                | `Debit`                              | `debit`                                | 左側要素                                     |
| 貸方                | `Credit`                             | `credit`                               | 右側要素                                     |
| 勘定科目            | `Account`                            | `account`                              | 詳細は `docs/勘定科目一覧/` を参照           |
| 勘定区分（5要素）   | `AccountCategory`                    | `account_category`                     | 資産・負債・純資産・収益・費用               |
| 資産                | `Asset`                              | `asset`                                | 定位置: 借方 (`Debit`)                       |
| 負債                | `Liability`                          | `liability`                            | 定位置: 貸方 (`Credit`)                      |
| 純資産（資本）      | `Equity`                             | `equity`                               | 定位置: 貸方 (`Credit`)                      |
| 収益                | `Revenue`                            | `revenue`                              | 定位置: 貸方 (`Credit`)                      |
| 費用                | `Expense`                            | `expense`                              | 定位置: 借方 (`Debit`)                       |
| 金額                | `Amount` / `Money`                   | `amount`                               | 浮動小数点 (`f32`/`f64`) は使用禁止          |
| 伝票                | `Slip`                               | `slip`                                 | 仕訳を生成する原憑記録                       |
| 仕訳帳              | `GeneralJournal`                     | `general_journal`                      | 記帳順の仕訳ビュー                           |
| 総勘定元帳          | `GeneralLedger`                      | `general_ledger`                       | 勘定科目別の元帳ビュー                       |
| 試算表              | `TrialBalance`                       | `trial_balance`                        | 合計・残高試算表ビュー                       |
| 損益計算書 (P/L)    | `IncomeStatement`                    | `income_statement`                     | 指定期間の経営成績ビュー                     |
| 貸借対照表 (B/S)    | `BalanceSheet`                       | `balance_sheet`                        | 指定基準日時の財政状態ビュー                 |
| 開始残高            | `OpeningBalance`                     | `opening_balance`                      | 帳簿開始時のみ許容される初期仕訳             |
| 訂正仕訳 / 取消仕訳 | `CorrectingEntry` / `ReversingEntry` | `correcting_entry` / `reversing_entry` | 既存事実を上書きせず追記で修正する仕訳       |
| 決算整理仕訳        | `AdjustingEntry`                     | `adjusting_entry`                      | 経過勘定・減価償却・引当金・売上原価等の整理 |
| 決算振替仕訳        | `ClosingEntry`                       | `closing_entry`                        | 損益勘定・繰越利益剰余金への振替             |

### 勘定科目の日英対訳リファレンス

個別の勘定科目（日商簿記3級範囲）の英語識別子については、独自の英訳を作らず、必ず以下のファイルに定義された識別子（`UpperCamelCase`
/ `snake_case`）を使用してください。

- [`docs/勘定科目一覧/資産.md`](docs/勘定科目一覧/資産.md)
- [`docs/勘定科目一覧/負債.md`](docs/勘定科目一覧/負債.md)
- [`docs/勘定科目一覧/純資産.md`](docs/勘定科目一覧/純資産.md)
- [`docs/勘定科目一覧/収益.md`](docs/勘定科目一覧/収益.md)
- [`docs/勘定科目一覧/費用.md`](docs/勘定科目一覧/費用.md)
- [`docs/勘定科目一覧/その他・評価勘定.md`](docs/勘定科目一覧/その他・評価勘定.md)

---

## 5. 型設計とエラーハンドリング

- **パニックの完全禁止**:
  - プロダクションコードにおいて `unwrap()`, `expect()`, `panic!()`, `todo!()`,
    `unimplemented!()`,
    および境界チェックなしのスライス添字アクセス（`slice[i]`）は禁止します。
- **Newtype パターンと不変条件の型表現（Parse, don't validate）**:
  - 金額や日時、識別子などを生のプリミティブ型（`u64`, `i64`, `String`
    等）のまま引き回さず、意味のある Newtype
    で包んで不変条件を型レベルで保証します。
  - 金額の表現に浮動小数点数（`f32`, `f64`）は絶対に使用しないでください。
- **明示的なドメインエラー**:
  - 「貸借不一致」「最後の仕訳より前の日時（日時逆行）」「簿記3級範囲外の記録」などのドメイン制約違反は、すべて呼び出し元が判別可能な専用のエラー型（`Result<T, DomainError>`）として返します。

---

## 6. Agentic TDD とテスト規約

### 開発サイクル

開発は **Agentic TDD** を基本パラダイムとし、以下のサイクルで進めます：

1. **OpenSpec / 合意済み設計**: 要求と受入条件を確認する
2. **Red（失敗するテスト）**: 期待するドメインの振る舞いを表すテストを先に書く
3. **Green（最小限の実装）**: テストを通す最小限の実装を行う
4. **Refactor（リファクタリング）**: 重複を排除し、型・命名・構造を洗練させる
5. **rustdoc / doctest**: 公開 API のドキュメントテストを整備し、`cargo test`
   で全テスト（単体・統合・doctest）を通す

### テスト関数名への日本語識別子の推奨・許容

- `#[test]` 関数の名前には、**`docs/README.md` や OpenSpec
  の受入条件と1対1で対応が分かる日本語の関数名を推奨・許容します。**
- 日本語識別子に対するコンパイラ警告を抑制するため、テストモジュールには
  `#[allow(non_ascii_idents)]`（またはファイル先頭の
  `#![cfg_attr(test, allow(non_ascii_idents))]`）を付与してください。
- テストコード内でも公開 API
  のフルーエントインターフェースを活用し、テストコード自体が仕様書として読めるように記述します。

```rust
#[cfg(test)]
#[allow(non_ascii_idents)]
mod tests {
    use super::*;

    #[test]
    fn 貸借が不一致の仕訳は拒否される() {
        // 公開APIのメソッドチェーンで意図をそのまま表現する
    }

    #[test]
    fn 最後の仕訳より前の日時を持つ仕訳は拒否される() {
        // ...
    }
}
```

---

## 7. Git Commit と Pull Request 運用ルール

### Git Commit ルール（スコープ付き Conventional Commits・Bodyなし）

- **コミットメッセージはスコープ付き Conventional Commits
  形式のヘッダ1行のみとし、Body（本文）および Footer は一切記述しません。**
- フォーマット：
  ```text
  <type>(<scope>): <subject>
  ```
- `<type>`: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
  のいずれかを使用します。
- `<scope>`: 変更対象のモジュールや領域（例: `journal`, `ledger`, `account`,
  `rules`, `spec`, `repo` 等）を簡潔に指定します。
- `<subject>`:
  **日本語**で変更の核心を簡潔に記述します（末尾に句点「。」は付けません）。
  - 良い例: `feat(journal): 貸借一致を検証する仕訳ビルダーを追加`
  - 良い例: `test(ledger): 日時逆行の仕訳が拒否されるテストを追加`
  - 良い例: `docs(rules): CONTRIBUTING.mdにコーディング規約を定義`
- **1行のコミットメッセージで説明しきれない変更は、コミットの粒度が大きすぎるサインです。**
  論理的な最小単位にコミットを分割してください。

### Pull Request ルール

- **コード差分（Diff）が変更を自明にする**:
  - 1 PR = 1つの明確な関心事に絞ります。
  - レビュアー（承認者）がコード差分とテスト名を見るだけで「何が変わったか（What
    / How）」を理解できる状態にします。
- **PR 本文は承認者が理解できる最低限にする**:
  - PR 本文でコード差分の要約を繰り返しません。
  - **「変更の背景・目的（Why）」** と **「関連する OpenSpec / Issue」**
    のみを数行で簡潔に記述してください。

### フォーマット規約（`deno fmt`）と整形用 PR の自動運用

- **Markdown / YAML / JSON のフォーマット**:
  - リポジトリ内の Markdown、YAML、JSON ファイル（`.agents/`, `.claude/`,
    `.commandcode/`, `.opencode/`, `.github/` を含む）はすべて `deno fmt`
    で整形します。
  - コミット前にローカルで `deno fmt`（検証は
    `deno fmt --check`）を実行してください。
- **CI における責務分離と整形用 PR の自動化**:
  - **検証 CI（必須チェック）**: 読み取り専用権限で `deno fmt --check`
    を実行し、未整形のファイルが含まれる PR の `main` マージを防止します。
  - **整形用 PR の自動提案**: 同一リポジトリ内の作業ブランチから未整形の PR
    が出された場合、GitHub Actions
    が元のブランチを直接書き換えず、`automation/format-pr-<PR番号>`
    ブランチを作成して元の作業ブランチ向けの整形用 PR
    を1件のみ作成・更新します。PR 作成者がその整形用 PR
    を取り込むことでフォーマットチェックを通過させることができます。

### Rust 検証 CI（必須チェック）

- **独立したワークフロー**: Rust
  コードの検証は、ドキュメント整形系のワークフローとは独立した
  `.github/workflows/rust.yml`（ワークフロー名
  `Rust`）で実行します。読み取り専用権限（`contents: read`）で動作し、fork
  からの PR でも同一の検証が走ります。
- **ジョブと責務**:
  - `cargo test`: `cargo test --workspace --all-targets` と `cargo test --doc`
    を実行し、単体・統合テストと doctest を検証します。
  - `cargo clippy`: `cargo clippy --workspace --all-targets -- -D warnings`
    を実行し、lint 警告を1件も許容しません。
  - `cargo fmt --check`: `cargo fmt --all -- --check` を実行し、Rust
    ソースの整形を検証します。
- **コミット前のローカル検証**: Rust
  コードを変更した場合は、以下の4コマンドがすべて成功することをコミット前に確認してください。

  ```bash
  cargo test --workspace --all-targets
  cargo test --doc
  cargo clippy --workspace --all-targets -- -D warnings
  cargo fmt --all -- --check
  ```

  `cargo clippy` の警告は `allow`
  属性で抑制せず、コード側を修正して0件にします。

### `main` ブランチのマージ前チェック設定（リポジトリ管理者向け）

ブランチ保護はワークフローファイルでは表現できないため、GitHub
側で以下を設定します（`Settings > Rules > Rulesets`、または
`Settings > Branches` のブランチ保護）。

1. 対象ブランチを `main` にしたルールセットを作成します。
2. **Require a pull request before merging** を有効にします。
3. **Require status checks to pass before merging**
   を有効にし、必須チェックとして次の4件を登録します。
   - `cargo test`
   - `cargo clippy`
   - `cargo fmt --check`
   - `deno fmt --check`

必須ステータスチェックは**ジョブ名**で登録されるため、登録済みのジョブ名（`cargo test`
/ `cargo clippy` / `cargo fmt --check` /
`deno fmt --check`）は変更しないでください。やむを得ず変更する場合は、ルールセットの登録名も同時に更新します（更新しないと旧名のチェックが永久に未完了となり、すべての
PR がマージ不能になります）。

また、Rust 検証ワークフローには `paths`
フィルタを設定しません。パスで起動を絞ると、ドキュメントのみを変更した PR
で必須チェックが未実行のまま保留され、マージがブロックされるためです。
