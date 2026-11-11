//! 簿記の仕訳を記述するためのテキストDSLの構文解析器。
//!
//! 一回の解析に渡された入力全体を一つの仕訳として扱い、`借方明細 / 貸方明細`
//! 形式の明細行を借方明細列と貸方明細列へ分解する。構文のみを扱い、貸借一致・
//! 科目の重複・科目マスターへの登録状況・金額の値域といった意味検証は行わない。
//!
//! # Examples
//!
//! ```
//! use periodless_journal_dsl::JournalEntryDescription;
//!
//! let description = JournalEntryDescription::parse(
//!     "給料 400000 / 所得税預り金 30000\n   /現金 370000\n",
//! )?;
//!
//! assert_eq!(description.debits().count(), 1);
//! assert_eq!(description.credits().count(), 2);
//! # Ok::<(), periodless_journal_dsl::SyntaxError>(())
//! ```

mod amount_overflow;
mod description;
mod grammar;
mod syntax_error;

pub use amount_overflow::AmountOverflow;
pub use description::{DescribedDetail, JournalEntryDescription};
pub use syntax_error::{SyntaxError, SyntaxViolation};
