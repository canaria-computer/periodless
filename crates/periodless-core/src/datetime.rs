//! 取引日時とコミット日時の正規化。
//!
//! [`TransactionAt`]（取引日時）・[`CommittedAt`]（コミット日時）とも、構築時に
//! ミリ秒境界への切り上げ正規化を施した正規化済み瞬間を保持する。正準文字列は
//! `YYYYMMDDTHHMMSS.sssZ`（年4桁・月日時分秒2桁・ミリ秒3桁・大文字T/Z）で、
//! 秒は00〜59の範囲（うるう秒のSS=60は表現しない）。
//!
//! # Examples
//!
//! ```
//! use periodless_core::{CommittedAt, TransactionAt};
//!
//! let 取引日時 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123001Z")?;
//! let コミット日時 = CommittedAt::parse_rfc3339("2026-11-13T02:20:00Z")?;
//!
//! assert_eq!(取引日時.canonical_string(), "20261113T021900.124Z");
//! assert_eq!(コミット日時.canonical_string(), "20261113T022000.000Z");
//! assert!(取引日時.canonical_string() < コミット日時.canonical_string());
//! # Ok::<(), periodless_core::NormalizationError>(())
//! ```
//!
//! 秒として60（うるう秒）を与えた場合は拒否する:
//!
//! ```
//! use periodless_core::{NormalizationError, TransactionAt};
//!
//! assert_eq!(
//!     TransactionAt::parse_rfc3339("2016-12-31T23:59:60Z"),
//!     Err(NormalizationError::LeapSecondUnsupported)
//! );
//! ```

use core::fmt::{Display, Formatter, Result as FormatResult};
use hifitime::{Epoch, Unit};

/// 日時の正規化に違反があったことを表すエラー。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationError {
    /// 入力日時をRFC3339として解析できなかった。
    Unparsable,
    /// 切り上げの結果が暦として表現できない。
    InvalidCalendar,
    /// 入力日時の秒として60（うるう秒）が与えられた。正準表現で区別できないため拒否する。
    LeapSecondUnsupported,
    /// 切り上げによって、年4桁（0000〜9999）で表せる範囲を超えた。
    YearOutOfRange { year: i32 },
}

impl Display for NormalizationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self {
            NormalizationError::Unparsable => {
                write!(formatter, "入力日時をRFC3339として解析できません")
            }
            NormalizationError::InvalidCalendar => {
                write!(formatter, "切り上げの結果が暦として表現できません")
            }
            NormalizationError::LeapSecondUnsupported => {
                write!(
                    formatter,
                    "秒60（うるう秒）の入力は正準表現で区別できないため受理できません"
                )
            }
            NormalizationError::YearOutOfRange { year } => {
                write!(
                    formatter,
                    "切り上げの結果が年の範囲 0000〜9999 を超えています: {year}"
                )
            }
        }
    }
}

impl core::error::Error for NormalizationError {}

/// ミリ秒境界への切り上げ正規化を施したUTCの瞬間。
///
/// 正規化は、入力日時以上となるミリ秒境界のうち最も早い時刻へ切り上げる。
/// 入力が既にミリ秒境界に一致する場合は値を変更しない。ミリ秒未満の端数は、
/// 判定前に切り捨てずに入力が保持する精度で扱う。切り上げによる繰り上がりは
/// 暦に従い、うるう秒挿入日の UTC 23:59:60 も有効な暦秒として保持する。
///
/// 正準文字列表現は `YYYYMMDDTHHMMSS.sssZ`（年4桁・月日時分秒2桁・ミリ秒3桁）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NormalizedInstant(Epoch);

impl NormalizedInstant {
    fn from_epoch(epoch: Epoch) -> Result<Self, NormalizationError> {
        let normalized = normalize_to_millisecond(epoch)?;

        Ok(Self(normalized))
    }

    fn parse_rfc3339(text: &str) -> Result<Self, NormalizationError> {
        reject_leap_second_input(text)?;

        let epoch = Epoch::from_gregorian_str(text).map_err(|_| NormalizationError::Unparsable)?;

        Self::from_epoch(epoch)
    }

    fn from_gregorian_utc(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Self, NormalizationError> {
        if second == 60 {
            return Err(NormalizationError::LeapSecondUnsupported);
        }

        let epoch =
            Epoch::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanosecond)
                .map_err(|_| NormalizationError::InvalidCalendar)?;

        Self::from_epoch(epoch)
    }

    fn canonical_string(&self) -> String {
        let (year, month, day, hour, minute, second, nanosecond) = self.0.to_gregorian_utc();

        format!(
            "{year:04}{month:02}{day:02}T{hour:02}{minute:02}{second:02}.{:03}Z",
            nanosecond / 1_000_000
        )
    }
}

fn normalize_to_millisecond(epoch: Epoch) -> Result<Epoch, NormalizationError> {
    let (year, month, day, hour, minute, second, nanosecond) = epoch.to_gregorian_utc();
    let remainder = nanosecond % 1_000_000;

    let normalized = if remainder == 0 {
        Epoch::maybe_from_gregorian_utc(year, month, day, hour, minute, second, nanosecond)
            .map_err(|_| NormalizationError::InvalidCalendar)?
    } else {
        let floored = Epoch::maybe_from_gregorian_utc(
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond - remainder,
        )
        .map_err(|_| NormalizationError::InvalidCalendar)?;

        floored + Unit::Millisecond
    };

    let (normalized_year, month, day, hour, minute, second, nanosecond) =
        normalized.to_gregorian_utc();

    if second == 60 {
        return Err(NormalizationError::LeapSecondUnsupported);
    }

    if !(0..=9999).contains(&normalized_year) {
        return Err(NormalizationError::YearOutOfRange {
            year: normalized_year,
        });
    }

    Epoch::maybe_from_gregorian_utc(
        normalized_year,
        month,
        day,
        hour,
        minute,
        second,
        nanosecond,
    )
    .map_err(|_| NormalizationError::InvalidCalendar)
}

/// RFC3339形式の入力文字列の秒フィールドが60（うるう秒）である場合に拒否する。
///
/// うるう秒の瞬間は正準文字列で区別できないため、意図と1秒ずれた値が黙って
/// 記録されることを防ぐ。入力の時刻部は `HH:MM:SS` 形式であるため、2番目の
/// コロンの直後が `60` であるかどうかで判定する。
fn reject_leap_second_input(text: &str) -> Result<(), NormalizationError> {
    let seconds_field = text
        .split(':')
        .nth(2)
        .ok_or(NormalizationError::Unparsable)?;

    if seconds_field.starts_with("60") {
        return Err(NormalizationError::LeapSecondUnsupported);
    }

    Ok(())
}

macro_rules! normalized_datetime {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        ///
        /// 構築時にミリ秒境界への切り上げ正規化を施した正規化済み瞬間を保持する。
        /// システムは時計を持たないため、値は入力として与えられた日時のみから
        /// 決まる。秒として60（うるう秒）を与えた場合は正準表現で区別できない
        /// ため拒否する。
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(NormalizedInstant);

        impl $name {
            /// RFC3339形式の文字列から構築する。ミリ秒境界への切り上げ正規化を
            /// 施した正規化済み瞬間を保持する。
            ///
            /// # Errors
            ///
            /// 文字列をRFC3339として解析できない場合、または切り上げの結果が
            /// 年4桁（0000〜9999）の範囲を超える場合に
            /// [`NormalizationError`] を返す。
            pub fn parse_rfc3339(text: &str) -> Result<Self, NormalizationError> {
                Ok(Self(NormalizedInstant::parse_rfc3339(text)?))
            }

            /// UTCの暦日時から構築する。ミリ秒境界への切り上げ正規化を施した
            /// 正規化済み瞬間を保持する。秒として60（うるう秒）を与えた場合は
            /// 正準表現で区別できないため拒否する。
            ///
            /// # Errors
            ///
            /// 暦として表現できない日時である場合、秒として60が与えられた場合、
            /// または切り上げの結果が年4桁（0000〜9999）の範囲を超える場合に
            /// [`NormalizationError`] を返す。
            pub fn from_gregorian_utc(
                year: i32,
                month: u8,
                day: u8,
                hour: u8,
                minute: u8,
                second: u8,
                nanosecond: u32,
            ) -> Result<Self, NormalizationError> {
                Ok(Self(NormalizedInstant::from_gregorian_utc(
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    second,
                    nanosecond,
                )?))
            }

            /// 正準文字列 `YYYYMMDDTHHMMSS.sssZ`。
            ///
            /// 年は4桁、月日および時分秒は2桁、秒の小数部は3桁でゼロ埋めする。
            /// 日時の区切りには大文字の `T` を使用し、UTCを示す大文字の `Z`
            /// を末尾に付加する。
            pub fn canonical_string(&self) -> String {
                self.0.canonical_string()
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
                formatter.write_str(&self.0.canonical_string())
            }
        }
    };
}

normalized_datetime!(
    /// 取引日時。仕訳が表す取引の発生日時。
    ///
    /// 取引日時には単調性を適用しないため、過去の日時を自由に記録できる。
    TransactionAt
);

normalized_datetime!(
    /// コミット日時。仕訳を台帳へ追記した日時。
    ///
    /// 台帳は、追記する仕訳のコミット日時が直近に記録された仕訳のコミット日時
    /// より前である場合に追記を拒否する。等しい場合は受理し、順序は到達順序に
    /// よる。
    CommittedAt
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ミリ秒未満の端数は切り上げられる() -> Result<(), NormalizationError> {
        let 日時 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123001Z")?;

        assert_eq!(日時.canonical_string(), "20261113T021900.124Z");

        Ok(())
    }

    #[test]
    fn ミリ秒境界に一致する入力は変更されない() -> Result<(), NormalizationError> {
        let 日時 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123000Z")?;

        assert_eq!(日時.canonical_string(), "20261113T021900.123Z");

        Ok(())
    }

    #[test]
    fn 小数桁が3桁未満の入力は時刻を進めずゼロ埋めされる() -> Result<(), NormalizationError> {
        let 日時 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.1Z")?;

        assert_eq!(日時.canonical_string(), "20261113T021900.100Z");

        Ok(())
    }

    #[test]
    fn 日境界を超える繰り上がりは暦に従う() -> Result<(), NormalizationError> {
        let 日時 = TransactionAt::parse_rfc3339("2026-11-13T23:59:59.999001Z")?;

        assert_eq!(日時.canonical_string(), "20261114T000000.000Z");

        Ok(())
    }

    #[test]
    fn 月境界と年境界を超える繰り上がりは暦に従う() -> Result<(), NormalizationError> {
        let 月またぎ = TransactionAt::parse_rfc3339("2026-01-31T23:59:59.999001Z")?;
        let 年またぎ = TransactionAt::parse_rfc3339("2026-12-31T23:59:59.999001Z")?;

        assert_eq!(月またぎ.canonical_string(), "20260201T000000.000Z");
        assert_eq!(年またぎ.canonical_string(), "20270101T000000.000Z");

        Ok(())
    }

    #[test]
    fn うるう秒の秒六十の入力は拒否される() {
        assert_eq!(
            TransactionAt::parse_rfc3339("2016-12-31T23:59:60Z"),
            Err(NormalizationError::LeapSecondUnsupported)
        );
        assert_eq!(
            TransactionAt::parse_rfc3339("2016-12-31T23:59:60.500Z"),
            Err(NormalizationError::LeapSecondUnsupported)
        );
        assert_eq!(
            TransactionAt::from_gregorian_utc(2016, 12, 31, 23, 59, 60, 0),
            Err(NormalizationError::LeapSecondUnsupported)
        );
    }

    #[test]
    fn うるう秒の境界をまたぐ切り上げは翌日の零時となる() -> Result<(), NormalizationError> {
        let 日時 = TransactionAt::parse_rfc3339("2016-12-31T23:59:59.999001Z")?;

        assert_eq!(日時.canonical_string(), "20170101T000000.000Z");

        Ok(())
    }

    #[test]
    fn 年の範囲を超える繰り上がりは正規化エラーを返す() {
        let 最大年 = TransactionAt::from_gregorian_utc(9999, 12, 31, 23, 59, 59, 999_999_999);

        assert_eq!(
            最大年,
            Err(NormalizationError::YearOutOfRange { year: 10_000 })
        );
    }

    #[test]
    fn 解析できない入力は正規化エラーを返す() {
        assert_eq!(
            TransactionAt::parse_rfc3339("これは日時ではない"),
            Err(NormalizationError::Unparsable)
        );
    }

    #[test]
    fn 正規化は冪等である() -> Result<(), NormalizationError> {
        let 一回目 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123001Z")?;
        let (year, month, day, hour, minute, second, nanosecond) = hifitime_epoch_parts(&一回目);

        let 二回目 =
            TransactionAt::from_gregorian_utc(year, month, day, hour, minute, second, nanosecond)?;

        assert_eq!(一回目, 二回目);

        Ok(())
    }

    #[test]
    fn 同一入力は常に同一の正準文字列を返す() -> Result<(), NormalizationError> {
        let 一回目 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123001Z")?;
        let 二回目 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123001Z")?;

        assert_eq!(一回目.canonical_string(), 二回目.canonical_string());
        assert_eq!(一回目, 二回目);

        Ok(())
    }

    #[test]
    fn 同一ミリ秒境界への切り上げは等価になる() -> Result<(), NormalizationError> {
        let 端数あり = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123001Z")?;
        let 境界一致 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.124000Z")?;

        assert_eq!(端数あり, 境界一致);

        Ok(())
    }

    #[test]
    fn 順序は正規化済み瞬間に従う() -> Result<(), NormalizationError> {
        let 早い = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123Z")?;
        let 遅い = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.125Z")?;
        let 同じ = TransactionAt::parse_rfc3339("2026-11-13T02:19:00.124999Z")?;

        assert!(早い < 遅い);
        assert_eq!(遅い, 同じ);

        Ok(())
    }

    #[test]
    fn 取引日時とコミット日時は別の型である() -> Result<(), NormalizationError> {
        let 取引 = TransactionAt::parse_rfc3339("2026-11-13T02:19:00Z")?;
        let コミット = CommittedAt::parse_rfc3339("2026-11-13T02:19:00Z")?;

        assert_eq!(取引.canonical_string(), コミット.canonical_string());

        Ok(())
    }

    fn hifitime_epoch_parts(日時: &TransactionAt) -> (i32, u8, u8, u8, u8, u8, u32) {
        日時.0.0.to_gregorian_utc()
    }
}
