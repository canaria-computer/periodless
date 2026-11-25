# Spec Delta

## MODIFIED Requirements

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
