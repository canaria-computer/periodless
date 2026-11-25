use core::fmt::{Display, Formatter, Result as FormatResult};

/// 適用通信欄の構築時に違反した不変条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoError {
    /// 通信欄が空の文字列である。未設定は `Option` の `None` で表現する。
    Empty,
}

impl Display for MemoError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self {
            MemoError::Empty => write!(formatter, "適用通信欄は空の文字列を受け付けません"),
        }
    }
}

impl core::error::Error for MemoError {}

/// 適用通信欄。任意のUTF-8文章を保持する。
///
/// 空の文字列は拒否し、未設定であることを表すには `Option` の `None`
/// を使用する。これにより「未設定」と「空の通信欄の設定」を区別する。
///
/// # Examples
///
/// ```
/// use periodless_core::Memo;
///
/// let 通信欄 = Memo::new("月初の現金入金")?;
///
/// assert_eq!(通信欄.as_str(), "月初の現金入金");
/// # Ok::<(), periodless_core::MemoError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Memo(String);

impl Memo {
    /// UTF-8文章から通信欄を構築する。空の文字列は拒否する。
    ///
    /// # Errors
    ///
    /// 文章が空である場合に [`MemoError::Empty`] を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use periodless_core::Memo;
    ///
    /// assert!(Memo::new("").is_err());
    /// # Ok::<(), periodless_core::MemoError>(())
    /// ```
    pub fn new(text: impl Into<String>) -> Result<Self, MemoError> {
        let text = text.into();

        if text.is_empty() {
            return Err(MemoError::Empty);
        }

        Ok(Self(text))
    }

    /// 保持するUTF-8文章。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 空の通信欄は拒否される() {
        assert_eq!(Memo::new(""), Err(MemoError::Empty));
    }

    #[test]
    fn 任意のutf8文章を保持する() -> Result<(), MemoError> {
        assert_eq!(Memo::new("月初の現金入金")?.as_str(), "月初の現金入金");
        assert_eq!(Memo::new("😄")?.as_str(), "😄");

        Ok(())
    }
}
