#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::JournalEntryDescription;

fn 借方の金額表現(description: &str) -> Result<String, ()> {
    JournalEntryDescription::parse(description)
        .map_err(|_| ())
        .and_then(|parsed| {
            parsed
                .debits()
                .next()
                .map(|debit| debit.amount_expression().to_owned())
                .ok_or(())
        })
}

#[test]
fn 多様な桁区切り表記を受理する() {
    for 表記 in [
        "0", "000", "1000", "1,000", "1.000", "1;000", "1,,000", "1,0;00", "1e3",
    ] {
        let description = format!("現金 {表記} /\n");

        assert_eq!(借方の金額表現(&description).as_deref(), Ok(表記));
    }
}

#[test]
fn 数字で始まらない金額表現は拒否される() {
    assert!(借方の金額表現("現金 ,000 /\n").is_err());
    assert!(借方の金額表現("現金 e3 /\n").is_err());
}

#[test]
fn 数字で終わらない金額表現は拒否される() {
    assert!(借方の金額表現("現金 1, /\n").is_err());
    assert!(借方の金額表現("現金 1000. /\n").is_err());
}

#[test]
fn 空白と改行以外の_ascii制御文字を桁区切りとして受理する() {
    assert_eq!(
        借方の金額表現("現金 1\u{0007}000 /\n").as_deref(),
        Ok("1\u{0007}000")
    );
}

#[test]
fn 金額表現の内部に水平空白を許可しない() {
    assert!(借方の金額表現("現金 1 000 円 /\n").is_err());
}

#[test]
fn 非ASCII文字を含む金額表現は拒否される() {
    assert!(借方の金額表現("現金 1０00 /\n").is_err());
    assert!(借方の金額表現("現金 1，000 /\n").is_err());
}

#[test]
fn 構文上の桁数上限を設けない() {
    let 長い表記 = "1".repeat(200);
    let description = format!("現金 {長い表記} /\n");

    assert_eq!(
        借方の金額表現(&description).as_deref(),
        Ok(長い表記.as_str())
    );
}
