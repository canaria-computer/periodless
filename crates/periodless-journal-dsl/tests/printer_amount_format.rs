#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::{JournalEntryDescription, JournalEntryPrinter};

fn 整形後の借方金額(金額表現: &str) -> String {
    let parsed =
        JournalEntryDescription::parse(&format!("現金 {金額表現} /\n")).expect("入力は受理される");
    let 整形結果 = JournalEntryPrinter::new().print(&parsed);

    整形結果
        .trim_end_matches(['\n', ' ', '/'])
        .trim_start_matches("現金")
        .trim()
        .to_owned()
}

#[test]
fn 三桁ごとにカンマを挿入する() {
    for (金額表現, 正規形) in [
        ("1000", "1,000"),
        ("1,000", "1,000"),
        ("1.000", "1,000"),
        ("1;000", "1,000"),
        ("1,,000", "1,000"),
        ("1,0;00", "1,000"),
        ("1e3", "13"),
        ("400000", "400,000"),
        ("1234567", "1,234,567"),
    ] {
        assert_eq!(整形後の借方金額(金額表現), 正規形, "金額表現: {金額表現}");
    }
}

#[test]
fn 先頭のゼロを除去する() {
    assert_eq!(整形後の借方金額("000"), "0");
    assert_eq!(整形後の借方金額("0"), "0");
    assert_eq!(整形後の借方金額("0,001"), "1");
    assert_eq!(整形後の借方金額("0001000"), "1,000");
}

#[test]
fn 整数型に収まらない桁数の金額表現も整形できる() {
    let 四十桁 = "9".repeat(40);
    let 期待値 = format!("9,{}", (0..13).map(|_| "999").collect::<Vec<_>>().join(","));

    assert_eq!(整形後の借方金額(&四十桁), 期待値);
}
