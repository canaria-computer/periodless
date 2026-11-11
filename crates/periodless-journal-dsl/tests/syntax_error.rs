#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::{JournalEntryDescription, SyntaxViolation};

#[test]
fn 構文違反は行と列と違反種別を報告する() {
    let error = JournalEntryDescription::parse("現金 1000 / 売上 1000\n現金 100 円 /\n")
        .expect_err("最後の部分が金額表現でない明細は構文違反");

    assert_eq!(error.violation(), SyntaxViolation::MalformedDetailLine);
    assert_eq!(error.line(), 2);
    assert!(error.column() >= 1);
}

#[test]
fn 明細を一件も含まない入力は明細不在として報告される() {
    let error =
        JournalEntryDescription::parse("   \n\t\n").expect_err("明細が一件もない入力は構文違反");

    assert_eq!(error.violation(), SyntaxViolation::MissingDetail);
}

#[test]
fn 構文エラーは標準のエラートレイトを実装する() {
    let error = JournalEntryDescription::parse("/\n").expect_err("明細のない行は構文違反");
    let reported: &dyn std::error::Error = &error;

    assert!(!reported.to_string().is_empty());
}
