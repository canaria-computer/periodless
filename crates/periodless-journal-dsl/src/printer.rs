use crate::description::{DescribedDetail, JournalEntryDescription};
use unicode_width::UnicodeWidthStr;

/// 構文解析結果を正規形の仕訳記述テキストへ整形する。
///
/// 借方明細と貸方明細を同じ行に並べ、科目名を左揃え・金額を右揃えに桁揃えする。
/// 桁揃えの幅は Unicode の East Asian Width に基づく表示幅で数えるため、全角文字は
/// 幅2として扱われる。
///
/// 同じ行に並ぶ借方明細と貸方明細は表示上の配置に過ぎず、両者の対応関係を表す
/// ものではない。
///
/// # Examples
///
/// ```
/// use periodless_journal_dsl::{JournalEntryDescription, JournalEntryPrinter};
///
/// let description = JournalEntryDescription::parse("現金 100 / 売上 1000\n売掛金 20 /\n")?;
///
/// assert_eq!(
///     JournalEntryPrinter::new().print(&description),
///     "現金   100 / 売上 1,000\n売掛金  20 /\n",
/// );
/// # Ok::<(), periodless_journal_dsl::SyntaxError>(())
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JournalEntryPrinter;

impl JournalEntryPrinter {
    /// 既定の整形設定を持つプリンタを返す。
    pub fn new() -> Self {
        Self
    }

    /// 構文解析結果を正規形のテキストへ整形する。
    ///
    /// 金額は三桁ごとのカンマ区切りへ正規化されるため、元の金額表現の桁区切り文字は
    /// 保持されない。整形結果は再解析でき、再解析して再び整形した結果は同一になる。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_journal_dsl::{JournalEntryDescription, JournalEntryPrinter};
    ///
    /// let printer = JournalEntryPrinter::new();
    /// let 一回目 = printer.print(&JournalEntryDescription::parse("現金 1.000 /\n")?);
    /// let 二回目 = printer.print(&JournalEntryDescription::parse(&一回目)?);
    ///
    /// assert_eq!(一回目, "現金 1,000 /\n");
    /// assert_eq!(二回目, 一回目);
    /// # Ok::<(), periodless_journal_dsl::SyntaxError>(())
    /// ```
    pub fn print(&self, description: &JournalEntryDescription) -> String {
        let debits: Vec<&DescribedDetail> = description.debits().collect();
        let credits: Vec<&DescribedDetail> = description.credits().collect();
        let debit_layout = ColumnLayout::of(&debits);
        let credit_layout = ColumnLayout::of(&credits);

        let mut printed = String::new();

        for line in 0..debits.len().max(credits.len()) {
            if !debits.is_empty() {
                match debits.get(line) {
                    Some(debit) => printed.push_str(&aligned_detail_text(&debit_layout, debit)),
                    None => printed.push_str(&" ".repeat(debit_layout.total_width())),
                }
                printed.push(' ');
            }

            printed.push('/');

            if let Some(credit) = credits.get(line) {
                printed.push(' ');
                printed.push_str(&aligned_detail_text(&credit_layout, credit));
            }

            printed.push('\n');
        }

        printed
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ColumnLayout {
    account_name_width: usize,
    amount_width: usize,
}

impl ColumnLayout {
    fn of(details: &[&DescribedDetail]) -> Self {
        Self {
            account_name_width: details
                .iter()
                .map(|detail| detail.account_name().width())
                .max()
                .unwrap_or(0),
            amount_width: details
                .iter()
                .map(|detail| amount_text(detail).width())
                .max()
                .unwrap_or(0),
        }
    }

    fn total_width(&self) -> usize {
        self.account_name_width + 1 + self.amount_width
    }
}

fn aligned_detail_text(layout: &ColumnLayout, detail: &DescribedDetail) -> String {
    let account_name = detail.account_name();
    let amount = amount_text(detail);
    let account_name_padding = " ".repeat(
        layout
            .account_name_width
            .saturating_sub(account_name.width()),
    );
    let amount_padding = " ".repeat(layout.amount_width.saturating_sub(amount.width()));

    format!("{account_name}{account_name_padding} {amount_padding}{amount}")
}

fn amount_text(detail: &DescribedDetail) -> String {
    let digits: String = detail
        .amount_expression()
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();
    let significant_digits = digits.trim_start_matches('0');

    if significant_digits.is_empty() {
        return "0".to_owned();
    }

    let mut grouped = String::new();

    for (position, digit) in significant_digits.chars().enumerate() {
        if position > 0 && (significant_digits.len() - position).is_multiple_of(3) {
            grouped.push(',');
        }

        grouped.push(digit);
    }

    grouped
}

impl core::fmt::Display for JournalEntryDescription {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&JournalEntryPrinter::new().print(self))
    }
}
