#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::JournalEntryDescription;

#[test]
fn 借方と貸方の両方を持つ行を受理する() {
    let description = JournalEntryDescription::parse("現金 1000 / 売上 1000\n")
        .expect("借方と貸方を持つ行は受理される");

    assert_eq!(description.debits().count(), 1);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn 借方のみの行を受理する() {
    let description =
        JournalEntryDescription::parse("現金 1000 /\n").expect("借方のみの行は受理される");

    assert_eq!(description.debits().count(), 1);
    assert_eq!(description.credits().count(), 0);
}

#[test]
fn 貸方のみの行を受理する() {
    let description =
        JournalEntryDescription::parse("   / 売上 1000\n").expect("貸方のみの行は受理される");

    assert_eq!(description.debits().count(), 0);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn 行頭と行末およびスラッシュ前後の水平空白を許可する() {
    let description = JournalEntryDescription::parse("\t 現金 1000 \t/\t 売上 1000 \t\n")
        .expect("前後の水平空白は許可される");

    assert_eq!(description.debits().count(), 1);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn 左右とも明細が存在しない行は拒否される() {
    assert!(JournalEntryDescription::parse("/\n").is_err());
    assert!(JournalEntryDescription::parse("   /   \n").is_err());
}

#[test]
fn 空白のみの行は無視され仕訳を分割しない() {
    let description = JournalEntryDescription::parse("現金 1000 /\n   \n/ 売上 1000\n")
        .expect("空白のみの行は無視される");

    assert_eq!(description.debits().count(), 1);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn 省略された側に前の行の明細を引き継がない() {
    let description = JournalEntryDescription::parse("給料 400000 / 現金 350000\n売掛金 50000 /\n")
        .expect("片側省略の行は受理される");

    assert_eq!(description.debits().count(), 2);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn 連続する明細行は同一の仕訳に属する() {
    let description = JournalEntryDescription::parse("現金 600 / 売上 1000\n売掛金 400 /\n")
        .expect("連続する明細行は受理される");

    assert_eq!(description.debits().count(), 2);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn CRLF改行の入力を解析できる() {
    let description = JournalEntryDescription::parse("現金 1000 / 売上 1000\r\n売掛金 500 /\r\n")
        .expect("CRLFは一つの改行として扱われる");

    assert_eq!(description.debits().count(), 2);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn 末尾の改行がない入力は拒否される() {
    assert!(JournalEntryDescription::parse("現金 1000 / 売上 1000").is_err());
    assert!(JournalEntryDescription::parse("現金 1000 / 売上 1000\n   ").is_err());
}

#[test]
fn 明細を一件も含まない入力は拒否される() {
    assert!(JournalEntryDescription::parse("").is_err());
    assert!(JournalEntryDescription::parse("\n").is_err());
    assert!(JournalEntryDescription::parse("   \n\t\n").is_err());
}
