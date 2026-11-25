use periodless_core::{ValidationError, ValidationWarning};
use periodless_journal_dsl::JournalEntryDescription;

#[test]
fn 省略行が先頭の場合は違反となる() -> Result<(), Box<dyn std::error::Error>> {
    let description = JournalEntryDescription::parse("   / 現金 10,000\n")?;

    let errors = description
        .validate()
        .expect_err("省略行より前に両側の行が存在しない");

    assert_eq!(
        errors.into_errors(),
        vec![ValidationError::OmissionWithoutLeadingCompleteLine { line: 1 }]
    );

    Ok(())
}

#[test]
fn 省略行が先頭で後に両側行が続く場合も違反となる() -> Result<(), Box<dyn std::error::Error>> {
    let description = JournalEntryDescription::parse(
        "          / 現金  10,000\n旅費交通費 90,000 / 仮払金 100,000\n",
    )?;

    let errors = description
        .validate()
        .expect_err("最初の省略行より前に両側の行が存在しない");

    assert_eq!(
        errors.into_errors(),
        vec![ValidationError::OmissionWithoutLeadingCompleteLine { line: 1 }]
    );

    Ok(())
}

#[test]
fn 両側の行が先頭の場合は違反とならない() -> Result<(), Box<dyn std::error::Error>> {
    let description = JournalEntryDescription::parse(
        "旅費交通費 90,000 / 仮払金 100,000\n          / 現金  10,000\n",
    )?;

    let warnings = description.validate()?;

    assert!(warnings.is_empty());

    Ok(())
}

#[test]
fn 科目名の末尾が数字の場合は警告となる() -> Result<(), Box<dyn std::error::Error>> {
    let description = JournalEntryDescription::parse("現金 1,000 / 売上123 1,000\n")?;

    let warnings = description.validate()?;

    assert_eq!(
        warnings,
        vec![ValidationWarning::AccountNameEndsWithDigit {
            account_name: "売上123".to_owned()
        }]
    );

    Ok(())
}

#[test]
fn 科目名の末尾が数字でない場合は警告とならない() -> Result<(), Box<dyn std::error::Error>> {
    let 先頭が数字 = JournalEntryDescription::parse("現金 1,000 / 123売上 1,000\n")?;
    let 途中に数字 = JournalEntryDescription::parse("現金 1,000 / 第1営業部売上 1,000\n")?;

    assert!(先頭が数字.validate()?.is_empty());
    assert!(途中に数字.validate()?.is_empty());

    Ok(())
}

#[test]
fn 桁区切り文字が連続する場合は違反となる() -> Result<(), Box<dyn std::error::Error>> {
    let ピリオド連続 = JournalEntryDescription::parse("現金 1..5 / 売上 5\n")?;
    let カンマ連続 = JournalEntryDescription::parse("現金 1,,000 / 売上 1,000\n")?;

    for description in [&ピリオド連続, &カンマ連続] {
        let errors = description
            .validate()
            .expect_err("桁区切り文字が連続している");

        assert_eq!(
            errors.into_errors(),
            vec![ValidationError::ConsecutiveSeparators {
                amount_expression: description
                    .debits()
                    .next()
                    .expect("借方明細が存在する")
                    .amount_expression()
                    .to_owned()
            }]
        );
    }

    Ok(())
}

#[test]
fn 桁区切り文字以外の印字可能文字が含まれる場合は違反となる()
-> Result<(), Box<dyn std::error::Error>> {
    for 表現 in ["1e12", "1E3", "1.23e-4"] {
        let description = JournalEntryDescription::parse(&format!("現金 {表現} / 売上 1,000\n"))?;

        let errors = description
            .validate()
            .expect_err("桁区切り文字以外の印字可能ASCII文字が含まれる");

        assert_eq!(
            errors.into_errors(),
            vec![ValidationError::NonCanonicalSeparator {
                amount_expression: 表現.to_owned(),
                character: 表現
                    .chars()
                    .find(|character| {
                        !character.is_ascii_digit() && !matches!(character, '_' | '\'' | '.' | ',')
                    })
                    .expect("非正準の文字が含まれる")
            }]
        );
    }

    Ok(())
}

#[test]
fn 数字で終わらない金額表現は構文違反として拒否される() {
    let 検証結果 = JournalEntryDescription::parse("現金 1.234e / 売上 1,000\n");

    assert!(検証結果.is_err());
}

#[test]
fn 基数プレフィックス表記は違反となる() -> Result<(), Box<dyn std::error::Error>> {
    for 表現 in ["0b111", "0x12345", "0o12345"] {
        let description = JournalEntryDescription::parse(&format!("現金 {表現} / 売上 123\n"))?;

        let errors = description
            .validate()
            .expect_err("基数プレフィックス表記と混同される");

        assert_eq!(
            errors.into_errors(),
            vec![
                ValidationError::NonCanonicalSeparator {
                    amount_expression: 表現.to_owned(),
                    character: 表現.chars().nth(1).expect("基数文字が存在する"),
                },
                ValidationError::RadixPrefixedAmount {
                    amount_expression: 表現.to_owned()
                }
            ]
        );
    }

    Ok(())
}

#[test]
fn ゼロ埋めされた金額表現は警告となる() -> Result<(), Box<dyn std::error::Error>> {
    for 表現 in ["000", "0123"] {
        let description = JournalEntryDescription::parse(&format!("現金 {表現} / 売上 123\n"))?;

        let warnings = description.validate()?;

        assert_eq!(
            warnings,
            vec![ValidationWarning::ZeroPaddedAmount {
                amount_expression: 表現.to_owned()
            }]
        );
    }

    Ok(())
}

#[test]
fn 一桁のゼロは警告とならない() -> Result<(), Box<dyn std::error::Error>> {
    let description = JournalEntryDescription::parse("現金 0 / 売上 0\n")?;

    assert!(description.validate()?.is_empty());

    Ok(())
}

#[test]
fn 正準の桁区切り文字のみの表現は違反とならない() -> Result<(), Box<dyn std::error::Error>> {
    for 表現 in ["1,000", "1_000", "1.000", "1'000"] {
        let description = JournalEntryDescription::parse(&format!("現金 {表現} / 売上 1,000\n"))?;

        assert!(description.validate()?.is_empty());
    }

    Ok(())
}

#[test]
fn 検出したすべての違反が報告される() -> Result<(), Box<dyn std::error::Error>> {
    let description = JournalEntryDescription::parse("現金 0b111 / 売上 111\n現金 0x123 /\n")?;

    let errors = description.validate().expect_err("複数の違反が検出される");

    assert_eq!(errors.errors().len(), 4);

    Ok(())
}
