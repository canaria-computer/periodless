use core::fmt::{Display, Formatter, Result as FormatResult};

/// 勘定金額の構築時に違反した不変条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountError {
    /// 金額が負である。
    Negative { sen: i64 },
    /// 算術の結果が符号付き64bit整数の範囲を超えた。
    Overflow,
}

impl Display for AmountError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self {
            AmountError::Negative { sen } => {
                write!(formatter, "金額 {sen} 銭は負であるため保持できません")
            }
            AmountError::Overflow => write!(
                formatter,
                "金額の算術結果が符号付き64bit整数の範囲を超えています"
            ),
        }
    }
}

impl core::error::Error for AmountError {}

/// 勘定金額。1銭単位（0.01円）の非負整数として保持する。
///
/// 金額は浮動小数点を使用せず、符号付き64bit整数の銭単位で保持する。割合計算
/// （百分率・按分など）に伴う丸め誤差は、1銭単位の整数演算によって発生しない。
///
/// # Examples
///
/// ```
/// use periodless_core::Amount;
///
/// let 一千円五十銭 = Amount::new(100_050)?;
///
/// assert_eq!(一千円五十銭.to_yen(), (1_000, 50));
/// assert_eq!(Amount::from_yen(3_500)?.as_sen(), 350_000);
/// # Ok::<(), periodless_core::AmountError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Amount(i64);

impl Amount {
    /// 保持する値が0銭である金額。
    pub const ZERO: Self = Self(0);

    /// 1銭単位の値から金額を構築する。負の値は拒否する。
    ///
    /// # Errors
    ///
    /// `sen` が負である場合に [`AmountError::Negative`] を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_core::Amount;
    ///
    /// assert_eq!(Amount::new(100_050)?.to_yen(), (1_000, 50));
    /// # Ok::<(), periodless_core::AmountError>(())
    /// ```
    pub fn new(sen: i64) -> Result<Self, AmountError> {
        if sen < 0 {
            return Err(AmountError::Negative { sen });
        }

        Ok(Self(sen))
    }

    /// 円単位の整数から金額を構築する。100倍した銭単位で保持する。
    ///
    /// # Errors
    ///
    /// `yen` が負である場合、または100倍した結果が符号付き64bit整数の範囲を
    /// 超える場合にエラーを返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_core::Amount;
    ///
    /// let 三千五百円 = Amount::from_yen(3_500)?;
    ///
    /// assert_eq!(三千五百円.as_sen(), 350_000);
    /// # Ok::<(), periodless_core::AmountError>(())
    /// ```
    pub fn from_yen(yen: i64) -> Result<Self, AmountError> {
        let sen = yen.checked_mul(100).ok_or(AmountError::Overflow)?;

        Self::new(sen)
    }

    /// 金額を円と銭の組として返す。円未満の端数を切り捨てない。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_core::Amount;
    ///
    /// assert_eq!(Amount::new(100_050)?.to_yen(), (1_000, 50));
    /// assert_eq!(Amount::ZERO.to_yen(), (0, 0));
    /// # Ok::<(), periodless_core::AmountError>(())
    /// ```
    pub fn to_yen(self) -> (i64, i64) {
        (self.0 / 100, self.0 % 100)
    }

    /// 保持する1銭単位の値。
    pub fn as_sen(self) -> i64 {
        self.0
    }

    /// 二つの金額の合計を算出する。算術オーバーフローはパニックせずエラーを返す。
    ///
    /// # Errors
    ///
    /// 合計が符号付き64bit整数の範囲を超える場合に [`AmountError::Overflow`]
    /// を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_core::Amount;
    ///
    /// let 合計 = Amount::new(33_333)?.checked_add(Amount::new(33_334)?)?;
    ///
    /// assert_eq!(合計, Amount::new(66_667)?);
    /// # Ok::<(), periodless_core::AmountError>(())
    /// ```
    pub fn checked_add(self, other: Self) -> Result<Self, AmountError> {
        let sum = self.0.checked_add(other.0).ok_or(AmountError::Overflow)?;

        Ok(Self(sum))
    }
}

impl Display for Amount {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        let (yen, sen) = self.to_yen();

        write!(formatter, "{yen}.{sen:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 一銭単位で保持される() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(Amount::new(100_050)?.to_yen(), (1_000, 50));
        assert_eq!(Amount::from_yen(1_000)?.as_sen(), 100_000);

        Ok(())
    }

    #[test]
    fn 負の金額は拒否される() {
        assert_eq!(Amount::new(-1), Err(AmountError::Negative { sen: -1 }));
        assert_eq!(
            Amount::from_yen(-100),
            Err(AmountError::Negative { sen: -10_000 })
        );
    }

    #[test]
    fn 円から銭への変換の丸め誤差は発生しない() {
        assert_eq!(Amount::from_yen(1).map(Amount::to_yen), Ok((1, 0)));
        assert_eq!(Amount::new(1).map(Amount::to_yen), Ok((0, 1)));
        assert_eq!(Amount::new(99).map(Amount::to_yen), Ok((0, 99)));
        assert_eq!(Amount::new(101).map(Amount::to_yen), Ok((1, 1)));
    }

    #[test]
    fn 按分による丸め誤差は発生しない() -> Result<(), Box<dyn std::error::Error>> {
        let 一つ目 = Amount::new(33_333)?;
        let 二つ目 = Amount::new(33_333)?;
        let 三つ目 = Amount::new(33_334)?;

        let 合計 = 一つ目.checked_add(二つ目)?.checked_add(三つ目)?;

        assert_eq!(合計, Amount::new(100_000)?);

        Ok(())
    }

    #[test]
    fn 合計の算術オーバーフローはエラーを返す() -> Result<(), Box<dyn std::error::Error>> {
        let 最大値 = Amount::new(i64::MAX)?;

        assert_eq!(
            最大値.checked_add(Amount::new(1)?),
            Err(AmountError::Overflow)
        );

        Ok(())
    }

    #[test]
    fn 円単位からの構築のオーバーフローはエラーを返す() {
        let 巨大な円 = i64::MAX / 2;

        assert_eq!(Amount::from_yen(巨大な円), Err(AmountError::Overflow));
    }
}
