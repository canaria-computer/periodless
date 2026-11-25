use crate::amount_overflow::AmountOverflow;
use crate::grammar::{JournalEntryGrammar, Rule};
use crate::syntax_error::{SyntaxError, SyntaxViolation};
use periodless_core::{ValidationError, ValidationErrors, ValidationWarning};
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

    /// 金額表現が表す整数値。
    ///
    /// 金額表現から ASCII 数字以外を除去し、残った数字列を十進整数として解釈する。
    /// 桁区切り文字は数値的な意味を持たないため、`1e3` は 13、`000` は 0 になる。
    /// 金額表現は ASCII 数字で始まり ASCII 数字で終わる構文上の制約を持ち、符号文字も
    /// 除去されるため、負の値が得られることはない。
    ///
    /// 得られる整数は金額表現が表す数そのものであり、通貨および最小通貨単位の意味を
    /// 持たない。単位を伴う金額型への詰め替えは利用側の責務とする。
    ///
    /// # Errors
    ///
    /// 構文解析は金額表現の桁数に上限を設けないため、整数が `i64` の範囲に
    /// 収まらない場合に [`AmountOverflow`] を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_journal_dsl::JournalEntryDescription;
    ///
    /// let description = JournalEntryDescription::parse("現金 1,0;00 /\n")?;
    /// let debit = description.debits().next().ok_or("借方明細が存在しない")?;
    ///
    /// assert_eq!(debit.amount_as_integer()?, 1000);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn amount_as_integer(&self) -> Result<i64, AmountOverflow> {
        let digits: String = self
            .amount_expression
            .chars()
            .filter(|character| character.is_ascii_digit())
            .collect();

        digits
            .parse()
            .map_err(|_| AmountOverflow::new(&self.amount_expression))
    }
}

/// 一件の明細行。借方の明細と貸方の明細の有無を行の並び順で保持する。
///
/// 借方のみの行・貸方のみの行・両方を持つ行のいずれかである。同じ行の借方明細と
/// 貸方明細に個別の対応関係は持たせない。
///
/// # Examples
///
/// ```
/// use periodless_journal_dsl::JournalEntryDescription;
///
/// let description =
///     JournalEntryDescription::parse("現金 100 / 売上 1000\n売掛金 20 /\n")?;
/// let 借方のみの行 = description.lines().nth(1).ok_or("行が存在しない")?;
///
/// assert!(借方のみの行.debit().is_some());
/// assert!(借方のみの行.credit().is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescribedLine {
    debit: Option<DescribedDetail>,
    credit: Option<DescribedDetail>,
}

impl DescribedLine {
    /// 借方の明細。省略されている場合は `None`。
    pub fn debit(&self) -> Option<&DescribedDetail> {
        self.debit.as_ref()
    }

    /// 貸方の明細。省略されている場合は `None`。
    pub fn credit(&self) -> Option<&DescribedDetail> {
        self.credit.as_ref()
    }
}

/// 仕訳記述の構文解析結果。明細行の並びを保持する。
///
/// 借方明細列と貸方明細列は、各行から該当する側の明細を取り出した列として
/// 参照できる。同一科目の複数出現も集約せずに保持する。
///
/// 等価性（`PartialEq` / `Eq`）とハッシュ（`Hash`）は、借方明細列と貸方明細列の
/// それぞれを正規形（金額の数値の降順、次いで科目名の文字列長の降順、次いで
/// 科目名の辞書順の降順による安定ソート）で並べ替えた列に基づく。この等価性は
/// 仕訳だけを見た取引の同一性であり、同一内容の複数の取引は日時等によって
/// 判別する。保持する明細の並び順（入力順）は整形出力のために変更しない。
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
#[derive(Debug, Clone)]
pub struct JournalEntryDescription {
    lines: Vec<DescribedLine>,
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

        let mut lines = Vec::new();

        for journal_entry in parsed {
            for line in journal_entry.into_inner() {
                collect_line(line, &mut lines);
            }
        }

        if lines
            .iter()
            .all(|line| line.debit.is_none() && line.credit.is_none())
        {
            return Err(SyntaxError::new(1, 1, SyntaxViolation::MissingDetail));
        }

        Ok(Self { lines })
    }

    /// 明細行を行の並び順に返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_journal_dsl::JournalEntryDescription;
    ///
    /// let description = JournalEntryDescription::parse("現金 100 /\n / 売上 100\n")?;
    ///
    /// assert_eq!(description.lines().count(), 2);
    /// # Ok::<(), periodless_journal_dsl::SyntaxError>(())
    /// ```
    pub fn lines(&self) -> impl Iterator<Item = &DescribedLine> {
        self.lines.iter()
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
        self.lines.iter().filter_map(|line| line.debit.as_ref())
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
        self.lines.iter().filter_map(|line| line.credit.as_ref())
    }

    /// 仕訳記述の検証を行い、検出した警告を返す。
    ///
    /// 構文上セーフでも実務上支障になる記述を、違反（記録を拒否する）と警告
    /// （記録を受理して通知する）に区分して報告する。違反と警告の区分は
    /// `periodless-core` の検証仕様に従い、検出したすべての違反を
    /// [`ValidationErrors`] として返す。
    ///
    /// # Errors
    ///
    /// 検出したすべての違反を [`ValidationErrors`] として返す。違反には、
    /// 省略行より前に両側の明細を持つ行が存在しないこと、および金額表現に
    /// 関する違反（連続する桁区切り文字・桁区切り文字以外の印字可能ASCII文字・
    /// 基数プレフィックス表記）が含まれる。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_journal_dsl::JournalEntryDescription;
    ///
    /// let description = JournalEntryDescription::parse("現金 1,000 / 売上 1,000\n")?;
    /// let 警告 = description.validate()?;
    ///
    /// assert!(警告.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationErrors> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        self.validate_lines(&mut errors);
        self.validate_details(&mut errors, &mut warnings);

        if errors.is_empty() {
            return Ok(warnings);
        }

        Err(ValidationErrors::new(errors))
    }

    fn validate_lines(&self, errors: &mut Vec<ValidationError>) {
        let 最初の省略行 = self
            .lines
            .iter()
            .position(|line| line.debit.is_some() != line.credit.is_some());

        if let Some(position) = 最初の省略行 {
            let 直前に行全体が存在する = self.lines[..position]
                .iter()
                .any(|line| line.debit.is_some() && line.credit.is_some());

            if !直前に行全体が存在する {
                errors.push(ValidationError::OmissionWithoutLeadingCompleteLine {
                    line: position + 1,
                });
            }
        }
    }

    fn validate_details(
        &self,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        for detail in self.debits().chain(self.credits()) {
            validate_account_name(detail.account_name(), warnings);
            validate_amount_expression(detail.amount_expression(), errors, warnings);
        }
    }

    /// 正規化された借方明細列（金額の数値・科目名の文字列長・科目名の辞書順の
    /// 降順安定ソート）。保持する明細の並び順は変更しない。
    fn canonical_debits(&self) -> Vec<CanonicalDetail> {
        canonical_details(self.debits())
    }

    /// 正規化された貸方明細列。保持する明細の並び順は変更しない。
    fn canonical_credits(&self) -> Vec<CanonicalDetail> {
        canonical_details(self.credits())
    }
}

impl PartialEq for JournalEntryDescription {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_debits() == other.canonical_debits()
            && self.canonical_credits() == other.canonical_credits()
    }
}

impl Eq for JournalEntryDescription {}

impl std::hash::Hash for JournalEntryDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for detail in self.canonical_debits() {
            detail.hash(state);
        }

        for detail in self.canonical_credits() {
            detail.hash(state);
        }
    }
}

/// 正規化された明細。等価性のキーは科目名と金額の数値（正規化数字列）のみ。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
struct CanonicalDetail {
    account_name: String,
    normalized_digits: String,
}

impl CanonicalDetail {
    /// 正規化のソートキー（降順比較）。
    fn descending(&self, other: &Self) -> std::cmp::Ordering {
        other
            .normalized_digits
            .len()
            .cmp(&self.normalized_digits.len())
            .then_with(|| {
                other
                    .account_name
                    .chars()
                    .count()
                    .cmp(&self.account_name.chars().count())
            })
            .then_with(|| other.account_name.cmp(&self.account_name))
    }
}

fn canonical_details<'a>(
    details: impl Iterator<Item = &'a DescribedDetail>,
) -> Vec<CanonicalDetail> {
    let mut canonical: Vec<CanonicalDetail> = details
        .map(|detail| CanonicalDetail {
            account_name: detail.account_name().to_owned(),
            normalized_digits: normalized_digits(detail.amount_expression()),
        })
        .collect();

    canonical.sort_by(|left, right| left.descending(right));

    canonical
}

/// 金額表現から桁区切り文字と先頭ゼロを除去した数字列。すべてゼロの場合は `0`。
fn normalized_digits(amount_expression: &str) -> String {
    let digits: String = amount_expression
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();

    let significant = digits.trim_start_matches('0');

    if significant.is_empty() {
        "0".to_owned()
    } else {
        significant.to_owned()
    }
}

/// 金額の直前にある勘定科目名の末尾が数字である場合の警告（ルール5）。
fn validate_account_name(account_name: &str, warnings: &mut Vec<ValidationWarning>) {
    let 末尾が数字 = account_name
        .chars()
        .last()
        .is_some_and(|character| character.is_ascii_digit());

    if 末尾が数字 {
        warnings.push(ValidationWarning::AccountNameEndsWithDigit {
            account_name: account_name.to_owned(),
        });
    }
}

/// 金額表現の検証（ルール6・7・8・0埋めの警告）。
fn validate_amount_expression(
    amount_expression: &str,
    errors: &mut Vec<ValidationError>,
    warnings: &mut Vec<ValidationWarning>,
) {
    let 桁区切り文字 = ['_', '\'', '.', ','];
    let characters: Vec<char> = amount_expression.chars().collect();

    let 連続する区切り = characters
        .windows(2)
        .any(|pair| 桁区切り文字.contains(&pair[0]) && 桁区切り文字.contains(&pair[1]));

    if 連続する区切り {
        errors.push(ValidationError::ConsecutiveSeparators {
            amount_expression: amount_expression.to_owned(),
        });
    }

    let 非正準の印字可能文字 = characters.iter().copied().find(|character| {
        !character.is_ascii_digit()
            && character.is_ascii_graphic()
            && !桁区切り文字.contains(character)
    });

    if let Some(character) = 非正準の印字可能文字 {
        errors.push(ValidationError::NonCanonicalSeparator {
            amount_expression: amount_expression.to_owned(),
            character,
        });
    }

    let 基数プレフィックス = characters.len() >= 2
        && characters[0] == '0'
        && matches!(characters[1], 'b' | 'B' | 'x' | 'X' | 'o' | 'O');

    if 基数プレフィックス {
        errors.push(ValidationError::RadixPrefixedAmount {
            amount_expression: amount_expression.to_owned(),
        });
    }

    let 数字列: String = amount_expression
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();

    if 数字列.len() >= 2 && 数字列.starts_with('0') {
        warnings.push(ValidationWarning::ZeroPaddedAmount {
            amount_expression: amount_expression.to_owned(),
        });
    }
}

fn collect_line(line: Pair<'_, Rule>, lines: &mut Vec<DescribedLine>) {
    match line.as_rule() {
        Rule::detail_line => {
            for side in line.into_inner() {
                collect_line(side, lines);
            }
        }
        Rule::both_sides | Rule::debit_only | Rule::credit_only => {
            let mut debit = None;
            let mut credit = None;

            for side in line.into_inner() {
                match side.as_rule() {
                    Rule::debit => debit = described_detail(side),
                    Rule::credit => credit = described_detail(side),
                    _ => {}
                }
            }

            lines.push(DescribedLine { debit, credit });
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
