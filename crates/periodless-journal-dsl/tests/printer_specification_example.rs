#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::JournalEntryDescription;
use unicode_width::UnicodeWidthStr;

const 入力: &str =
    "給料 400000 / 所得税預り金 30000\n   /社会保険料預り金 20000\n   /現金 350000\n";

const 正規形: &str = "給料 400,000 / 所得税預り金      30,000\n             / 社会保険料預り金  20,000\n             / 現金             350,000\n";

#[test]
fn 給料の支払の例を正規形へ整形する() {
    let description = JournalEntryDescription::parse(入力).expect("仕様の入力例は受理される");

    assert_eq!(description.to_string(), 正規形);
}

#[test]
fn 正規形はスラッシュと金額の右端が揃っている() {
    let 行: Vec<&str> = 正規形.lines().collect();

    let スラッシュまでの幅: Vec<usize> = 行
        .iter()
        .map(|行| 行[..行.find('/').expect("各行はスラッシュを持つ")].width())
        .collect();
    let 行の幅: Vec<usize> = 行.iter().map(|行| 行.width()).collect();

    assert!(スラッシュまでの幅.windows(2).all(|隣接| 隣接[0] == 隣接[1]));
    assert!(行の幅.windows(2).all(|隣接| 隣接[0] == 隣接[1]));
}

#[test]
fn 正規形は再解析しても同じ正規形へ整形される() {
    let 再解析 = JournalEntryDescription::parse(正規形).expect("正規形は再解析できる");

    assert_eq!(再解析.to_string(), 正規形);
    assert_eq!(再解析.debits().count(), 1);
    assert_eq!(再解析.credits().count(), 3);
}
