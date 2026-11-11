# formatting-automation Specification

## Purpose

リポジトリ内のMarkdownやYAML等のドキュメント・設定ファイルを `deno fmt`
で統一的に整形し、検証CIによる `main`
ブランチ保護と、未整形PRに対する整形用ブランチ・PR自動提案ワークフローを定義する。

## Requirements

### Requirement: deno fmt による全対象ファイルのフォーマット検証CI

システムは、ドットディレクトリ（`.agents/`, `.claude/`, `.commandcode/`,
`.opencode/`,
`.github/`）を含むリポジトリ内のすべてのMarkdown・YAML・JSONファイルを対象に、`deno fmt --check`
を実行する読み取り専用の必須チェックCIを備えなければならない（SHALL）。

#### Scenario: 整形済みPRの検証成功

- **WHEN** Pull Request 内のすべての対象ファイルが `deno fmt`
  の規約に従って整形されている場合
- **THEN** フォーマット検証CIは成功（Pass）を返す

#### Scenario: 未整形PRの検証失敗

- **WHEN** Pull Request 内に `deno fmt --check`
  で差分が検出される未整形ファイルが含まれる場合
- **THEN** フォーマット検証CIは失敗（Fail）を返し、未整形のまま `main`
  ブランチへマージされることを防ぐ

### Requirement: 未整形PRに対する整形用ブランチおよび整形用PRの自動作成・更新

同一リポジトリ内の作業ブランチから作成されたPull
Requestに未整形ファイルが存在する場合、自動化ワークフローは元の作業ブランチを直接書き換えず、元のPRの最新コミットから整形差分のみをコミットしたブランチ
`automation/format-pr-<PR番号>`
を作成・更新し、元の作業ブランチをマージ先（base）とする整形用PRを1件のみ作成または更新しなければならない（SHALL）。

#### Scenario: 未整形PRに対する初回の整形用PR作成

- **WHEN** 同一リポジトリ内のブランチ `feature/example` から作成されたPR `#123`
  に未整形ファイルが存在し、まだ `automation/format-pr-123`
  のオープンPRが存在しない場合
- **THEN** 自動化ワークフローは `automation/format-pr-123`
  ブランチに整形差分をコミットしてpushし、マージ先を `feature/example`
  とする整形用PRを作成する

#### Scenario: 既存の整形用PRがある状態での元のPRの更新

- **WHEN** 元のPR `#123`
  に新しいコミットがpushされ、依然として未整形ファイルが存在し、すでに
  `automation/format-pr-123` のオープンPRが存在する場合
- **THEN** 自動化ワークフローは新しいPRを重複作成せず、既存の
  `automation/format-pr-123`
  ブランチと整形用PRを最新コミットベースの内容に更新する

### Requirement: 整形対象コミットSHAの競合確認

自動化ワークフローは、整形処理を開始した時点の元のPRのコミットSHAを記録し、整形用ブランチの公開（pushおよびPR作成・更新）直前に元のPRの最新
`head.sha` と一致することを確認しなければならない（SHALL）。

#### Scenario: 整形処理中に作者が追加コミットをpushした場合

- **WHEN** 整形処理の対象としたコミットSHAと、公開直前に取得した元のPRの最新
  `head.sha` が一致しない場合
- **THEN**
  自動化ワークフローはその古いコミットに基づく修正案を公開せず処理を中断する

### Requirement: 整形用PRの再帰防止とfork PRの除外

自動化ワークフローは、`automation/format-`
で始まるブランチからのPRおよびforkリポジトリからのPRに対して、整形用PRの作成処理を実行してはならない（SHALL
NOT）。ただし、読み取り専用のフォーマット検証CI自体はそれらのPRに対しても実行しなければならない（SHALL）。

#### Scenario: 整形用PR自体に対するトリガー

- **WHEN** `automation/format-pr-123`
  ブランチをheadとする整形用PRが作成または更新された場合
- **THEN**
  フォーマット検証CIは実行されるが、さらなる整形用PRを作成する自動化処理はスキップされる

### Requirement: 整形解消時または元のPR終了時の整形用PR自動クローズ

元のPRにおいて未整形差分が解消された場合、または元のPRがクローズ（マージを含む）された場合、自動化ワークフローは対応する
`automation/format-pr-<PR番号>`
のオープンPRをクローズし、整形用ブランチを削除しなければならない（SHALL）。

#### Scenario: 元のPRで整形が完了した場合のクリーンアップ

- **WHEN** 元のPR `#123` が更新されて `deno fmt`
  による差分が0件になった場合、または元のPR `#123` がクローズされた場合
- **THEN** オープン中の `automation/format-pr-123`
  PRがあれば自動的にクローズされ、リモートブランチ `automation/format-pr-123`
  が削除される
