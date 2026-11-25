use crate::account_id::AccountId;
use crate::amount::Amount;
use core::fmt::{Display, Formatter, Result as FormatResult};

/// 仕訳の検証で検出された違反（記録を拒否する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// 取引日時が未設定である。
    MissingTransactionAt,
    /// コミット日時が未設定である。
    MissingCommittedAt,
    /// 省略行より前に、両側の明細を持つ行が存在しない。
    OmissionWithoutLeadingCompleteLine { line: usize },
    /// 借方の明細の中に同一の勘定科目が2回出現する。
    DuplicateAccountWithinDebit { account_id: AccountId },
    /// 貸方の明細の中に同一の勘定科目が2回出現する。
    DuplicateAccountWithinCredit { account_id: AccountId },
    /// 借方と貸方の明細列が完全に一致する（自己取引）。
    SelfTransaction,
    /// 借方合計と貸方合計が一致しない。
    BalanceMismatch {
        debit_total: Amount,
        credit_total: Amount,
    },
    /// 合計金額が0である。
    ZeroTotal,
    /// 合計金額が符号付き64bit整数の最大値を超える。
    TotalOverflow,
    /// 金額が負である。
    NegativeAmount,
    /// 桁区切り文字（`_`・`'`・`.`・`,`）が連続して現れる。
    ConsecutiveSeparators { amount_expression: String },
    /// 桁区切り文字以外の印字可能ASCII文字が金額表現に含まれる。
    NonCanonicalSeparator {
        amount_expression: String,
        character: char,
    },
    /// 金額表現が基数プレフィックス表記（`0b`・`0x`・`0o` 等）と混同される。
    RadixPrefixedAmount { amount_expression: String },
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self {
            ValidationError::MissingTransactionAt => {
                write!(formatter, "取引日時が設定されていません")
            }
            ValidationError::MissingCommittedAt => {
                write!(formatter, "コミット日時が設定されていません")
            }
            ValidationError::OmissionWithoutLeadingCompleteLine { line } => write!(
                formatter,
                "{line}行目: 省略行より前に両側の明細を持つ行が存在しません"
            ),
            ValidationError::DuplicateAccountWithinDebit { account_id } => write!(
                formatter,
                "借方の中で勘定科目 {:#018x} が2回出現しています",
                account_id.as_u64()
            ),
            ValidationError::DuplicateAccountWithinCredit { account_id } => write!(
                formatter,
                "貸方の中で勘定科目 {:#018x} が2回出現しています",
                account_id.as_u64()
            ),
            ValidationError::SelfTransaction => {
                write!(
                    formatter,
                    "借方と貸方の明細列が完全に一致しています（自己取引）"
                )
            }
            ValidationError::BalanceMismatch {
                debit_total,
                credit_total,
            } => write!(
                formatter,
                "貸借が一致しません: 借方合計 {debit_total} 銭、貸方合計 {credit_total} 銭"
            ),
            ValidationError::ZeroTotal => {
                write!(formatter, "合計金額が0です")
            }
            ValidationError::TotalOverflow => {
                write!(
                    formatter,
                    "合計金額が符号付き64bit整数の最大値を超えています"
                )
            }
            ValidationError::NegativeAmount => {
                write!(formatter, "金額が負です")
            }
            ValidationError::ConsecutiveSeparators { amount_expression } => write!(
                formatter,
                "金額表現 {amount_expression} の桁区切り文字が連続しています"
            ),
            ValidationError::NonCanonicalSeparator {
                amount_expression,
                character,
            } => write!(
                formatter,
                "金額表現 {amount_expression} に桁区切り文字以外の文字 {character} が含まれています"
            ),
            ValidationError::RadixPrefixedAmount { amount_expression } => write!(
                formatter,
                "金額表現 {amount_expression} は基数プレフィックス表記と混同されます"
            ),
        }
    }
}

impl core::error::Error for ValidationError {}

/// 仕訳の検証で検出された警告（記録は受理し、呼び出し元へ通知する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarning {
    /// 金額の直前にある勘定科目名の末尾が数字である。
    AccountNameEndsWithDigit { account_name: String },
    /// 合計金額が2^40円を超える（誤入力の可能性）。
    LargeTotal { total: Amount },
    /// 金額表現が0埋めされている（外部インターフェースが8進数と解釈する恐れ）。
    ZeroPaddedAmount { amount_expression: String },
}

impl Display for ValidationWarning {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self {
            ValidationWarning::AccountNameEndsWithDigit { account_name } => write!(
                formatter,
                "勘定科目名 {account_name} の末尾が数字です（金額との区切りを確認してください）"
            ),
            ValidationWarning::LargeTotal { total } => write!(
                formatter,
                "合計金額 {total} 円が 2^40 円を超えています（誤入力の可能性があります）"
            ),
            ValidationWarning::ZeroPaddedAmount { amount_expression } => write!(
                formatter,
                "金額表現 {amount_expression} は0埋めされています（8進数と解釈される恐れがあります）"
            ),
        }
    }
}

/// 検出されたすべての違反。
///
/// 1件でも違反がある場合に記録を拒否するため、呼び出し元は検出した違反を
/// すべて受け取る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors(Vec<ValidationError>);

impl ValidationErrors {
    /// 違反の列から構築する。
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self(errors)
    }

    /// 違反が一件もない場合に `true`。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 検出された違反。
    pub fn errors(&self) -> &[ValidationError] {
        &self.0
    }

    /// 検出された違返を消費して取り出す。
    pub fn into_errors(self) -> Vec<ValidationError> {
        self.0
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        for (index, error) in self.0.iter().enumerate() {
            if index > 0 {
                writeln!(formatter)?;
            }

            write!(formatter, "{}", error)?;
        }

        Ok(())
    }
}

impl core::error::Error for ValidationErrors {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.0
            .first()
            .map(|error| error as &(dyn core::error::Error + 'static))
    }
}

impl From<Vec<ValidationError>> for ValidationErrors {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self(errors)
    }
}
