use core::fmt::{Display, Formatter, Result as FormatResult};

/// 勘定科目IDの構築時に違反した不変条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountIdError {
    /// 勘定区分が `0001`（資産）〜`0110`（評価勘定科目）のいずれでもない。
    UndefinedCategory { category: u8 },
    /// 将来予約ビット（bit 59..=48 または bit 15..=0）が非ゼロである。
    ReservedBitsSet,
}

impl Display for AccountIdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self {
            AccountIdError::UndefinedCategory { category } => {
                write!(
                    formatter,
                    "勘定区分 {category:#06b} は定義済みの区分ではありません"
                )
            }
            AccountIdError::ReservedBitsSet => {
                write!(formatter, "将来予約ビットが非ゼロです")
            }
        }
    }
}

impl core::error::Error for AccountIdError {}

/// 勘定科目ID。`account-id-encoding` 仕様のビットレイアウトに従う64bit整数。
///
/// 上位4ビット（bit 63..=60）が勘定区分、bit 47..=32 が流動性スコア、
/// bit 31..=16 が基底勘定科目番号であり、将来予約ビット（bit 59..=48・
/// bit 15..=0）はすべてゼロでなければならない。定義済みの勘定科目IDは
/// `docs/勘定科目一覧/accounts.yaml` を参照のこと。
///
/// # Examples
///
/// ```
/// use periodless_core::AccountId;
///
/// let 現金 = AccountId::new(0x1000_FFFF_0001_0000)?;
///
/// assert_eq!(現金.as_u64(), 0x1000_FFFF_0001_0000);
/// # Ok::<(), periodless_core::AccountIdError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(u64);

impl AccountId {
    /// 勘定区分・流動性スコア・基底勘定科目番号・予約ビットの配置に従い、
    /// 64bit整数から勘定科目IDを構築する。
    ///
    /// 検証内容は `account-id-encoding` 仕様による:
    ///
    /// - 勘定区分（`id >> 60`）が `0b0001`（資産）〜`0b0110`（評価勘定科目）である
    /// - 将来予約ビット（bit 59..=48 および bit 15..=0）がすべてゼロである
    ///
    /// # Errors
    ///
    /// 勘定区分が未定義である場合に [`AccountIdError::UndefinedCategory`]、
    /// 予約ビットが非ゼロである場合に [`AccountIdError::ReservedBitsSet`] を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_core::AccountId;
    ///
    /// let 現金 = AccountId::new(0x1000_FFFF_0001_0000)?;
    /// let 売上 = AccountId::new(0x5000_0000_0001_0000)?;
    ///
    /// assert!(現金.as_u64() < 売上.as_u64());
    /// # Ok::<(), periodless_core::AccountIdError>(())
    /// ```
    pub fn new(id: u64) -> Result<Self, AccountIdError> {
        let category = (id >> 60) as u8;

        if !(1..=0b0110).contains(&category) {
            return Err(AccountIdError::UndefinedCategory { category });
        }

        if id >> 48 & 0x0FFF != 0 || id & 0xFFFF != 0 {
            return Err(AccountIdError::ReservedBitsSet);
        }

        Ok(Self(id))
    }

    /// 保持する64bit整数値。
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 有効な勘定科目IDは受理される() -> Result<(), AccountIdError> {
        AccountId::new(0x1000_FFFF_0001_0000)?;
        AccountId::new(0x2000_F000_0001_0000)?;
        AccountId::new(0x3000_0000_0001_0000)?;
        AccountId::new(0x4000_0000_0001_0000)?;
        AccountId::new(0x5000_0000_0001_0000)?;
        AccountId::new(0x6000_0000_0001_0000)?;

        Ok(())
    }

    #[test]
    fn 未定義の勘定区分は拒否される() {
        assert_eq!(
            AccountId::new(0),
            Err(AccountIdError::UndefinedCategory { category: 0 })
        );
        assert_eq!(
            AccountId::new(0b0111 << 60),
            Err(AccountIdError::UndefinedCategory { category: 0b0111 })
        );
        assert_eq!(
            AccountId::new(u64::MAX),
            Err(AccountIdError::UndefinedCategory { category: 0b1111 })
        );
    }

    #[test]
    fn 予約ビットが非ゼロのIDは拒否される() {
        let 予約ビット1が立った現金 = 0x1001_FFFF_0001_0000;

        assert_eq!(
            AccountId::new(予約ビット1が立った現金),
            Err(AccountIdError::ReservedBitsSet)
        );

        let 予約ビット2が立った現金 = 0x1000_FFFF_0001_0001;

        assert_eq!(
            AccountId::new(予約ビット2が立った現金),
            Err(AccountIdError::ReservedBitsSet)
        );
    }

    #[test]
    fn 値の大きさは勘定区分の順序に従う() -> Result<(), AccountIdError> {
        let 現金 = AccountId::new(0x1000_FFFF_0001_0000)?;
        let 売上 = AccountId::new(0x5000_0000_0001_0000)?;

        assert!(現金 < 売上);

        Ok(())
    }
}
