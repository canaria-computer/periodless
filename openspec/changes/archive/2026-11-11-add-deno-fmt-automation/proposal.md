# Proposal

## Why

Issue #1 に対応し、リポジトリ内の Markdown や YAML（および JSON
等）のフォーマットを `deno fmt` に統一して CI で検証するとともに、未整形の Pull
Request
に対しては開発者のブランチを直接書き換えず整形用ブランチ・PR（`automation/format-pr-<PR番号>`）を自動提案・更新・クローズする運用を確立する。

## What Changes

- `deno.json` を新設し、ドットディレクトリ（`.agents/`, `.claude/`,
  `.commandcode/`, `.opencode/`,
  `.github/`）を含むリポジトリ内の全対象ファイルを `deno fmt`
  の対象としつつ、ビルド生成物や外部パッケージディレクトリ（`target/`,
  `.opencode/node_modules/` 等）を除外する設定を定義する。
- 必須チェック用の GitHub Actions
  ワークフロー（`.github/workflows/format-check.yml`）を新設し、読み取り専用権限（`contents: read`）で
  `deno fmt --check` を実行して未整形コードの `main` マージを防止する。
- 整形提案用の GitHub Actions
  ワークフロー（`.github/workflows/format-pr.yml`）を新設し、同一リポジトリ内の未整形
  PR に対して `automation/format-pr-<PR番号>`
  ブランチおよび元の作業ブランチ向けの整形用 PR
  を自動作成・更新・クローズする（コミット SHA の競合検証および
  `automation/format-` ブランチの再帰防止を含む）。
- `CONTRIBUTING.md`, `AGENTS.md`, `openspec/config.yaml` に `deno fmt`
  によるフォーマット規約を追記する。
- リポジトリ内の既存 Markdown / YAML / JSON ファイル全体に `deno fmt`
  を適用し、`deno fmt --check` が成功するベースラインを整える。

## Capabilities

### New Capabilities

- `formatting-automation`: `deno fmt` によるフォーマット検証 CI と、未整形 PR
  に対する整形用ブランチ・PR 自動作成・更新・クローズの仕様

### Modified Capabilities

- `coding-standards`: プロジェクトのフォーマット規約（Markdown / YAML 等の
  `deno fmt` 準拠）の追加

## Impact

- 新規ファイル:
  - `deno.json`
  - `.github/workflows/format-check.yml`
  - `.github/workflows/format-pr.yml`
- 更新ファイル:
  - `CONTRIBUTING.md`
  - `AGENTS.md`
  - `openspec/config.yaml`
  - リポジトリ内の既存 Markdown / YAML / JSON ファイル（`deno fmt`
    による整形適用）
