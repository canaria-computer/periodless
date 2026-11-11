# rust-verification-ci Specification

## Purpose

Rustワークスペースのテスト・doctest・lint・整形を Pull Request および `main`
への push で機械的に検証し、検証に失敗したコードが `main`
へマージされることを防ぐ、ドキュメント整形系CIから独立した必須チェックCIの仕様と運用規定を定義する。

## Requirements

### Requirement: ドキュメント整形CIから独立したRust専用検証ワークフロー

システムは、Rustコードの検証を、ドキュメント・設定ファイルのフォーマット検証および整形用PR自動化とは別個のGitHub
Actionsワークフロー `.github/workflows/rust.yml`
として提供しなければならない（SHALL）。Rust検証ワークフローは既存のフォーマット系ワークフローのトリガー・ジョブ・権限を変更してはならず（SHALL
NOT）、逆に既存のフォーマット系ワークフローにRust検証の責務を追加してはならない（SHALL
NOT）。

#### Scenario: Rust検証の追加がフォーマット検証に影響しない

- **WHEN** Rust検証ワークフローが追加された状態で Pull Request が作成された場合
- **THEN**
  フォーマット検証CIとRust検証CIはそれぞれ独立したワークフロー実行として並行して走り、一方の失敗が他方の実行を妨げない

#### Scenario: Rust検証の設定変更

- **WHEN** Rust検証の対象コマンドやツールチェイン設定を変更する必要が生じた場合
- **THEN** 変更は `.github/workflows/rust.yml`
  のみに閉じ、フォーマット系ワークフローの編集を必要としない

### Requirement: Pull Requestおよびmainへのpushにおけるテストとdoctestの実行

Rust検証ワークフローは、`main` を対象とする Pull Request
の作成・更新・再オープン時、および `main` への push
時に、ワークスペース全体の単体テスト・統合テストとドキュメントテスト（doctest）をすべて実行しなければならない（SHALL）。いずれかのテストが失敗した場合、ワークフローは失敗（Fail）を返さなければならない（SHALL）。

#### Scenario: すべてのテストが成功するPR

- **WHEN** Pull Request
  のコードでワークスペース全体のテストおよびdoctestがすべて成功する場合
- **THEN** テストジョブは成功（Pass）を返す

#### Scenario: テストが失敗するPR

- **WHEN** Pull Request のコードでいずれかのテストまたはdoctestが失敗する場合
- **THEN** テストジョブは失敗（Fail）を返し、そのPull Requestは `main`
  へマージできない

#### Scenario: doctestのみが失敗するPR

- **WHEN**
  単体テストは成功するが、公開APIのrustdoc内コード例（doctest）がコンパイルまたは実行に失敗する場合
- **THEN** テストジョブは失敗（Fail）を返す

### Requirement: Rustコードの整形検証とlint検証

Rust検証ワークフローは、ワークスペース全体のRustソースが標準フォーマッタの規約に従って整形されていることの検証と、警告を1件も許容しないlint検証（警告をエラーとして扱う）を実行しなければならない（SHALL）。整形違反またはlint警告が検出された場合、ワークフローは失敗（Fail）を返さなければならない（SHALL）。

#### Scenario: 未整形のRustコードを含むPR

- **WHEN** Pull Request
  に標準フォーマッタの規約に従っていないRustソースが含まれる場合
- **THEN** 整形検証ジョブは失敗（Fail）を返す

#### Scenario: lint警告を含むPR

- **WHEN** Pull Request
  のコードに未使用変数や冗長な記述などのlint警告が1件以上存在する場合
- **THEN** lint検証ジョブは失敗（Fail）を返し、警告を解消するまで `main`
  へマージできない

### Requirement: 最小権限での実行とforkからのPRへの適用

Rust検証ワークフローは読み取り専用権限（`contents: read`）で実行し、リポジトリシークレットや書き込み権限を必要としてはならない（SHALL
NOT）。forkリポジトリから作成されたPull
Requestに対しても同一の検証を実行しなければならない（SHALL）。

#### Scenario: forkリポジトリからのPR

- **WHEN** forkリポジトリのブランチから `main` 向けのPull
  Requestが作成された場合
- **THEN**
  Rust検証ワークフローはシークレットなしで実行され、テスト・lint・整形の検証結果を返す

### Requirement: 検証対象を限定しない起動条件

Rust検証ワークフローは、変更されたファイルの種別やパスによって起動を抑制してはならない（SHALL
NOT）。ドキュメントのみを変更するPull
Requestに対しても起動し、必須ステータスチェックとして成功を報告しなければならない（SHALL）。

#### Scenario: ドキュメントのみを変更するPR

- **WHEN** Markdownファイルのみを変更するPull Requestが作成された場合
- **THEN**
  Rust検証ワークフローは起動して成功を報告し、必須ステータスチェックが未実行のまま保留されてマージが阻害されることがない

### Requirement: mainブランチにおける必須ステータスチェックの登録運用

リポジトリは、`main` ブランチへのマージ前にPull
Requestを必須とし、Rust検証ワークフローの各ジョブおよびフォーマット検証CIを必須ステータスチェックとして登録する運用を定めなければならない（SHALL）。GitHub上のブランチ保護設定はワークフローファイルでは表現できないため、必要な設定項目と手順を
`CONTRIBUTING.md`
に文書として明示しなければならない（SHALL）。また、必須ステータスチェックはジョブ名で登録されるため、登録済みジョブ名を無断で変更してはならない（SHALL
NOT）。

#### Scenario: 必須チェックが失敗している状態でのマージ試行

- **WHEN** Rust検証ワークフローのいずれかのジョブが失敗している Pull Request
  をマージしようとする場合
- **THEN** `main` のブランチ保護設定により、マージはブロックされる

#### Scenario: 必須チェック登録手順の参照

- **WHEN** リポジトリ管理者が `main`
  のマージ前チェックを設定または再設定する場合
- **THEN** `CONTRIBUTING.md`
  に記載された必須ステータスチェック名の一覧と設定手順に従って設定できる

#### Scenario: ジョブ名の変更

- **WHEN** Rust検証ワークフローのジョブ名を変更する場合
- **THEN**
  必須ステータスチェックの登録名も同時に更新しなければ、変更後のジョブが必須チェックとして認識されないことを前提に、ジョブ名は安定した固定名として維持する
