# Design

## Context

動機は proposal.md の「Why」、構文および解析結果の要件は
`specs/journal-entry-dsl/spec.md` を参照する。

現状のリポジトリは単一クレート `periodless`（edition
2024、依存ゼロ、`src/lib.rs` はテンプレートのみ）。AGENTS.md
はドメインコアに特定のフレームワーク依存を持ち込まないこと（コアの純粋性）、プロダクションコードでのパニック禁止、公開APIのフルーエント設計、`//`
行コメントの原則禁止と `///` / doctest
の充実を要求する。構文解析は入力アダプタ層の関心事であり、PEGパーサージェネレータへの依存をコアに持ち込まない構成が必要になる。

## Goals / Non-Goals

**Goals:**

- 仕様の構文をPEG文法ファイルとして一箇所に表現し、仕様と文法の対応を追跡可能にする。
- 「空白区切りの最後の部分を金額表現とする」境界決定を、Rust側の後処理ではなく文法で表現する。
- pest への依存をDSLクレートに閉じ込め、公開型・公開エラー型に pest
  の型を漏らさない。

**Non-Goals:**

- 金額表現から整数値への変換API（本変更では提供しない。仕様上は構文解析と分離された責務であり、後続変更で
  `JournalEntry` 構築APIと合わせて導入する）。
- 構文エラーメッセージの多言語化および診断表示の整形。
- 解析結果から `JournalEntry` を構築する経路。

## Decisions

### 1. リポジトリをCargoワークスペース化し、DSLを別クレートにする

ルート `Cargo.toml` を `[workspace]` とし、既存ライブラリを
`crates/periodless-core`、新規DSLを `crates/periodless-journal-dsl`
に配置する。`pest` / `pest_derive` の依存は後者のみに追加し、コアは依存しない。

検討した代替案:

- 単一クレートに `dsl` feature
  を追加してモジュール分割する案。構成変更は小さいが、feature
  の有効化によってコアクレート自体がパーサージェネレータに依存し得るため、コアの純粋性が構成上保証されない。
- 単一クレートで無条件に依存する案。最も単純だがコアがpestに依存するため却下。

### 2. 境界決定を否定先読みで文法に表現する

pest（PEG）の繰り返しは貪欲で、繰り返し内部では後戻りしない。そのため「科目名は内部空白を許可する」「最後の空白区切り部分が金額表現」という二つの要件を素直に並べると、科目名が金額表現まで飲み込んで失敗する。繰り返しの各反復に「次のチャンクが明細末尾の金額表現でないこと」の否定先読みを置くことで、最後のチャンクを金額表現用に残す。

`crates/periodless-journal-dsl/src/journal_entry.pest` の骨格:

```pest
journal_entry = { (detail_line | blank_line)+ ~ EOI }
detail_line   = { hspace* ~ (both_sides | debit_only | credit_only) ~ hspace* ~ newline }
both_sides    = { debit ~ hspace* ~ "/" ~ hspace* ~ credit }
debit_only    = { debit ~ hspace* ~ "/" }
credit_only   = { "/" ~ hspace* ~ credit }
blank_line    = { hspace* ~ newline }

debit  = { detail }
credit = { detail }

detail       = ${ account_name ~ hspace+ ~ amount ~ &detail_end }
account_name = @{ chunk ~ (hspace+ ~ !(amount ~ &detail_end) ~ chunk)* }
chunk        = @{ (!("/" | hspace | newline) ~ ANY)+ }
detail_end   = { hspace* ~ ("/" | newline | EOI) }

amount       = @{ ASCII_DIGIT ~ (separator* ~ ASCII_DIGIT)* }
separator    = { !ASCII_DIGIT ~ amount_ascii }
amount_ascii = { !("/" | WHITE_SPACE | NEWLINE) ~ '\x00'..'\x7f' }

hspace  = { !newline ~ WHITE_SPACE }
newline = { "\r\n" | "\n" | "\u{000b}" | "\u{000c}" | "\r" | "\u{0085}" | "\u{2028}" | "\u{2029}" }
```

この形が仕様を満たす理由:

- `科目 1 000` は、最終チャンク `000` が金額表現かつ明細末尾なので
  `account_name` の反復がその手前で停止し、`科目 1` / `000` に分かれる。
- `現金 100 円` は最終チャンク `円` が金額表現でないため反復が `円`
  まで消費し、続く `hspace+ ~ amount`
  が失敗して明細全体が構文違反になる（別境界の再解釈をしない）。
- `1,000` 単体は `account_name` の先頭チャンクとして無条件に消費され、続く
  `hspace+` が失敗するため、金額表現のみの明細は拒否される。
- `separator` から ASCII 数字を除外するのが要点。除外しないと `inner*`
  が末尾の非数字まで貪欲に消費して反復が巻き戻り、`1,,000`
  のような表記を取りこぼす。
- `hspace` を「改行以外の White_Space」、`newline` を CRLF
  優先の順序選択で定義することで、水平空白と改行の仕様（CRLF
  は一改行）をそのまま表現する。`journal_entry` が `(detail_line | blank_line)+`
  と各行の `newline`
  を要求するため、末尾改行必須と「明細一件以上」は文法で担保される（明細ゼロ件の入力は
  `blank_line`
  のみとなるため、解析後に明細列が空であることをもって構文違反として扱う）。

検討した代替案:
行を左右に分割したうえでRust側で最終チャンクを切り出す二段方式。実装は平易だが、境界決定が文法に現れず仕様との対応が追えなくなるため却下。

### 3. 公開APIは解析エントリポイントと参照返却の読み取りAPIに限定する

- `JournalEntryDescription::parse(&str) -> Result<JournalEntryDescription, SyntaxError>`
  を唯一のエントリポイントとする。
- 明細は `DescribedDetail` とし、`account_name() -> &str` と
  `amount_expression() -> &str` で元の文字列を返す。正規化・集約は行わない。
- 借方・貸方は `debits()` / `credits()`
  のイテレータで出現順に返し、メソッドチェーンで絞り込み・集計ができる形にする。
- `SyntaxError` は行・列および違反の種別を保持する独自型とし、pest の
  `Error<Rule>` を内部に閉じ込める。`unwrap` / `expect` / `panic!`
  を使わず、文法由来の失敗はすべて `Err` に変換する。
- `Item` / `Data` / `Record` 等の汎用名は使用しない（CONTRIBUTING.md
  の対訳に従い `Debit` / `Credit` / `Account` 系の語を用いる）。

### 4. 所有形態は入力文字列からの切り出しを `String` として保持する

解析結果は入力のライフタイムに縛られない独立した値とし、科目名と金額表現を
`String`
で保持する。借用版（`&'a str`）は利用者側のライフタイム制約が強くなるため採用しない。

## Risks / Trade-offs

- 否定先読みを含む文法は読み手にとって非自明 → 境界決定のテスト（`科目 1 000`,
  `現金 100 円`, `1,000`
  単体）を仕様のシナリオと1対1で対応させ、文法変更時に退行を検出できるようにする。`//`
  コメントは使わず、意図は規則名とテスト名で示す。
- `amount_ascii`
  が空白・改行以外のASCII制御文字を受理するため、不可視文字を含む金額表現が通る
  →
  仕様上の意図した挙動であり、可視性の問題は意味検証・入力UI側の責務として扱う。
- ワークスペース分割によりパスとインポートが変わる → 既存の公開APIは `add`
  のみで利用者がいないため影響は限定的。分割はタスクの最初に独立して行い、`cargo test`
  が通る状態を維持する。
- pest の繰り返し・先読みの挙動は文法の細部に敏感 →
  失敗するテストを先に用意し（Agentic TDD）、文法は最小から育てる。
