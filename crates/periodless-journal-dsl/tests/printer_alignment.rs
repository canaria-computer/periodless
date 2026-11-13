#![allow(non_ascii_idents, non_snake_case)]

use periodless_journal_dsl::{JournalEntryDescription, JournalEntryPrinter};
use unicode_width::UnicodeWidthStr;

fn 整形(description: &str) -> String {
    let parsed = JournalEntryDescription::parse(description).expect("入力は受理される");

    JournalEntryPrinter::new().print(&parsed)
}

fn スラッシュまでの表示幅(行: &str) -> usize {
    let スラッシュ位置 = 行.find('/').expect("各行はスラッシュを持つ");

    行[..スラッシュ位置].width()
}

#[test]
fn 全角と半角が混在する科目名でもスラッシュ位置が揃う() {
    let 整形結果 = 整形("現金 100 / 売上 100\nCash 200 / 売上 200\n売掛金 300 / 売上 300\n");

    let 幅: Vec<usize> = 整形結果.lines().map(スラッシュまでの表示幅).collect();

    assert_eq!(幅.len(), 3);
    assert!(
        幅.windows(2).all(|隣接| 隣接[0] == 隣接[1]),
        "スラッシュ位置が揃っていない: {整形結果:?}"
    );
}

#[test]
fn 借方が欠けた行でもスラッシュ位置が揃う() {
    let 整形結果 = 整形("給料 500 / 預り金 100\n/ 現金 400\n");

    let 幅: Vec<usize> = 整形結果.lines().map(スラッシュまでの表示幅).collect();

    assert_eq!(幅.len(), 2);
    assert_eq!(幅[0], 幅[1], "スラッシュ位置が揃っていない: {整形結果:?}");
}

#[test]
fn 科目名を左揃えにし金額を右揃えにする() {
    let 整形結果 = 整形(
        "現金 100 / 売上 500
売掛金 20 / 売上 20
",
    );

    assert_eq!(
        整形結果,
        "現金   100 / 売上 500
売掛金  20 / 売上  20
"
    );
}

#[test]
fn 全角科目名の桁揃えは表示幅で計算される() {
    let 整形結果 = 整形("現金 100 / 売上 100\nCash 200 / 売上 200\n");

    assert_eq!(整形結果, "現金 100 / 売上 100\nCash 200 / 売上 200\n");
}
