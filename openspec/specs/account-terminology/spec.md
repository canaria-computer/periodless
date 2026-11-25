# account-terminology Specification

## Purpose

日商簿記3級の範囲における勘定科目および会計ドメイン概念について、日本語用語とRustコード上の英語識別子（型・バリアント名および変数・メソッド名）の1対1の標準対訳を提供し、LLMと開発者が揺れのない命名を行えるようにする。

## Requirements

### Requirement: 勘定科目一覧のLLM参照用日英対訳構造化

`docs/勘定科目一覧/` 配下の各Markdownファイル（`資産.md`, `負債.md`,
`純資産.md`, `収益.md`, `費用.md`, `その他・評価勘定.md`）および `accounts.yaml`
は、収録されているすべての勘定科目について、64bit勘定科目ID・日本語科目名・英語の型/列挙子識別子（`UpperCamelCase`）・英語の変数/メソッド識別子（`snake_case`）・会計上の区分や流動性スコアを構造化されたテーブルおよびYAMLとして定義しなければならない（SHALL）。その際、「法人税、住民税及び事業税（法人税等）」および「雑損」は費用（`0100`）、「雑益」は収益（`0101`）、「貸倒引当金」「減価償却累計額」「現金過不足」「損益」は評価・特殊勘定（`0110`）として整理しなければならない（SHALL）。

#### Scenario: 勘定科目を扱うコードの実装・テスト作成

- **WHEN**
  開発者またはLLMが勘定科目を表す型・列挙子・変数・メソッド・勘定科目IDを記述する場合
- **THEN** `docs/勘定科目一覧/*.md` および `docs/勘定科目一覧/accounts.yaml`
  に定義された英語識別子と64bit勘定科目IDをそのまま使用し、独自の別訳や汎用名（`Item`,
  `Data`, `Record` 等）を作らない

### Requirement: 会計基本概念のユビキタス言語定義

勘定科目だけでなく、仕訳、伝票、借方、貸方、仕訳帳、損益計算書、貸借対照表、決算整理、取引日時、コミット日時、適用通信欄、台帳、前レコードハッシュ、レコードハッシュなどの基本ドメイン概念についても、日本語と英語識別子の標準対訳をドキュメント内で明示しなければならない（SHALL）。

#### Scenario: ドメインモデルの型・API命名

- **WHEN** 複式簿記のドメインモデルや公開APIを設計・実装する場合
- **THEN** 標準対訳で定義された英語ドメイン用語（`JournalEntry`, `Debit`,
  `Credit`, `Slip`, `Account`, `GeneralJournal`, `IncomeStatement`,
  `BalanceSheet`, `TransactionAt`, `CommittedAt`, `Memo`, `Ledger`, `RecordHash`
  等）を一貫して使用する

#### Scenario: 仕訳記録機能の用語の使用

- **WHEN** 仕訳記録機能の型・変数・メソッド・ドキュメントを記述する場合
- **THEN** 取引日時＝`TransactionAt` /
  `transaction_at`、コミット日時＝`CommittedAt` /
  `committed_at`、適用通信欄＝`Memo` / `memo`、台帳＝`Ledger` /
  `ledger`、前レコードハッシュ＝`previous_record_hash`、レコードハッシュ＝`RecordHash`
  / `record_hash` の標準対訳を使用する
