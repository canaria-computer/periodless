use crate::journal_entry::{JournalEntry, ValidatedJournalEntry};
use crate::record_hash::{RecordHash, RecordHashParts, record_hash_of};
use core::fmt::{Display, Formatter, Result as FormatResult};

/// 台帳への追記に違反があったことを表すエラー。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerError {
    /// 追記する仕訳のコミット日時が、直近に記録された仕訳のコミット日時より前である。
    CommitTimeRegression,
    /// 保持するレコード列のハッシュチェーンが切れている、または内容が改変されている。
    BrokenChain { index: usize },
    /// 保持するレコードの明細列が正規化順でない（順序が改変されている）。
    LinesNotNormalized { index: usize },
}

impl Display for LedgerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self {
            LedgerError::CommitTimeRegression => write!(
                formatter,
                "コミット日時が直近に記録された仕訳より前であるため追記できません"
            ),
            LedgerError::BrokenChain { index } => write!(
                formatter,
                "{}件目のレコードでハッシュチェーンが壊れています（改変の可能性があります）",
                index + 1
            ),
            LedgerError::LinesNotNormalized { index } => write!(
                formatter,
                "{}件目のレコードの明細列が正規化順ではありません（改変の可能性があります）",
                index + 1
            ),
        }
    }
}

impl core::error::Error for LedgerError {}

/// 台帳へ追記済みの仕訳レコード。
///
/// 検証済みの [`JournalEntry`]、前レコードのハッシュ値、および両者から計算
/// したレコードハッシュを持つ。構築は [`Ledger::append`] 経由のみであり、
/// ハッシュ値を直接設定する手段は公開しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntryRecord {
    journal_entry: JournalEntry,
    previous_record_hash: RecordHash,
    record_hash: RecordHash,
}

impl JournalEntryRecord {
    /// 追記済みの仕訳。
    pub fn journal_entry(&self) -> &JournalEntry {
        &self.journal_entry
    }

    /// 前レコードのハッシュ値。最初のレコードでは32バイトのゼロ（[`RecordHash::GENESIS`]）。
    pub fn previous_record_hash(&self) -> &RecordHash {
        &self.previous_record_hash
    }

    /// このレコードのハッシュ値。
    pub fn record_hash(&self) -> &RecordHash {
        &self.record_hash
    }
}

/// 追記専用の仕訳台帳。
///
/// インメモリの追記専用コレクションであり、記帳済みの仕訳を変更または削除する
/// 操作は提供しない。追記時にコミット日時の単調性（直近のレコードと等しいか
/// 増加であること）を検証し、前レコードハッシュを設定してレコードハッシュを
/// 計算する。[`Ledger::verify`] は保持する全レコードのハッシュを再計算し、
/// 連鎖と正規化順を確認して改変を検出する。
///
/// # Examples
///
/// ```
/// use periodless_core::{
///     AccountId, Amount, CommittedAt, JournalEntry, Ledger, TransactionAt,
/// };
///
/// let 現金 = AccountId::new(0x1000_FFFF_0001_0000)?;
/// let 売上 = AccountId::new(0x5000_0000_0001_0000)?;
///
/// let 仕訳 = JournalEntry::builder()
///     .transacted_at(TransactionAt::parse_rfc3339("2026-11-13T02:19:00Z")?)
///     .committed_at(CommittedAt::parse_rfc3339("2026-11-13T02:20:00Z")?)
///     .debit(現金, Amount::from_yen(3_500)?)
///     .credit(売上, Amount::from_yen(3_500)?)
///     .build()?;
///
/// let mut ledger = Ledger::new();
/// let record = ledger.append(仕訳)?;
///
/// assert_eq!(ledger.records().len(), 1);
/// assert_eq!(record.previous_record_hash(), &periodless_core::RecordHash::GENESIS);
/// assert!(ledger.verify().is_ok());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    records: Vec<JournalEntryRecord>,
}

impl Ledger {
    /// 空の台帳を返す。
    pub fn new() -> Self {
        Self::default()
    }

    /// 検証済みの仕訳を追記する。
    ///
    /// コミット日時が直近に記録された仕訳のコミット日時より前である場合は
    /// 追記を拒否する。等しい場合は受理し、順序は到達順序による。
    ///
    /// # Errors
    ///
    /// コミット日時が直近のレコードより前である場合に
    /// [`LedgerError::CommitTimeRegression`] を返す。
    pub fn append(
        &mut self,
        validated: ValidatedJournalEntry,
    ) -> Result<JournalEntryRecord, LedgerError> {
        let journal_entry = validated.into_journal_entry();

        if let Some(latest) = self.records.last()
            && journal_entry.committed_at() < latest.journal_entry().committed_at()
        {
            return Err(LedgerError::CommitTimeRegression);
        }

        let previous_record_hash = self
            .records
            .last()
            .map(|latest| *latest.record_hash())
            .unwrap_or(RecordHash::GENESIS);

        let record_hash = record_hash_of(&RecordHashParts {
            transaction_at: journal_entry.transaction_at(),
            committed_at: journal_entry.committed_at(),
            debits: journal_entry.debits(),
            credits: journal_entry.credits(),
            memo: journal_entry.memo(),
            previous_record_hash: &previous_record_hash,
        });

        let record = JournalEntryRecord {
            journal_entry,
            previous_record_hash,
            record_hash,
        };

        self.records.push(record.clone());

        Ok(record)
    }

    /// 保持する全レコードのハッシュを再計算し、連鎖と正規化順を確認する。
    ///
    /// # Errors
    ///
    /// 前レコードハッシュが連鎖と一致しない、または再計算したレコードハッシュが
    /// 保持しているハッシュと一致しない場合に [`LedgerError::BrokenChain`]、
    /// 明細列が正規化順でない場合に [`LedgerError::LinesNotNormalized`] を返す。
    pub fn verify(&self) -> Result<(), LedgerError> {
        let mut previous_record_hash = RecordHash::GENESIS;

        for (index, record) in self.records.iter().enumerate() {
            if record.previous_record_hash != previous_record_hash {
                return Err(LedgerError::BrokenChain { index });
            }

            if !record.journal_entry.has_normalized_lines() {
                return Err(LedgerError::LinesNotNormalized { index });
            }

            let recomputed = record_hash_of(&RecordHashParts {
                transaction_at: record.journal_entry.transaction_at(),
                committed_at: record.journal_entry.committed_at(),
                debits: record.journal_entry.debits(),
                credits: record.journal_entry.credits(),
                memo: record.journal_entry.memo(),
                previous_record_hash: &record.previous_record_hash,
            });

            if recomputed != record.record_hash {
                return Err(LedgerError::BrokenChain { index });
            }

            previous_record_hash = record.record_hash;
        }

        Ok(())
    }

    /// 保持する全レコード（記帳順）。
    pub fn records(&self) -> &[JournalEntryRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id::AccountId;
    use crate::amount::Amount;
    use crate::{CommittedAt, TransactionAt};

    const 現金: u64 = 0x1000_FFFF_0001_0000;
    const 売上: u64 = 0x5000_0000_0001_0000;
    const 雑収入: u64 = 0x5000_0000_0002_0000;

    fn 仕訳(
        transaction_at: &str,
        committed_at: &str,
        debits: Vec<(u64, i64)>,
        credits: Vec<(u64, i64)>,
    ) -> Result<ValidatedJournalEntry, Box<dyn std::error::Error>> {
        let mut builder = JournalEntry::builder()
            .transacted_at(TransactionAt::parse_rfc3339(transaction_at)?)
            .committed_at(CommittedAt::parse_rfc3339(committed_at)?);

        for (account, yen) in debits {
            builder = builder.debit(AccountId::new(account)?, Amount::from_yen(yen)?);
        }

        for (account, yen) in credits {
            builder = builder.credit(AccountId::new(account)?, Amount::from_yen(yen)?);
        }

        Ok(builder.build()?)
    }

    #[test]
    fn 最初のレコードの前レコードハッシュはジェネシスである()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();
        let record = ledger.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T02:20:00Z",
            vec![(現金, 3_500)],
            vec![(売上, 3_500)],
        )?)?;

        assert_eq!(record.previous_record_hash(), &RecordHash::GENESIS);
        assert!(ledger.verify().is_ok());

        Ok(())
    }

    #[test]
    fn 二件目以降の前レコードハッシュは直近のレコードハッシュである()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();

        let 一件目 = ledger.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T02:20:00Z",
            vec![(現金, 3_500)],
            vec![(売上, 3_500)],
        )?)?;
        let 二件目 = ledger.append(仕訳(
            "2026-11-13T03:19:00Z",
            "2026-11-13T03:20:00Z",
            vec![(現金, 1_000)],
            vec![(雑収入, 1_000)],
        )?)?;

        assert_eq!(二件目.previous_record_hash(), 一件目.record_hash());
        assert!(ledger.verify().is_ok());

        Ok(())
    }

    #[test]
    fn 過去のコミット日時を持つ仕訳は拒否される() -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();

        ledger.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T05:00:00Z",
            vec![(現金, 3_500)],
            vec![(売上, 3_500)],
        )?)?;

        let 検証結果 = ledger.append(仕訳(
            "2026-11-13T06:19:00Z",
            "2026-11-13T04:59:59.999Z",
            vec![(現金, 1_000)],
            vec![(売上, 1_000)],
        )?);

        assert_eq!(
            検証結果.expect_err("過去のコミット日時は拒否される"),
            LedgerError::CommitTimeRegression
        );
        assert_eq!(ledger.records().len(), 1);

        Ok(())
    }

    #[test]
    fn 等しいコミット日時の仕訳は受理される() -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();

        ledger.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T05:00:00Z",
            vec![(現金, 3_500)],
            vec![(売上, 3_500)],
        )?)?;
        ledger.append(仕訳(
            "2026-11-13T02:25:00Z",
            "2026-11-13T05:00:00Z",
            vec![(現金, 1_000)],
            vec![(売上, 1_000)],
        )?)?;

        assert_eq!(ledger.records().len(), 2);
        assert!(ledger.verify().is_ok());

        Ok(())
    }

    #[test]
    fn 過去の取引日時を持つ仕訳は受理される() -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();

        ledger.append(仕訳(
            "2026-11-13T10:00:00Z",
            "2026-11-13T11:00:00Z",
            vec![(現金, 3_500)],
            vec![(売上, 3_500)],
        )?)?;
        ledger.append(仕訳(
            "2001-01-01T00:00:00Z",
            "2026-11-13T12:00:00Z",
            vec![(現金, 1_000)],
            vec![(売上, 1_000)],
        )?)?;

        assert_eq!(ledger.records().len(), 2);

        Ok(())
    }

    #[test]
    fn 明細順序が改変された場合は検証が失敗する() -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();

        ledger.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T02:20:00Z",
            vec![(現金, 1_000), (0x1000_4000_0017_0000, 9_000)],
            vec![(売上, 5_000), (雑収入, 5_000)],
        )?)?;

        if let Some(record) = ledger.records.last_mut() {
            record.journal_entry.swap_lines_for_test();
        }

        assert_eq!(
            ledger.verify(),
            Err(LedgerError::LinesNotNormalized { index: 0 })
        );

        Ok(())
    }

    #[test]
    fn レコードの内容が改変された場合は検証が失敗する() -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();

        ledger.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T02:20:00Z",
            vec![(現金, 3_500)],
            vec![(売上, 3_500)],
        )?)?;
        ledger.append(仕訳(
            "2026-11-13T03:19:00Z",
            "2026-11-13T03:20:00Z",
            vec![(現金, 1_000)],
            vec![(雑収入, 1_000)],
        )?)?;

        if let Some(record) = ledger.records.first_mut() {
            record.journal_entry.replace_first_debit_for_test();
        }

        assert_eq!(ledger.verify(), Err(LedgerError::BrokenChain { index: 0 }));

        Ok(())
    }

    #[test]
    fn 側内の順序のみが異なる仕訳は同一のレコードハッシュになる()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut ledger = Ledger::new();

        let 一件目 = ledger.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T02:20:00Z",
            vec![(現金, 1_000), (0x1000_4000_0017_0000, 9_000)],
            vec![(売上, 5_000), (雑収入, 5_000)],
        )?)?;

        let mut 別の台帳 = Ledger::new();
        let 別の記述の一件目 = 別の台帳.append(仕訳(
            "2026-11-13T02:19:00Z",
            "2026-11-13T02:20:00Z",
            vec![(0x1000_4000_0017_0000, 9_000), (現金, 1_000)],
            vec![(雑収入, 5_000), (売上, 5_000)],
        )?)?;

        assert_eq!(一件目.record_hash(), 別の記述の一件目.record_hash());

        Ok(())
    }
}
