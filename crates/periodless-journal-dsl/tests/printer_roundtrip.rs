#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::{JournalEntryDescription, JournalEntryPrinter};

fn 解析(description: &str) -> JournalEntryDescription {
    JournalEntryDescription::parse(description).expect("入力は受理される")
}

fn 整形(description: &JournalEntryDescription) -> String {
    JournalEntryPrinter::new().print(description)
}

const 入力: &str =
    "給料 400000 / 所得税預り金 30000\n   /社会保険料預り金 20,000\n   /現金 350000\n";

#[test]
fn 整形出力は再解析でき科目名が一致する() {
    let 元 = 解析(入力);
    let 再解析 = 解析(&整形(&元));

    let 元の科目名: Vec<&str> = 元
        .debits()
        .chain(元.credits())
        .map(|明細| 明細.account_name())
        .collect();
    let 再解析の科目名: Vec<&str> = 再解析
        .debits()
        .chain(再解析.credits())
        .map(|明細| 明細.account_name())
        .collect();

    assert_eq!(再解析の科目名, 元の科目名);
    assert_eq!(再解析.debits().count(), 元.debits().count());
    assert_eq!(再解析.credits().count(), 元.credits().count());
}

#[test]
fn 整形は冪等である() {
    let 一回目 = 整形(&解析(入力));
    let 二回目 = 整形(&解析(&一回目));

    assert_eq!(二回目, 一回目);
}

#[test]
fn 整形の前後で金額の整数値が保たれる() {
    let 元 = 解析(入力);
    let 再解析 = 解析(&整形(&元));

    let 整数値 = |description: &JournalEntryDescription| -> Vec<i64> {
        description
            .debits()
            .chain(description.credits())
            .map(|明細| 明細.amount_as_integer().expect("範囲内の金額表現"))
            .collect()
    };

    assert_eq!(整数値(&再解析), 整数値(&元));
}

#[test]
fn 科目名内部の水平空白は整形と再解析を越えて保たれる() {
    let 元 = 解析("売買目的  有価証券 1000 / 現金 1000\n");
    let 再解析 = 解析(&整形(&元));

    assert_eq!(
        再解析
            .debits()
            .next()
            .expect("借方明細が存在する")
            .account_name(),
        "売買目的  有価証券"
    );
}

#[test]
fn 最後の空白区切り部分が金額になる科目名も整形と再解析を越えて保たれる() {
    let 元 = 解析("科目 1 000 / 現金 100\n");
    let 再解析 = 解析(&整形(&元));
    let 借方 = 再解析.debits().next().expect("借方明細が存在する");

    assert_eq!(借方.account_name(), "科目 1");
    assert_eq!(借方.amount_expression(), "0");
}

#[test]
fn Display実装は既定の整形結果を返す() {
    let 元 = 解析(入力);

    assert_eq!(元.to_string(), 整形(&元));
}
