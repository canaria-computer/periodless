# Tasks

## 1. ワークスペース分割

- [x] 1.1 ルート `Cargo.toml` を `[workspace]` 化し、既存 `src/lib.rs` を
      `crates/periodless-core/src/lib.rs` へ移動して、`cargo test`
      がワークスペース全体で成功することを確認する
- [x] 1.2 `crates/periodless-journal-dsl` クレートを作成し、`pest` /
      `pest_derive`
      を当該クレートのみの依存として追加して、`cargo tree -p periodless-core` に
      pest が現れないことを確認する

## 2. 行構造とレイアウトの構文（Red → Green）

- [x] 2.1
      行構造の失敗するテストを日本語テスト関数名で追加する（テストモジュールに
      `#[allow(non_ascii_idents)]` を付与）: `借方 / 貸方`・`借方 /`・`/ 貸方`
      の三形の受理、スラッシュのみの行の拒否、空白のみの行の無視、省略側への引き継ぎが起きないこと、CRLF
      の受理、末尾改行欠落の拒否、明細ゼロ件入力の拒否。`cargo test`
      が失敗することを確認する
- [x] 2.2 `journal_entry.pest` に `journal_entry` / `detail_line` / `blank_line`
      / `both_sides` / `debit_only` / `credit_only` / `hspace` / `newline`
      を実装し、2.1 のテストが成功することを確認する

## 3. 明細の境界決定と金額表現の構文（Red → Green）

- [x] 3.1 境界決定の失敗するテストを追加する: `科目 1 000` が科目名 `科目 1` /
      金額表現 `000` に分かれること、`現金 100 円` の拒否、`1,000`
      単体（科目名欠落）の拒否、科目名のみの拒否、科目名内部の空白が保持されること。`cargo test`
      が失敗することを確認する
- [x] 3.2 金額表現の失敗するテストを追加する: `0` / `000` / `1000` / `1,000` /
      `1.000` / `1;000` / `1,,000` / `1,0;00` / `1e3` の受理、`,000` と `1,`
      の拒否、空白・改行以外のASCII制御文字を桁区切りとして受理すること、桁数上限がないこと。`cargo test`
      が失敗することを確認する
- [x] 3.3 `detail` / `account_name` / `chunk` / `detail_end` / `amount` /
      `separator` / `amount_ascii` を design.md の否定先読み方式で実装し、3.1 と
      3.2 のテストが成功することを確認する

## 4. 解析結果の公開APIと構文エラー型

- [x] 4.1 解析結果の失敗するテストを追加する: 仕様の `給料 400000 / ...`
      例が借方1件・貸方3件に出現順で分解されること、同一科目の複数出現が集約されずに保持されること、金額表現が元の文字列のまま保持されること、科目名が正規化されないこと。`cargo test`
      が失敗することを確認する
- [x] 4.2 `JournalEntryDescription::parse` / `DescribedDetail` / `debits()` /
      `credits()` を実装し、4.1 のテストが成功することを確認する
- [x] 4.3 `SyntaxError` を行・列と違反種別を持つ独自型として実装し、pest の
      `Error<Rule>` が公開シグネチャに現れないこと、および
      `grep -rn "unwrap()\|expect(\|panic!\|todo!\|unimplemented!" crates/*/src`
      が（テストを除き）無結果であることを確認する
- [x] 4.4 公開API（`parse`, `debits`, `credits`, `account_name`,
      `amount_expression`, `SyntaxError`）に `///` と doctest
      を付与し、`cargo test --doc` が成功することを確認する
- [x] 4.5 `//` 行コメントを残していないことを
      `grep -rn "^\s*//[^/]" crates/*/src`
      で確認し、意図が規則名・型名・テスト名で表現されていることを確認する

## 5. 統合確認

- [x] 5.1
      `cargo test`、`cargo test --doc`、`cargo clippy --all-targets -- -D warnings`
      がワークスペース全体で成功することを確認する
- [x] 5.2 `deno fmt` を Markdown / YAML / JSON に適用し、`deno fmt --check`
      が成功することを確認する
- [x] 5.3 `openspec validate --change add-journal-entry-dsl --strict`
      が成功し、仕様の各シナリオに対応するテストが存在することを確認する
