#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::{JournalEntryDescription, JournalEntryPrinter};

fn 整形(description: &str) -> String {
    let parsed = JournalEntryDescription::parse(description).expect("入力は受理される");

    JournalEntryPrinter::new().print(&parsed)
}

#[test]
fn 借方と貸方が同数の仕訳を各行に一組ずつ並べる() {
    let 整形結果 = 整形("現金 100 / 売上 100\n預金 200 / 売上 200\n");

    assert_eq!(整形結果, "現金 100 / 売上 100\n預金 200 / 売上 200\n");
}

#[test]
fn 貸方が多い仕訳は借方側を空けて出力する() {
    let 整形結果 = 整形("Salary 500 / Deposit 100\n/ Cash 400\n");

    assert_eq!(
        整形結果,
        "Salary 500 / Deposit 100\n           / Cash    400\n"
    );
}

#[test]
fn 借方のみの仕訳はスラッシュで行を終える() {
    let 整形結果 = 整形("現金 100 /\n預金 200 /\n");

    assert_eq!(整形結果, "現金 100 /\n預金 200 /\n");
}

#[test]
fn 貸方のみの仕訳は借方列を出力しない() {
    let 整形結果 = 整形("/ 売上 100\n/ 預金 200\n");

    assert_eq!(整形結果, "/ 売上 100\n/ 預金 200\n");
}

#[test]
fn 借方が多い仕訳は貸方側を空けてスラッシュで行を終える() {
    let 整形結果 = 整形("現金 100 / 売上 300\n預金 200 /\n");

    assert_eq!(整形結果, "現金 100 / 売上 300\n預金 200 /\n");
}

#[test]
fn 行末に水平空白を持たず末尾改行で終わる() {
    let 整形結果 = 整形("現金 100 / 売上 300\n売掛金 200 /\n/ 受取利息 50\n");

    assert!(整形結果.ends_with('\n'));

    for 行 in 整形結果.lines() {
        assert_eq!(行.trim_end(), 行, "行末に空白がある: {行:?}");
    }
}
