use crate::account_id::AccountId;
use crate::amount::Amount;
use crate::memo::Memo;
use crate::validation::ValidationError;
use crate::{CommittedAt, TransactionAt};

/// 仕訳行。借方・貸方の各明細を表す勘定科目IDと金額の組。
///
/// 仕訳は借方明細列と貸方明細列を持ち、各側の明細列は正規化順（金額の降順、
/// 次いで勘定科目IDの降順）で保持される。同じ行の借方と貸方に個別の対応関係
/// は持たない。
///
/// # Examples
///
/// ```
/// use periodless_core::{AccountId, Amount, JournalLine};
///
/// let 現金入金 = JournalLine::new(
///     AccountId::new(0x1000_FFFF_0001_0000)?,
///     Amount::from_yen(3_500)?,
/// );
///
/// assert_eq!(現金入金.amount().as_sen(), 350_000);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JournalLine {
    account_id: AccountId,
    amount: Amount,
}

impl JournalLine {
    /// 勘定科目IDと金額から仕訳行を構築する。
    pub fn new(account_id: AccountId, amount: Amount) -> Self {
        Self { account_id, amount }
    }

    /// 勘定科目ID。
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// 金額。
    pub fn amount(&self) -> Amount {
        self.amount
    }
}

/// 明細列を正規化順（金額の降順、次いで勘定科目IDの降順）で並べ替える。
fn normalize_lines(lines: &mut [JournalLine]) {
    lines.sort_by(|left, right| {
        right
            .amount()
            .cmp(&left.amount())
            .then_with(|| right.account_id().cmp(&left.account_id()))
    });
}

/// 正規化された明細列が正規化順であるかどうか。
fn is_normalized(lines: &[JournalLine]) -> bool {
    lines.windows(2).all(|pair| {
        let left = &pair[0];
        let right = &pair[1];

        left.amount() > right.amount()
            || (left.amount() == right.amount() && left.account_id() >= right.account_id())
    })
}

/// 検証済みの仕訳。
///
/// [`JournalEntry::builder`] から構築し、`build()` が検証（貸借一致・明細の
/// 妥当性・検証ルール2/3/4/6/7）と明細列の正規化を行う。明細列は正規化順
/// （金額の降順、次いで勘定科目IDの降順）で保持し、等価性（`PartialEq` /
/// `Eq`）とハッシュ（`Hash`）は保持順そのものに基づく。構築後は変更も削除も
/// できない。
///
/// # Examples
///
/// ```
/// use periodless_core::{
///     AccountId, Amount, CommittedAt, JournalEntry, Memo, TransactionAt,
/// };
///
/// let 現金 = AccountId::new(0x1000_FFFF_0001_0000)?;
/// let 売上 = AccountId::new(0x5000_0000_0001_0000)?;
///
/// let 検証済み = JournalEntry::builder()
///     .transacted_at(TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123001Z")?)
///     .committed_at(CommittedAt::parse_rfc3339("2026-11-13T02:20:00Z")?)
///     .debit(現金, Amount::from_yen(3_500)?)
///     .credit(売上, Amount::from_yen(3_500)?)
///     .memo(Memo::new("月初の現金入金")?)
///     .build()?;
///
/// assert!(検証済み.warnings().is_empty());
/// assert_eq!(検証済み.journal_entry().debits().len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JournalEntry {
    transaction_at: TransactionAt,
    committed_at: CommittedAt,
    debits: Vec<JournalLine>,
    credits: Vec<JournalLine>,
    memo: Option<Memo>,
}

impl JournalEntry {
    /// フルーエントビルダーを返す。
    pub fn builder() -> JournalEntryBuilder {
        JournalEntryBuilder::default()
    }

    /// 取引日時。
    pub fn transaction_at(&self) -> &TransactionAt {
        &self.transaction_at
    }

    /// コミット日時。
    pub fn committed_at(&self) -> &CommittedAt {
        &self.committed_at
    }

    /// 借方明細列（正規化順）。
    pub fn debits(&self) -> &[JournalLine] {
        &self.debits
    }

    /// 貸方明細列（正規化順）。
    pub fn credits(&self) -> &[JournalLine] {
        &self.credits
    }

    /// 適用通信欄。未設定の場合は `None`。
    pub fn memo(&self) -> Option<&Memo> {
        self.memo.as_ref()
    }

    /// 明細列が正規化順であるかどうか（台帳の検証用）。
    pub(crate) fn has_normalized_lines(&self) -> bool {
        is_normalized(&self.debits) && is_normalized(&self.credits)
    }

    /// 借方明細列の先頭二件を入れ替える（正規化順を壊す改変の再現用）。
    #[cfg(test)]
    pub(crate) fn swap_lines_for_test(&mut self) {
        if self.debits.len() >= 2 {
            self.debits.swap(0, 1);
        }
    }

    /// 借方明細列の先頭の金額を変更する（内容の改変の再現用）。
    #[cfg(test)]
    pub(crate) fn replace_first_debit_for_test(&mut self) {
        if let Some(first) = self.debits.first_mut() {
            let 大きな金額 = Amount::new(i64::MAX / 4).expect("非負の金額である");
            *first = JournalLine::new(first.account_id(), 大きな金額);
        }
    }
}

/// [`JournalEntry`] のフルーエントビルダー。
///
/// `build()` は、貸借一致・明細の妥当性・検証ルール2（同一勘定科目の重複）・
/// ルール3（自己取引）・ルール4（合計ゼロ）・ルール6（合計の上限超過）を検証
/// して違反をすべて報告し、違反がなければ明細列を正規化順へ並べ替えた
/// 検証済みの仕訳と警告（ルール7: 合計が2^40円超）を返す。
#[derive(Debug, Clone, Default)]
pub struct JournalEntryBuilder {
    transaction_at: Option<TransactionAt>,
    committed_at: Option<CommittedAt>,
    debits: Vec<JournalLine>,
    credits: Vec<JournalLine>,
    memo: Option<Memo>,
}

impl JournalEntryBuilder {
    /// 取引日時を設定する。
    pub fn transacted_at(mut self, transaction_at: TransactionAt) -> Self {
        self.transaction_at = Some(transaction_at);
        self
    }

    /// コミット日時を設定する。
    pub fn committed_at(mut self, committed_at: CommittedAt) -> Self {
        self.committed_at = Some(committed_at);
        self
    }

    /// 借方明細を追加する。
    pub fn debit(mut self, account_id: AccountId, amount: Amount) -> Self {
        self.debits.push(JournalLine::new(account_id, amount));
        self
    }

    /// 貸方明細を追加する。
    pub fn credit(mut self, account_id: AccountId, amount: Amount) -> Self {
        self.credits.push(JournalLine::new(account_id, amount));
        self
    }

    /// 適用通信欄を設定する。
    pub fn memo(mut self, memo: Memo) -> Self {
        self.memo = Some(memo);
        self
    }

    /// すべての検証を行い、検証済みの仕訳を返す。
    ///
    /// # Errors
    ///
    /// 検出したすべての違反を [`crate::ValidationErrors`] として返す。
    pub fn build(self) -> Result<ValidatedJournalEntry, crate::ValidationErrors> {
        let mut errors = Vec::new();

        let transaction_at = self.transaction_at;
        let committed_at = self.committed_at;

        if transaction_at.is_none() {
            errors.push(ValidationError::MissingTransactionAt);
        }

        if committed_at.is_none() {
            errors.push(ValidationError::MissingCommittedAt);
        }

        let mut debits = self.debits;
        let mut credits = self.credits;
        normalize_lines(&mut debits);
        normalize_lines(&mut credits);

        collect_duplicate_accounts(&debits, Side::Debit, &mut errors);
        collect_duplicate_accounts(&credits, Side::Credit, &mut errors);

        let debit_total = sum_lines(&debits, &mut errors);
        let credit_total = sum_lines(&credits, &mut errors);

        if let (Some(debit_total), Some(credit_total)) = (debit_total, credit_total) {
            if debit_total != credit_total {
                errors.push(ValidationError::BalanceMismatch {
                    debit_total,
                    credit_total,
                });
            } else if debit_total == Amount::ZERO {
                errors.push(ValidationError::ZeroTotal);
            }
        }

        if debits == credits && !debits.is_empty() {
            errors.push(ValidationError::SelfTransaction);
        }

        if !errors.is_empty() {
            return Err(crate::ValidationErrors::new(errors));
        }

        let warnings = match debit_total {
            Some(total) if total.as_sen() > LARGE_TOTAL_FEN => {
                vec![crate::ValidationWarning::LargeTotal { total }]
            }
            _ => Vec::new(),
        };

        let journal_entry = JournalEntry {
            transaction_at: transaction_at.expect("違反がなければ取引日時は設定されている"),
            committed_at: committed_at.expect("違反がなければコミット日時は設定されている"),
            debits,
            credits,
            memo: self.memo,
        };

        Ok(ValidatedJournalEntry {
            journal_entry,
            warnings,
        })
    }
}

/// 検証済みの仕訳と、検出された警告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedJournalEntry {
    journal_entry: JournalEntry,
    warnings: Vec<crate::ValidationWarning>,
}

impl ValidatedJournalEntry {
    /// 検証済みの仕訳。
    pub fn journal_entry(&self) -> &JournalEntry {
        &self.journal_entry
    }

    /// 検証済みの仕訳を消費して取り出す。
    pub fn into_journal_entry(self) -> JournalEntry {
        self.journal_entry
    }

    /// 検出された警告（空の場合もある）。
    pub fn warnings(&self) -> &[crate::ValidationWarning] {
        &self.warnings
    }
}

/// 合計が2^40円を超える場合の警告の閾値（1銭単位）。2^40 × 100 銭。
const LARGE_TOTAL_FEN: i64 = 1_099_511_627_776 * 100;

enum Side {
    Debit,
    Credit,
}

fn collect_duplicate_accounts(
    lines: &[JournalLine],
    side: Side,
    errors: &mut Vec<ValidationError>,
) {
    for (index, line) in lines.iter().enumerate() {
        let duplicated = lines
            .iter()
            .skip(index + 1)
            .any(|other| other.account_id() == line.account_id());

        if duplicated {
            let error = match side {
                Side::Debit => ValidationError::DuplicateAccountWithinDebit {
                    account_id: line.account_id(),
                },
                Side::Credit => ValidationError::DuplicateAccountWithinCredit {
                    account_id: line.account_id(),
                },
            };

            if !errors.contains(&error) {
                errors.push(error);
            }
        }
    }
}

fn sum_lines(lines: &[JournalLine], errors: &mut Vec<ValidationError>) -> Option<Amount> {
    let mut total = Amount::ZERO;

    for line in lines {
        total = match total.checked_add(line.amount()) {
            Ok(sum) => sum,
            Err(crate::AmountError::Overflow) => {
                errors.push(ValidationError::TotalOverflow);
                return None;
            }
            Err(crate::AmountError::Negative { .. }) => {
                errors.push(ValidationError::NegativeAmount);
                return None;
            }
        };
    }

    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ValidationWarning;

    const 現金: u64 = 0x1000_FFFF_0001_0000;
    const 売上: u64 = 0x5000_0000_0001_0000;
    const 備品: u64 = 0x1000_4000_0017_0000;
    const 雑収入: u64 = 0x5000_0000_0002_0000;

    fn 取引日時() -> Result<TransactionAt, crate::NormalizationError> {
        TransactionAt::parse_rfc3339("2026-11-13T02:19:00Z")
    }

    fn コミット日時() -> Result<CommittedAt, crate::NormalizationError> {
        CommittedAt::parse_rfc3339("2026-11-13T02:20:00Z")
    }

    fn 行(account: u64, yen: i64) -> (AccountId, Amount) {
        (
            AccountId::new(account).expect("定義済みの勘定科目IDである"),
            Amount::from_yen(yen).expect("非負の金額である"),
        )
    }

    fn 検証する(
        debits: Vec<(u64, i64)>,
        credits: Vec<(u64, i64)>,
    ) -> Result<ValidatedJournalEntry, crate::ValidationErrors> {
        let mut builder = JournalEntry::builder()
            .transacted_at(取引日時().expect("RFC3339の日時である"))
            .committed_at(コミット日時().expect("RFC3339の日時である"));

        for (account, yen) in debits {
            builder = builder.debit(行(account, yen).0, 行(account, yen).1);
        }

        for (account, yen) in credits {
            builder = builder.credit(行(account, yen).0, 行(account, yen).1);
        }

        builder.build()
    }

    #[test]
    fn 貸借が一致する仕訳は受理される() -> Result<(), Box<dyn std::error::Error>> {
        let 検証済み = 検証する(vec![(現金, 3_500)], vec![(売上, 3_500)])?;

        assert!(検証済み.warnings().is_empty());
        assert_eq!(検証済み.journal_entry().debits().len(), 1);

        Ok(())
    }

    #[test]
    fn 貸借が不一致の仕訳は拒否される() {
        let 検証結果 = 検証する(vec![(現金, 3_500)], vec![(売上, 3_600)]);

        assert_eq!(
            検証結果.err().map(|errors| errors.into_errors()),
            Some(vec![ValidationError::BalanceMismatch {
                debit_total: Amount::from_yen(3_500).expect("非負の金額である"),
                credit_total: Amount::from_yen(3_600).expect("非負の金額である"),
            }])
        );
    }

    #[test]
    fn 同一勘定科目が二回出現する仕訳は拒否される() {
        let 借方重複 = 検証する(vec![(現金, 100), (現金, 200)], vec![(売上, 300)]);

        assert_eq!(
            借方重複.err().map(|errors| errors.into_errors()),
            Some(vec![ValidationError::DuplicateAccountWithinDebit {
                account_id: AccountId::new(現金).expect("定義済みの勘定科目IDである"),
            }])
        );

        let 貸方重複 = 検証する(vec![(現金, 300)], vec![(売上, 100), (売上, 200)]);

        assert_eq!(
            貸方重複.err().map(|errors| errors.into_errors()),
            Some(vec![ValidationError::DuplicateAccountWithinCredit {
                account_id: AccountId::new(売上).expect("定義済みの勘定科目IDである"),
            }])
        );
    }

    #[test]
    fn 左右に一回ずつ出現する同一勘定科目は拒否されない() -> Result<(), Box<dyn std::error::Error>>
    {
        let 仕入 = 0x4000_0000_0001_0000;
        let 繰越商品 = 0x1000_C000_0009_0000;

        let 検証済み = 検証する(
            vec![(仕入, 1_234), (繰越商品, 5_678)],
            vec![(繰越商品, 1_234), (仕入, 5_678)],
        )?;

        assert!(検証済み.journal_entry().debits().len() == 2);

        Ok(())
    }

    #[test]
    fn 自己取引は拒否される() {
        let 検証結果 = 検証する(vec![(現金, 100)], vec![(現金, 100)]);

        assert_eq!(
            検証結果.err().map(|errors| errors.into_errors()),
            Some(vec![ValidationError::SelfTransaction])
        );
    }

    #[test]
    fn 合計がゼロの仕訳は拒否される() {
        let 検証結果 = 検証する(vec![(現金, 0)], vec![(売上, 0)]);

        assert_eq!(
            検証結果.err().map(|errors| errors.into_errors()),
            Some(vec![ValidationError::ZeroTotal])
        );
    }

    #[test]
    fn 合計が上限を超える仕訳は拒否される() {
        let 巨額 = i64::MAX / 200;
        let 土地 = 0x1000_3D00_001A_0000;

        let 検証結果 = 検証する(
            vec![(現金, 巨額), (備品, 巨額), (土地, 巨額)],
            vec![(売上, 1)],
        );

        assert_eq!(
            検証結果.err().map(|errors| errors.into_errors()),
            Some(vec![ValidationError::TotalOverflow])
        );
    }

    #[test]
    fn 合計が2の四十乗円を超える仕訳は警告される() -> Result<(), Box<dyn std::error::Error>> {
        let 巨額 = 1_100_000_000_000;

        let 検証済み = 検証する(vec![(現金, 巨額)], vec![(売上, 巨額)])?;

        assert_eq!(
            検証済み.warnings(),
            &[ValidationWarning::LargeTotal {
                total: Amount::from_yen(巨額)?
            }]
        );

        Ok(())
    }

    #[test]
    fn 取引日時とコミット日時が未設定の仕訳は拒否される() {
        let 検証結果 = JournalEntry::builder().build();

        assert_eq!(
            検証結果.err().map(|errors| errors.into_errors()),
            Some(vec![
                ValidationError::MissingTransactionAt,
                ValidationError::MissingCommittedAt,
                ValidationError::ZeroTotal,
            ])
        );
    }

    #[test]
    fn 明細がない仕訳は拒否される() {
        let 検証結果 = JournalEntry::builder()
            .transacted_at(取引日時().expect("RFC3339の日時である"))
            .committed_at(コミット日時().expect("RFC3339の日時である"))
            .build();

        assert_eq!(
            検証結果.err().map(|errors| errors.into_errors()),
            Some(vec![ValidationError::ZeroTotal])
        );
    }

    #[test]
    fn 明細列は正規化順で保持される() -> Result<(), Box<dyn std::error::Error>> {
        let 検証済み = 検証する(
            vec![(現金, 1_000), (備品, 9_000)],
            vec![(売上, 5_000), (雑収入, 5_000)],
        )?;

        let 借方金額: Vec<i64> = 検証済み
            .journal_entry()
            .debits()
            .iter()
            .map(|line| line.amount().as_sen())
            .collect();

        assert_eq!(借方金額, vec![900_000, 100_000]);

        Ok(())
    }

    #[test]
    fn 側内の順序のみが異なる仕訳は同一の取引とみなされる() -> Result<(), Box<dyn std::error::Error>>
    {
        let 先に現金 = 検証する(
            vec![(現金, 1_000), (備品, 9_000)],
            vec![(売上, 5_000), (雑収入, 5_000)],
        )?;
        let 先に備品 = 検証する(
            vec![(備品, 9_000), (現金, 1_000)],
            vec![(雑収入, 5_000), (売上, 5_000)],
        )?;

        assert_eq!(先に現金.journal_entry(), 先に備品.journal_entry());

        Ok(())
    }

    #[test]
    fn 側内の順序のみが異なる仕訳は同一のハッシュになる() -> Result<(), Box<dyn std::error::Error>>
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let 先に現金 = 検証する(
            vec![(現金, 1_000), (備品, 9_000)],
            vec![(売上, 5_000), (雑収入, 5_000)],
        )?;
        let 先に備品 = 検証する(
            vec![(備品, 9_000), (現金, 1_000)],
            vec![(雑収入, 5_000), (売上, 5_000)],
        )?;

        let 一回目のハッシュ = {
            let mut hasher = DefaultHasher::new();
            先に現金.journal_entry().hash(&mut hasher);
            hasher.finish()
        };
        let 二回目のハッシュ = {
            let mut hasher = DefaultHasher::new();
            先に備品.journal_entry().hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(一回目のハッシュ, 二回目のハッシュ);

        Ok(())
    }

    #[test]
    fn 明細の内容が異なる仕訳は同一とみなされない() -> Result<(), Box<dyn std::error::Error>> {
        let 一方 = 検証する(vec![(現金, 1_000)], vec![(売上, 1_000)])?;
        let 他方 = 検証する(vec![(備品, 1_000)], vec![(売上, 1_000)])?;

        assert_ne!(一方.journal_entry(), 他方.journal_entry());

        Ok(())
    }
}
