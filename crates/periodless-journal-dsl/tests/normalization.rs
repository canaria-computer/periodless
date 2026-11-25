use periodless_journal_dsl::JournalEntryDescription;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn 一明細ずつで両側が完全一致する仕訳記述は同一の取引とみなされる()
-> Result<(), Box<dyn std::error::Error>> {
    let 一方 = JournalEntryDescription::parse("現金 100 / 売上 100\n")?;
    let 他方 = JournalEntryDescription::parse("現金 100 / 売上 100\n")?;

    assert_eq!(一方, 他方);

    Ok(())
}

#[test]
fn 両側行の行順を入れ替えた仕訳記述は同一の取引とみなされる()
-> Result<(), Box<dyn std::error::Error>> {
    let 一方 = JournalEntryDescription::parse("現金 100 / 売上 100\n備品 200 / 雑収入 200\n")?;
    let 他方 = JournalEntryDescription::parse("備品 200 / 雑収入 200\n現金 100 / 売上 100\n")?;

    assert_eq!(一方, 他方);

    Ok(())
}

#[test]
fn 両側行の左右の組み合わせのみが異なる仕訳記述は同一の取引とみなされる()
-> Result<(), Box<dyn std::error::Error>> {
    let 一方 = JournalEntryDescription::parse("現金 100 / 売上 100\n備品 200 / 雑収入 200\n")?;
    let 他方 = JournalEntryDescription::parse("現金 100 / 雑収入 200\n備品 200 / 売上 100\n")?;

    assert_eq!(一方, 他方);

    Ok(())
}

#[test]
fn 単独行を含む順序の違いのみの仕訳記述は同一の取引とみなされる()
-> Result<(), Box<dyn std::error::Error>> {
    let 一方 = JournalEntryDescription::parse(
        "旅費交通費 90,000 / 仮払金 100,000\n          / 現金  10,000\n",
    )?;
    let 他方 = JournalEntryDescription::parse(
        "旅費交通費 90,000 / 現金  10,000\n          / 仮払金 100,000\n",
    )?;

    assert_eq!(一方, 他方);

    Ok(())
}

#[test]
fn 金額表現の表記の違いは等価性に影響しない() -> Result<(), Box<dyn std::error::Error>> {
    let カンマ区切り = JournalEntryDescription::parse("現金 1,000 / 売上 1,000\n")?;
    let 区切りなし = JournalEntryDescription::parse("現金 1000 / 売上 1000\n")?;
    let 先頭ゼロ = JournalEntryDescription::parse("現金 001,000 / 売上 0001000\n")?;

    assert_eq!(カンマ区切り, 区切りなし);
    assert_eq!(カンマ区切り, 先頭ゼロ);

    Ok(())
}

#[test]
fn 明細の内容が異なる仕訳記述は同一とみなされない() -> Result<(), Box<dyn std::error::Error>> {
    let 金額が異なる = JournalEntryDescription::parse("現金 100 / 売上 100\n")?;
    let 科目が異なる = JournalEntryDescription::parse("現金 100 / 雑収入 100\n")?;

    assert_ne!(金額が異なる, 科目が異なる);

    let 一方 = JournalEntryDescription::parse("現金 100 / 売上 100\n")?;
    let 借方が異なる = JournalEntryDescription::parse("備品 100 / 売上 100\n")?;

    assert_ne!(一方, 借方が異なる);

    Ok(())
}

#[test]
fn 側の違いは等価性に影響する() -> Result<(), Box<dyn std::error::Error>> {
    let 現金が借方 = JournalEntryDescription::parse("現金 100 / 売上 100\n")?;
    let 現金が貸方 = JournalEntryDescription::parse("売上 100 / 現金 100\n")?;

    assert_ne!(現金が借方, 現金が貸方);

    Ok(())
}

#[test]
fn 等価な仕訳記述は同一のハッシュになる() -> Result<(), Box<dyn std::error::Error>> {
    let 一方 = JournalEntryDescription::parse("現金 100 / 売上 100\n備品 200 / 雑収入 200\n")?;
    let 他方 = JournalEntryDescription::parse("備品 200 / 雑収入 200\n現金 100 / 売上 100\n")?;

    let 一方のハッシュ = {
        let mut hasher = DefaultHasher::new();
        一方.hash(&mut hasher);
        hasher.finish()
    };
    let 他方のハッシュ = {
        let mut hasher = DefaultHasher::new();
        他方.hash(&mut hasher);
        hasher.finish()
    };

    assert_eq!(一方のハッシュ, 他方のハッシュ);

    Ok(())
}

#[test]
fn 整形出力は入力順を保持する() -> Result<(), Box<dyn std::error::Error>> {
    use periodless_journal_dsl::JournalEntryPrinter;

    let description =
        JournalEntryDescription::parse("現金 100 / 売上 100\n備品 200 / 雑収入 200\n")?;
    let 逆順 = JournalEntryDescription::parse("備品 200 / 雑収入 200\n現金 100 / 売上 100\n")?;

    let 印刷 = JournalEntryPrinter::new().print(&description);
    let 逆順の印刷 = JournalEntryPrinter::new().print(&逆順);

    assert!(印刷.starts_with("現金"));
    assert!(逆順の印刷.starts_with("備品"));
    assert_ne!(印刷, 逆順の印刷);

    assert_eq!(
        JournalEntryPrinter::new().print(&JournalEntryDescription::parse(&印刷)?),
        印刷
    );

    Ok(())
}

#[test]
fn 正規化順は金額降順・文字列長降順・辞書順降順である() -> Result<(), Box<dyn std::error::Error>> {
    let 同一金額で科目名が異なる = JournalEntryDescription::parse("現金 100 / 現金過不足 100\n")?;

    let 借方の科目名: Vec<&str> = 同一金額で科目名が異なる
        .debits()
        .map(|debit| debit.account_name())
        .collect();

    assert_eq!(借方の科目名, vec!["現金"]);

    Ok(())
}
