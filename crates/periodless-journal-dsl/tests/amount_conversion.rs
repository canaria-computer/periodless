#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::JournalEntryDescription;

fn 借方の整数値(金額表現: &str) -> Option<Result<i64, impl std::error::Error>> {
    let description = JournalEntryDescription::parse(&format!("現金 {金額表現} /\n")).ok()?;
    let debit = description.debits().next()?;

    Some(debit.amount_as_integer())
}

#[test]
fn 桁区切り文字を除去して十進整数として解釈する() {
    for (金額表現, 整数値) in [
        ("000", 0),
        ("0", 0),
        ("1000", 1000),
        ("1,000", 1000),
        ("1.000", 1000),
        ("1;000", 1000),
        ("1,,000", 1000),
        ("1,0;00", 1000),
        ("1e3", 13),
    ] {
        let 変換結果 = 借方の整数値(金額表現).expect("金額表現は解析できる");

        assert_eq!(変換結果.ok(), Some(整数値), "金額表現: {金額表現}");
    }
}

#[test]
fn 指数表記や小数表記として解釈しない() {
    let 指数 = 借方の整数値("1e3").expect("解析できる");
    let 小数 = 借方の整数値("1.5").expect("解析できる");

    assert_eq!(指数.ok(), Some(13));
    assert_eq!(小数.ok(), Some(15));
}

#[test]
fn 整数型の上限値を受理する() {
    let 上限 = i64::MAX.to_string();
    let 変換結果 = 借方の整数値(&上限).expect("解析できる");

    assert_eq!(変換結果.ok(), Some(i64::MAX));
}

#[test]
fn 整数型の範囲を超える金額表現は範囲超過として報告される() {
    let 巨大な金額表現 = "9".repeat(40);
    let 変換結果 = 借方の整数値(&巨大な金額表現).expect("解析できる");

    assert!(変換結果.is_err());
}

#[test]
fn 範囲超過の金額表現でも構文解析は成功する() {
    let 巨大な金額表現 = "9".repeat(40);
    let description = JournalEntryDescription::parse(&format!("現金 {巨大な金額表現} /\n"))
        .expect("構文解析は値の範囲を判定しない");

    assert_eq!(
        description
            .debits()
            .next()
            .expect("借方明細が存在する")
            .amount_expression(),
        巨大な金額表現
    );
}

#[test]
fn 変換結果が負になることはない() {
    for 金額表現 in ["1-000", "0-0", "000"] {
        let 変換結果 = 借方の整数値(金額表現).expect("解析できる");

        assert!(変換結果.ok().is_some_and(|整数値| 整数値 >= 0));
    }
}
