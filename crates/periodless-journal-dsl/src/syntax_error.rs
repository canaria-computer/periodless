use core::fmt::{Display, Formatter, Result as FormatResult};

/// 構文違反の種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxViolation {
    /// 明細行が `借方明細 / 貸方明細`、`借方明細 /`、`/ 貸方明細`
    /// のいずれの形も満たさない。
    MalformedDetailLine,
    /// 入力が空白のみの行で構成され、明細を一件も含まない。
    MissingDetail,
}

/// 仕訳記述の構文違反。違反箇所の位置と種別を保持する。
///
/// # Examples
///
/// ```
/// use periodless_journal_dsl::{JournalEntryDescription, SyntaxViolation};
///
/// let reported = JournalEntryDescription::parse("現金 100 円 /\n")
///     .err()
///     .map(|error| (error.line(), error.violation()));
///
/// assert_eq!(reported, Some((1, SyntaxViolation::MalformedDetailLine)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    line: usize,
    column: usize,
    violation: SyntaxViolation,
}

impl SyntaxError {
    pub(crate) fn new(line: usize, column: usize, violation: SyntaxViolation) -> Self {
        Self {
            line,
            column,
            violation,
        }
    }

    /// 違反箇所の行番号（1始まり）。
    pub fn line(&self) -> usize {
        self.line
    }

    /// 違反箇所の列番号（1始まり）。
    pub fn column(&self) -> usize {
        self.column
    }

    /// 違反の種別。
    pub fn violation(&self) -> SyntaxViolation {
        self.violation
    }
}

impl Display for SyntaxError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        let violation = match self.violation {
            SyntaxViolation::MalformedDetailLine => "明細行の構文が不正です",
            SyntaxViolation::MissingDetail => "明細が一件も存在しません",
        };

        write!(formatter, "{}行{}列: {}", self.line, self.column, violation)
    }
}

impl core::error::Error for SyntaxError {}
