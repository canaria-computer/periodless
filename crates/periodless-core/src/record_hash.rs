use crate::journal_entry::JournalLine;
use crate::memo::Memo;
use crate::{CommittedAt, TransactionAt};
use core::fmt::{Display, Formatter, Result as FormatResult};

/// preimage を構成する一部（ドメイン分離接頭辞）。
const PREIMAGE_PREFIX: &[u8] = b"periodless.journal-entry.v1";

/// 仕訳レコードのBLAKE3ハッシュ（32バイト）。
///
/// ハッシュは、ドメイン分離のための固定接頭辞
/// `periodless.journal-entry.v1` に始まる決定的なバイト列（preimage）から
/// 計算する。前レコードのハッシュ値を preimage に含めることで、仕訳・履歴の
/// レコードをハッシュで連結する。
///
/// # Examples
///
/// ```
/// use periodless_core::RecordHash;
///
/// assert_eq!(RecordHash::GENESIS.as_bytes(), &[0u8; 32]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordHash([u8; 32]);

impl RecordHash {
    /// 前レコードハッシュの初期値（ジェネシス）。32バイトのゼロである。
    pub const GENESIS: Self = Self([0u8; 32]);

    /// preimage から BLAKE3 ハッシュを計算する。
    pub(crate) fn of_preimage(preimage: &[u8]) -> Self {
        Self(*blake3::hash(preimage).as_bytes())
    }

    /// 保持する32バイトのハッシュ値。
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for RecordHash {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }

        Ok(())
    }
}

/// 仕訳レコードのハッシュ計算に必要な構成要素。
pub(crate) struct RecordHashParts<'a> {
    pub transaction_at: &'a TransactionAt,
    pub committed_at: &'a CommittedAt,
    pub debits: &'a [JournalLine],
    pub credits: &'a [JournalLine],
    pub memo: Option<&'a Memo>,
    pub previous_record_hash: &'a RecordHash,
}

/// 仕訳レコードの preimage を構成する。
///
/// 整数はビッグエンディアン固定幅、明細列は要素数接頭辞、通信欄はバイト長
/// 接頭辞を伴う。明細列は正規化順で渡されることを前提とし、ここでは並べ替えを
/// 行わない。
pub(crate) fn preimage_bytes(parts: &RecordHashParts<'_>) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(PREIMAGE_PREFIX);
    bytes.extend_from_slice(parts.transaction_at.canonical_string().as_bytes());
    bytes.extend_from_slice(parts.committed_at.canonical_string().as_bytes());

    bytes.extend_from_slice(&(parts.debits.len() as u32).to_be_bytes());
    for line in parts.debits {
        bytes.extend_from_slice(&line.account_id().as_u64().to_be_bytes());
        bytes.extend_from_slice(&line.amount().as_sen().to_be_bytes());
    }

    bytes.extend_from_slice(&(parts.credits.len() as u32).to_be_bytes());
    for line in parts.credits {
        bytes.extend_from_slice(&line.account_id().as_u64().to_be_bytes());
        bytes.extend_from_slice(&line.amount().as_sen().to_be_bytes());
    }

    match parts.memo {
        Some(memo) => {
            bytes.extend_from_slice(&(memo.as_str().len() as u32).to_be_bytes());
            bytes.extend_from_slice(memo.as_str().as_bytes());
        }
        None => bytes.extend_from_slice(&0u32.to_be_bytes()),
    }

    bytes.extend_from_slice(parts.previous_record_hash.as_bytes());

    bytes
}

/// 構成要素から仕訳レコードのハッシュを計算する。
pub(crate) fn record_hash_of(parts: &RecordHashParts<'_>) -> RecordHash {
    RecordHash::of_preimage(&preimage_bytes(parts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id::AccountId;
    use crate::amount::Amount;

    struct 構成要素 {
        transaction_at: TransactionAt,
        committed_at: CommittedAt,
        memo: Memo,
    }

    impl 構成要素 {
        fn 基本() -> Self {
            Self {
                transaction_at: TransactionAt::parse_rfc3339("2026-11-13T02:19:00.123Z")
                    .expect("RFC3339の日時である"),
                committed_at: CommittedAt::parse_rfc3339("2026-11-13T02:20:00Z")
                    .expect("RFC3339の日時である"),
                memo: Memo::new("月初の現金入金").expect("非空の通信欄である"),
            }
        }

        fn 現金(fen: i64) -> JournalLine {
            JournalLine::new(
                AccountId::new(0x1000_FFFF_0001_0000).expect("定義済みの勘定科目IDである"),
                Amount::new(fen).expect("非負の金額である"),
            )
        }

        fn 売上(fen: i64) -> JournalLine {
            JournalLine::new(
                AccountId::new(0x5000_0000_0001_0000).expect("定義済みの勘定科目IDである"),
                Amount::new(fen).expect("非負の金額である"),
            )
        }

        fn parts<'a>(
            &'a self,
            debits: &'a [JournalLine],
            credits: &'a [JournalLine],
            memo: Option<&'a Memo>,
            previous_record_hash: &'a RecordHash,
        ) -> RecordHashParts<'a> {
            RecordHashParts {
                transaction_at: &self.transaction_at,
                committed_at: &self.committed_at,
                debits,
                credits,
                memo,
                previous_record_hash,
            }
        }
    }

    #[test]
    fn ジェネシスは三十二バイトのゼロである() {
        assert_eq!(RecordHash::GENESIS.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn 同一内容からは同一ハッシュが得られる() {
        let 構成 = 構成要素::基本();
        let 借方 = [構成要素::現金(350_000)];
        let 貸方 = [構成要素::売上(350_000)];

        let 一回目 =
            record_hash_of(&構成.parts(&借方, &貸方, Some(&構成.memo), &RecordHash::GENESIS));
        let 二回目 =
            record_hash_of(&構成.parts(&借方, &貸方, Some(&構成.memo), &RecordHash::GENESIS));

        assert_eq!(一回目, 二回目);
    }

    #[test]
    fn 前レコードハッシュが変わればハッシュも変わる() {
        let 構成 = 構成要素::基本();
        let 借方 = [構成要素::現金(350_000)];
        let 貸方 = [構成要素::売上(350_000)];

        let 前がジェネシス = record_hash_of(&構成.parts(&借方, &貸方, None, &RecordHash::GENESIS));
        let 前が別レコード = record_hash_of(&構成.parts(&借方, &貸方, None, &前がジェネシス));

        assert_ne!(前がジェネシス, 前が別レコード);
    }

    #[test]
    fn 明細の内容が変わればハッシュも変わる() {
        let 構成 = 構成要素::基本();
        let 借方 = [構成要素::現金(350_000)];

        let 三百五十円の売上 = record_hash_of(&構成.parts(
            &借方,
            &[構成要素::売上(350_000)],
            None,
            &RecordHash::GENESIS,
        ));
        let 三百六十円の売上 = record_hash_of(&構成.parts(
            &借方,
            &[構成要素::売上(360_000)],
            None,
            &RecordHash::GENESIS,
        ));

        assert_ne!(三百五十円の売上, 三百六十円の売上);
    }

    #[test]
    fn 通信欄の有無はハッシュに反映される() {
        let 構成 = 構成要素::基本();
        let 借方 = [構成要素::現金(350_000)];
        let 貸方 = [構成要素::売上(350_000)];

        let 通信欄あり =
            record_hash_of(&構成.parts(&借方, &貸方, Some(&構成.memo), &RecordHash::GENESIS));
        let 通信欄なし = record_hash_of(&構成.parts(&借方, &貸方, None, &RecordHash::GENESIS));

        assert_ne!(通信欄あり, 通信欄なし);
    }

    #[test]
    fn ハッシュは三十二バイトである() {
        let 構成 = 構成要素::基本();
        let 借方 = [構成要素::現金(350_000)];
        let 貸方 = [構成要素::売上(350_000)];

        let hash = record_hash_of(&構成.parts(&借方, &貸方, None, &RecordHash::GENESIS));

        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn 表示は小文字の十六進である() {
        assert_eq!(
            RecordHash::GENESIS.to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
