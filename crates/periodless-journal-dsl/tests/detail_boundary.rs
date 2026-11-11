#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::JournalEntryDescription;

#[test]
fn 最後の空白区切り部分を金額表現として解析する() {
    let description =
        JournalEntryDescription::parse("科目 1 000 /\n").expect("最後の部分が金額表現になる");

    let debit = description.debits().next().expect("借方明細が存在する");

    assert_eq!(debit.account_name(), "科目 1");
    assert_eq!(debit.amount_expression(), "000");
}

#[test]
fn 科目名内部の水平空白はそのまま保持される() {
    let description = JournalEntryDescription::parse("売買目的  有価証券 1000 /\n")
        .expect("科目名内部の空白は許可される");

    let debit = description.debits().next().expect("借方明細が存在する");

    assert_eq!(debit.account_name(), "売買目的  有価証券");
    assert_eq!(debit.amount_expression(), "1000");
}

#[test]
fn 最後の部分が金額表現の構文を満たさない明細は拒否される() {
    assert!(JournalEntryDescription::parse("現金 100 円 /\n").is_err());
    assert!(JournalEntryDescription::parse("/ 現金 100 円\n").is_err());
}

#[test]
fn 金額表現のみの明細は拒否される() {
    assert!(JournalEntryDescription::parse("1,000 /\n").is_err());
    assert!(JournalEntryDescription::parse("/ 1000\n").is_err());
}

#[test]
fn 科目名のみの明細は拒否される() {
    assert!(JournalEntryDescription::parse("現金 /\n").is_err());
    assert!(JournalEntryDescription::parse("/ 売上\n").is_err());
}

#[test]
fn 明細前後の水平空白と区切り空白は科目名に含まれない() {
    let description = JournalEntryDescription::parse("\t  現金 \t 1000  \t/\n")
        .expect("前後の水平空白は許可される");

    let debit = description.debits().next().expect("借方明細が存在する");

    assert_eq!(debit.account_name(), "現金");
    assert_eq!(debit.amount_expression(), "1000");
}

#[test]
fn 科目名はスラッシュと改行以外のUnicode文字を許可する() {
    let description =
        JournalEntryDescription::parse("Cash＆預金-01 1000 /\n").expect("文字種による制限はない");

    let debit = description.debits().next().expect("借方明細が存在する");

    assert_eq!(debit.account_name(), "Cash＆預金-01");
}
