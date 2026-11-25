#![cfg_attr(test, allow(non_ascii_idents, non_snake_case))]

//! 紙の複式簿記をデジタル化する会計コアライブラリ。
//!
//! 仕訳を不変なレコードとして台帳へ追記記録し、BLAKE3によるハッシュチェーンで
//! 履歴の完全性を検証する。システムは時計を持たず、取引日時・コミット日時は
//! すべて入力として与えられる。

mod account_id;
mod amount;
mod datetime;
mod journal_entry;
mod ledger;
mod memo;
mod record_hash;
mod validation;

pub use account_id::{AccountId, AccountIdError};
pub use amount::{Amount, AmountError};
pub use datetime::{CommittedAt, NormalizationError, TransactionAt};
pub use journal_entry::{JournalEntry, JournalEntryBuilder, JournalLine, ValidatedJournalEntry};
pub use ledger::{JournalEntryRecord, Ledger, LedgerError};
pub use memo::{Memo, MemoError};
pub use record_hash::RecordHash;
pub use validation::{ValidationError, ValidationErrors, ValidationWarning};
