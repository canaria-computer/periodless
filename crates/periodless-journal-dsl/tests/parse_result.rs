#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::{DescribedDetail, JournalEntryDescription};

fn 科目名と金額表現<'a>(
    明細: impl Iterator<Item = &'a DescribedDetail>,
) -> Vec<(&'a str, &'a str)> {
    明細
        .map(|detail| (detail.account_name(), detail.amount_expression()))
        .collect()
}

#[test]
fn 給料の支払の例を借方一件と貸方三件に分解する() {
    let description = JournalEntryDescription::parse(
        "給料 400000 / 所得税預り金 30000\n   /社会保険料預り金 20000\n   /現金 350000\n",
    )
    .expect("仕様の入力例は受理される");

    assert_eq!(
        科目名と金額表現(description.debits()),
        vec![("給料", "400000")]
    );
    assert_eq!(
        科目名と金額表現(description.credits()),
        vec![
            ("所得税預り金", "30000"),
            ("社会保険料預り金", "20000"),
            ("現金", "350000"),
        ]
    );
}

#[test]
fn 同一科目の複数出現を集約せず保持する() {
    let description = JournalEntryDescription::parse("現金 100 / 売上 100\n現金 200 / 売上 200\n")
        .expect("同一科目の重複は構文上受理される");

    assert_eq!(
        科目名と金額表現(description.debits()),
        vec![("現金", "100"), ("現金", "200")]
    );
    assert_eq!(
        科目名と金額表現(description.credits()),
        vec![("売上", "100"), ("売上", "200")]
    );
}

#[test]
fn 金額表現は元の表記のまま保持される() {
    let description =
        JournalEntryDescription::parse("現金 1,0;00 / 売上 000\n").expect("桁区切りは受理される");

    assert_eq!(
        科目名と金額表現(description.debits()),
        vec![("現金", "1,0;00")]
    );
    assert_eq!(
        科目名と金額表現(description.credits()),
        vec![("売上", "000")]
    );
}

#[test]
fn 科目名を正規化しない() {
    let description = JournalEntryDescription::parse("cash CASH  ｷｬｯｼｭ 1000 /\n")
        .expect("科目名の表記差は保持される");

    assert_eq!(
        科目名と金額表現(description.debits()),
        vec![("cash CASH  ｷｬｯｼｭ", "1000")]
    );
}

#[test]
fn 同一行の借方と貸方に対応関係を持たせない() {
    let description = JournalEntryDescription::parse("現金 600 / 売上 1000\n売掛金 400 /\n")
        .expect("左右の件数が異なる入力は受理される");

    assert_eq!(
        科目名と金額表現(description.debits()),
        vec![("現金", "600"), ("売掛金", "400")]
    );
    assert_eq!(
        科目名と金額表現(description.credits()),
        vec![("売上", "1000")]
    );
}

#[test]
fn 貸借不一致の入力を構文上受理する() {
    let description =
        JournalEntryDescription::parse("現金 100 / 売上 999\n").expect("貸借一致は検証しない");

    assert_eq!(description.debits().count(), 1);
    assert_eq!(description.credits().count(), 1);
}

#[test]
fn 片側のみで構成された入力を構文上受理する() {
    let 借方のみ = JournalEntryDescription::parse("現金 100 /\n").expect("双方の存在は要求しない");
    let 貸方のみ = JournalEntryDescription::parse("/ 売上 100\n").expect("双方の存在は要求しない");

    assert_eq!(借方のみ.credits().count(), 0);
    assert_eq!(貸方のみ.debits().count(), 0);
}

#[test]
fn 整数型に収まらない桁数の金額表現を構文上受理する() {
    let 巨大な金額表現 = "9".repeat(40);
    let description = JournalEntryDescription::parse(&format!("現金 {巨大な金額表現} /\n"))
        .expect("値の範囲は検証しない");

    assert_eq!(
        科目名と金額表現(description.debits()),
        vec![("現金", 巨大な金額表現.as_str())]
    );
}
