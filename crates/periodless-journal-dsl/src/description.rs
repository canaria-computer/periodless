use crate::grammar::{JournalEntryGrammar, Rule};
use crate::syntax_error::{SyntaxError, SyntaxViolation};
use pest::Parser;
use pest::error::{Error as PestError, LineColLocation};
use pest::iterators::Pair;

/// 一件の明細。科目名と、入力に現れたままの金額表現を保持する。
///
/// 大文字小文字・全角半角・内部空白の正規化は行わず、金額表現の桁区切り文字も
/// 除去しない。整数値への変換は構文解析とは別の責務として扱う。
///
/// # Examples
///
/// ```
/// use periodless_journal_dsl::JournalEntryDescription;
///
/// let description = JournalEntryDescription::parse("売買目的  有価証券 1,0;00 /\n")?;
/// let debit = description.debits().next().ok_or("借方明細が存在しない")?;
///
/// assert_eq!(debit.account_name(), "売買目的  有価証券");
/// assert_eq!(debit.amount_expression(), "1,0;00");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescribedDetail {
    account_name: String,
    amount_expression: String,
}

impl DescribedDetail {
    /// 入力に現れたままの科目名。
    pub fn account_name(&self) -> &str {
        &self.account_name
    }

    /// 入力に現れたままの金額表現。
    pub fn amount_expression(&self) -> &str {
        &self.amount_expression
    }
}

/// 仕訳記述の構文解析結果。借方明細列と貸方明細列を出現順に保持する。
///
/// 同じ行の借方明細と貸方明細に対応関係は持たせず、同一科目の複数出現も
/// 集約せずに保持する。
///
/// # Examples
///
/// ```
/// use periodless_journal_dsl::JournalEntryDescription;
///
/// let description = JournalEntryDescription::parse("現金 600 / 売上 1000\n売掛金 400 /\n")?;
///
/// let 借方の科目名: Vec<&str> = description.debits().map(|debit| debit.account_name()).collect();
///
/// assert_eq!(借方の科目名, vec!["現金", "売掛金"]);
/// assert_eq!(description.credits().count(), 1);
/// # Ok::<(), periodless_journal_dsl::SyntaxError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntryDescription {
    debits: Vec<DescribedDetail>,
    credits: Vec<DescribedDetail>,
}

impl JournalEntryDescription {
    /// 入力全体を一つの仕訳として解析する。
    ///
    /// 空白のみの行は無視し、入力を複数の仕訳へ分割しない。すべての行は改行で
    /// 終了していなければならず、明細を一件も含まない入力は構文違反となる。
    ///
    /// # Errors
    ///
    /// 明細行がいずれの形も満たさない場合、または明細が一件も存在しない場合に
    /// [`SyntaxError`] を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_journal_dsl::JournalEntryDescription;
    ///
    /// let description = JournalEntryDescription::parse("科目 1 000 /\n")?;
    /// let debit = description.debits().next().ok_or("借方明細が存在しない")?;
    ///
    /// assert_eq!(debit.account_name(), "科目 1");
    /// assert_eq!(debit.amount_expression(), "000");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(description: &str) -> Result<Self, SyntaxError> {
        let parsed = JournalEntryGrammar::parse(Rule::journal_entry, description)
            .map_err(malformed_detail_line)?;

        let mut debits = Vec::new();
        let mut credits = Vec::new();

        for journal_entry in parsed {
            for line in journal_entry.into_inner() {
                collect_sides(line, &mut debits, &mut credits);
            }
        }

        if debits.is_empty() && credits.is_empty() {
            return Err(SyntaxError::new(1, 1, SyntaxViolation::MissingDetail));
        }

        Ok(Self { debits, credits })
    }

    /// 借方明細を入力での出現順に返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_journal_dsl::JournalEntryDescription;
    ///
    /// let description = JournalEntryDescription::parse("現金 100 /\n現金 200 /\n")?;
    ///
    /// let 金額表現: Vec<&str> = description
    ///     .debits()
    ///     .map(|debit| debit.amount_expression())
    ///     .collect();
    ///
    /// assert_eq!(金額表現, vec!["100", "200"]);
    /// # Ok::<(), periodless_journal_dsl::SyntaxError>(())
    /// ```
    pub fn debits(&self) -> impl Iterator<Item = &DescribedDetail> {
        self.debits.iter()
    }

    /// 貸方明細を入力での出現順に返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_journal_dsl::JournalEntryDescription;
    ///
    /// let description = JournalEntryDescription::parse("/ 売上 1000\n")?;
    ///
    /// assert_eq!(description.credits().count(), 1);
    /// assert_eq!(description.debits().count(), 0);
    /// # Ok::<(), periodless_journal_dsl::SyntaxError>(())
    /// ```
    pub fn credits(&self) -> impl Iterator<Item = &DescribedDetail> {
        self.credits.iter()
    }
}

fn collect_sides(
    line: Pair<'_, Rule>,
    debits: &mut Vec<DescribedDetail>,
    credits: &mut Vec<DescribedDetail>,
) {
    match line.as_rule() {
        Rule::detail_line | Rule::both_sides | Rule::debit_only | Rule::credit_only => {
            for side in line.into_inner() {
                collect_sides(side, debits, credits);
            }
        }
        Rule::debit => {
            debits.extend(described_detail(line));
        }
        Rule::credit => {
            credits.extend(described_detail(line));
        }
        _ => {}
    }
}

fn described_detail(side: Pair<'_, Rule>) -> Option<DescribedDetail> {
    let detail = side.into_inner().next()?;
    let mut parts = detail.into_inner();
    let account_name = parts.next()?.as_str().to_owned();
    let amount_expression = parts.next()?.as_str().to_owned();

    Some(DescribedDetail {
        account_name,
        amount_expression,
    })
}

fn malformed_detail_line(error: PestError<Rule>) -> SyntaxError {
    let (line, column) = match error.line_col {
        LineColLocation::Pos(position) => position,
        LineColLocation::Span(start, _) => start,
    };

    SyntaxError::new(line, column, SyntaxViolation::MalformedDetailLine)
}
