# Tasks

## 1. 金額表現から整数値への変換

- [x] 1.1 変換の失敗するテストを追加する（日本語テスト関数名、
      `#![allow(non_ascii_idents, non_snake_case)]`）: `000`→0、`1,000`→1000、
      `1.000`/`1;000`→1000、`1e3`→13、`i64::MAX`
      の受理、40桁の金額表現は範囲超過エラー、構文解析自体は成功すること。`cargo test`
      が失敗することを確認する
- [x] 1.2 `DescribedDetail::amount_as_integer() -> Result<i64, AmountOverflow>`
      と `AmountOverflow`（`Display` と `core::error::Error`
      を実装）を実装し、1.1 のテストが成功することを確認する
- [x] 1.3 変換APIに `///` と doctest
      を付与し（返す整数が単位を持たないこと、単位付き型への詰め替えは利用側の責務であることを明記）、`cargo test --doc`
      が成功することを確認する

## 2. 整形出力の行組み立て

- [x] 2.1 行組み立ての失敗するテストを追加する:
      借方と貸方が同数の仕訳、貸方が多い仕訳（借方側を空ける）、借方のみの仕訳（`借方明細 /`）、貸方のみの仕訳、行末に水平空白がないこと、出力が末尾改行で終わること。`cargo test`
      が失敗することを確認する
- [x] 2.2 `printer.rs` に `JournalEntryPrinter::new().print(&description)`
      を実装し、2.1 のテストが成功することを確認する

## 3. 表示幅による桁揃え

- [x] 3.1 桁揃えの失敗するテストを追加する:
      全角と半角が混在する科目名でスラッシュ位置が各行で一致すること、桁数の異なる金額の右端が揃うこと。`cargo test`
      が失敗することを確認する
- [x] 3.2 `unicode-width` を `periodless-journal-dsl` のみに追加し
      `UnicodeWidthStr::width` による桁揃えを実装して、3.1
      のテストが成功することを確認する。`cargo tree -p periodless-core`
      に当該依存が現れないことを確認する

## 4. 金額表現の正規化

- [x] 4.1 正規化の失敗するテストを追加する: `1,0;00`→`1,000`、`1e3`→`13`、
      `000`→`0`、`i64` に収まらない40桁の金額表現も整形できること。`cargo test`
      が失敗することを確認する
- [x] 4.2 ASCII数字の抽出・先頭ゼロ除去・右から三桁ごとのカンマ挿入を実装し、4.1
      のテストが成功することを確認する

## 5. ラウンドトリップと公開API

- [x] 5.1 ラウンドトリップの失敗するテストを追加する:
      整形出力が再解析でき科目名が一致すること、`print → parse → print`
      が一致すること（冪等）、`amount_as_integer()`
      の結果が保たれること、科目名内部の水平空白が復元されること、`Display`
      経由で同じ出力が得られること。`cargo test` が失敗することを確認する
- [x] 5.2 `impl Display for JournalEntryDescription` を実装し、5.1
      のテストが成功することを確認する
- [x] 5.3 整形APIに `///` と doctest
      を付与し（同じ行に並ぶことが対応関係を意味しないこと、金額が正規化されることを明記）、`cargo test --doc`
      が成功することを確認する

## 6. 統合確認

- [x] 6.1
      仕様の入力例（給料の支払）を整形した結果を期待値として固定する統合テストを追加し、`cargo test`
      が成功することを確認する
- [x] 6.2 `cargo test`、`cargo test --doc`、
      `cargo clippy --all-targets -- -D warnings`、`cargo fmt --all --check`
      がワークスペース全体で成功することを確認する
- [x] 6.3 `src` 配下に `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`、
      浮動小数点、`//` 行コメントが無いことを grep で確認し、
      `cargo tree -p periodless-journal-dsl` に `periodless-core`
      が現れないことを確認する
- [x] 6.4 `deno fmt` を適用し `deno fmt --check` と
      `openspec validate add-journal-entry-printer --strict`
      が成功することを確認する
