# Tasks

## 1. ワークスペース設定と値オブジェクト

- [x] 1.1 workspaceの `Cargo.toml` に `hifitime = "=4.3.2"` と `blake3`
      を追加して `periodless-core` から参照し、スタブ関数 `add` を削除した上で
      `cargo build --workspace` が成功することを確認する
- [x] 1.2 `amount` モジュールに `Amount`（1銭単位の i64
      Newtype・負値拒否・`from_yen`/`to_yen`・`checked_add` による合計）と
      `account_id` モジュールに `AccountId`（u64
      Newtype・勘定区分と予約ビットの検証）、`memo` モジュールに
      `Memo`（非空UTF-8）を実装し、1銭単位保持（1,000円50銭＝100,050銭）・按分の過不足なし・負値拒否・無効ID拒否の日本語テスト名テストと
      rustdoc/doctest が `cargo test --workspace --all-targets` と
      `cargo test --doc` で通ることを確認する

## 2. 日時の正規化

- [x] 2.1 `datetime` モジュールに `TransactionAt` / `CommittedAt`（hifitime
      Epoch Newtype・構築時ミリ秒切り上げ正規化・年0000〜9999・正準文字列
      `YYYYMMDDTHHMMSS.sssZ`・秒00〜59のみでSS=60は出力せず秒60の入力は拒否）を実装し、仕様の変換例4件（`.123001`→`.124`、`.123000`→`.123`、`.1`→`.100`、`235959.999001`→翌日`000000.000`）、年範囲超過エラー、うるう秒入力の拒否（`2016-12-31T23:59:60Z`）、うるう秒境界をまたぐ切り上げ（`20161231T235959.999001Z`→`20170101T000000.000Z`）、冪等性・決定性の日本語テスト名テストが
      `cargo test --workspace --all-targets` で通ることを確認する

## 3. ハッシュチェーン

- [x] 3.1 `record_hash` モジュールに `RecordHash`（`[u8; 32]`
      Newtype・BLAKE3・ジェネシス＝32バイトゼロ）と design.md 記載の正準
      preimage 構成（固定接頭辞
      `periodless.journal-entry.v1`・両日時の正準文字列・正規化順の明細列・通信欄・前レコードハッシュ）を実装し、同一内容から同一ハッシュ・ジェネシスの前ハッシュ・側内の明細順序の違いがハッシュに影響しないことの日本語テスト名テストが
      `cargo test --workspace --all-targets` で通ることを確認する

## 4. 仕訳と台帳

- [x] 4.1 `journal_entry` モジュールに
      `JournalEntry`（構成要素の保持・明細列は金額降順→勘定科目ID降順の安定ソートで正規化順保持・`Option<Memo>`）とフルーエントビルダー（`transacted_at`/`committed_at`/`debit`/`credit`/`memo`/`build`・`build()`時に正規化）を実装し、貸借一致の強制（`貸借が不一致の仕訳は拒否される`
      等）、明細検証（無効な勘定科目ID・負の金額の拒否）、合計オーバーフロー、正規化と等価性（`側内の順序のみが異なる仕訳は同一の取引とみなされる`・`側内の順序のみが異なる仕訳は同一のハッシュになる`）の日本語テスト名テストと
      design.md のdoctest例が `cargo test --workspace --all-targets` と
      `cargo test --doc` で通ることを確認する
- [x] 4.2 `ledger` モジュールに `Ledger`（追記専用・`append`
      でのコミット日時単調性と前レコードハッシュ設定・レコードハッシュ計算、`verify`
      での全チェーン検証および各レコードの明細列が正規化順であることの確認・反復と件数参照のみ公開）を実装し、過去のコミット日時拒否（`過去のコミット日時を持つ仕訳は拒否される`）、等しいコミット日時の受理、過去の取引日時の受理、改変検出（`明細順序が改変された場合は検証が失敗する`）の日本語テスト名テストと
      rustdoc/doctest が `cargo test --workspace --all-targets` と
      `cargo test --doc` で通ることを確認する

## 5. 検証と正規化の仕様（違反・警告・取引の同一性）

- [x] 5.1 `periodless-core` の `validation` モジュールに
      `ValidationError`（省略行の位置・同一勘定科目の重複・自己取引・合計ゼロ・上限超過）、`ValidationWarning`（科目名末尾が数字・合計が2^40円超）、`ValidationErrors`
      を実装し、coreのビルダーがルール2・3・4・6・7を検証して違反は拒否し警告は通知する動作（`同一勘定科目が二回出現する仕訳は拒否される`・`自己取引は拒否される`・`合計がゼロの仕訳は拒否される`・`合計が上限を超える仕訳は拒否される`・`合計が2^40円を超える仕訳は警告される`）を日本語テスト名で確認する
- [x] 5.2 `periodless-journal-dsl`
      に解析結果の行構造（各行の借方・貸方の有無）の保持・公開と、DSLクレート内の検証器（省略行の位置の違反・科目名末尾が数字の警告・金額表現の連続する桁区切り文字／桁区切り文字以外の印字可能ASCII文字／基数プレフィックス表記の違反・0埋めされた表記の警告）を実装し、`periodless-core`
      の `ValidationError` / `ValidationWarning`
      を再利用する形にして、`省略行が先頭の場合は違反となる`・`両側の行が先頭の場合は違反とならない`・`科目名の末尾が数字の場合は警告となる`・`科目名の末尾が数字でない場合は警告とならない`・`桁区切り文字が連続する場合は違反となる`・`桁区切り文字以外の印字可能文字が含まれる場合は違反となる`・`基数プレフィックス表記は違反となる`・`0埋めされた金額表現は警告となる`
      の日本語テスト名テストと rustdoc/doctest が
      `cargo test --workspace --all-targets` と `cargo test --doc`
      で通ることを確認する
- [x] 5.3 `periodless-journal-dsl` の `JournalEntryDescription`
      に正規化等価性を実装する。借方・貸方の各明細列を独立に「金額（数値・降順）→
      科目名文字列長（降順）→
      科目名辞書順（降順）」の安定ソートで並べ替えた正規形に基づいて `PartialEq`
      / `Eq` / `Hash`
      を定義する（保持する明細の並び順は入力順のまま変更しない）。`両側行の行順を入れ替えた仕訳記述は同一の取引とみなされる`・`左右の組み合わせのみが異なる仕訳記述は同一の取引とみなされる`・`単独行を含む順序の違いのみの仕訳記述は同一の取引とみなされる`・`金額表現の表記の違いは等価性に影響しない`・`等価な仕訳記述は同一のハッシュになる`・`整形出力は入力順を保持する`
      の日本語テスト名テストと rustdoc/doctest が
      `cargo test --workspace --all-targets` と `cargo test --doc`
      で通ることを確認する

## 6. 用語対訳の更新と統合検証

- [x] 6.1 `CONTRIBUTING.md`
      の基本用語対訳表に取引日時（`TransactionAt`/`transaction_at`）、コミット日時（`CommittedAt`/`committed_at`）、適用通信欄（`Memo`/`memo`）、台帳（`Ledger`/`ledger`）、前レコードハッシュ（`previous_record_hash`）、レコードハッシュ（`RecordHash`/`record_hash`）を追加し、`deno fmt --check`
      が通ることを確認する
- [x] 6.2 統合検証として
      `cargo test --workspace --all-targets`、`cargo test --doc`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`、`openspec validate add-journal-entry-recording --strict`
      のすべてが成功することを確認する
