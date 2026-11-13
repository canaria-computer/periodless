use core::fmt::{Display, Formatter, Result as FormatResult};

/// 金額表現が表す整数が `i64` の範囲に収まらないことを表すエラー。
///
/// 構文解析は金額表現の桁数に上限を設けないため、この違反は整数値への変換を
/// 要求した時点でのみ報告される。
///
/// # Examples
///
/// ```
/// use periodless_journal_dsl::JournalEntryDescription;
///
/// let 巨大な金額表現 = "9".repeat(40);
/// let description = JournalEntryDescription::parse(&format!("現金 {巨大な金額表現} /\n"))?;
/// let debit = description.debits().next().ok_or("借方明細が存在しない")?;
///
/// let overflow = debit.amount_as_integer().err().ok_or("範囲超過が報告されない")?;
///
/// assert_eq!(overflow.amount_expression(), 巨大な金額表現);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmountOverflow {
    amount_expression: String,
}

impl AmountOverflow {
    pub(crate) fn new(amount_expression: &str) -> Self {
        Self {
            amount_expression: amount_expression.to_owned(),
        }
    }

    /// 範囲を超えた元の金額表現。
    pub fn amount_expression(&self) -> &str {
        &self.amount_expression
    }
}

impl Display for AmountOverflow {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(
            formatter,
            "金額表現 {} が表す整数は i64 の範囲を超えています",
            self.amount_expression
        )
    }
}

impl core::error::Error for AmountOverflow {}
