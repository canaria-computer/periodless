# Design

## Context

Issue #1 では、MarkdownやYAMLのフォーマッターとして `deno fmt`
を導入し、以下の2つの責務を分離したGitHub Actions運用を定義している（背景は
`proposal.md` を参照）：

1. **検証する CI（必須チェック）**: 読み取り専用権限（`contents: read`）で
   `deno fmt --check` を実行し、未整形コードの `main` への混入を確実に防ぐ。
2. **修正案を作る自動化（開発者支援）**:
   未整形のPRに対して、作者のブランチを直接変更するのではなく、`automation/format-pr-<PR番号>`
   ブランチを作成して元の作業ブランチ向けの整形用PRを1件だけ作成・更新する。

## Goals / Non-Goals

**Goals:**

- ドットディレクトリ（`.agents/`, `.claude/`, `.commandcode/`, `.opencode/`,
  `.github/`）を含むリポジトリ内の全対象ファイルを `deno fmt`
  で一括管理できるように `deno.json` を構成する。
- 検証CI（`.github/workflows/format-check.yml`）と整形用PR自動化（`.github/workflows/format-pr.yml`）を最小権限で分離実装する。
- 元のPR 1件につき整形用PR
  1件の維持、コミットSHA一致確認、再帰防止、不要になった整形用PR・ブランチの自動クリーンアップを実現する。

**Non-Goals:**

- forkリポジトリからのPRに対する整形ブランチ自動作成（権限境界の都合上、同一リポジトリ内ブランチに限定する）。
- GitHub App や外部 PAT の導入（標準 `GITHUB_TOKEN`
  を用いたシンプルな構成とする）。

## Decisions

### 1. `deno.json` によるドットディレクトリを含むフォーマット対象定義

- **決定**: `deno fmt` はデフォルトでは先頭が `.`
  の隠しディレクトリを走査対象から外す挙動がある場合があるため、`deno.json` の
  `fmt.include` に
  `["." , ".agents", ".claude", ".commandcode", ".opencode", ".github"]`
  を明示し、`fmt.exclude` に `["target", ".opencode/node_modules"]` を設定する。
- **理由**: 引数なしで `deno fmt` / `deno fmt --check`
  を実行するだけで、ローカルでもCIでも同一の対象ファイル集合（ドットディレクトリを含む全件）が確実にフォーマット・検査されるようにするため。

### 2. ワークフローの分割（`format-check.yml` と `format-pr.yml`）

- **決定**:
  - `.github/workflows/format-check.yml`:
    - トリガー: `pull_request` (`opened`, `synchronize`, `reopened`) および
      `push` (`branches: [main]`)
    - 権限: `contents: read`
    - 実行コマンド: `deno fmt --check`
  - `.github/workflows/format-pr.yml`:
    - トリガー: `pull_request` (`opened`, `synchronize`, `reopened`, `closed`)
    - 権限: `contents: write`, `pull-requests: write`
- **理由**:
  検証CIはブランチ保護ルールの必須ステータスチェックとして常に最小権限（読み取りのみ）で動作させ、整形用PRの生成可否に関わらず
  `main` の整合性を保証するため。

### 3. 整形用PRのライフサイクル制御とSHA競合防止

- **決定**: `format-pr.yml` 内で以下の順序で処理を行う。
  1. **除外判定**:
     `github.event.pull_request.head.repo.full_name == github.repository` かつ
     `!startsWith(github.event.pull_request.head.ref, 'automation/format-')`
     の場合のみ実行。
  2. **PRクローズ時（`github.event.action == 'closed'`）**:
     - `automation/format-pr-<PR番号>`
       ブランチが存在すれば、オープン中の整形用PRをクローズし、リモートブランチを削除する。
  3. **PRオープン・更新時（`opened`, `synchronize`, `reopened`）**:
     - 開始時の `TARGET_SHA="${{ github.event.pull_request.head.sha }}"`
       を記録し、チェックアウトする。
     - `deno fmt` を実行し、`git status --porcelain` で差分の有無を確認する。
     - **差分がない場合**: オープン中の `automation/format-pr-<PR番号>` PR
       があればクローズし、リモートブランチを削除して終了する。
     - **差分がある場合**:
       - ローカルで `automation/format-pr-<PR番号>`
         ブランチを作成し、`style(format): deno fmtによる自動整形`
         でコミットする（プロジェクトのコミット規約に準拠）。
       - **公開直前のSHA検証**:
         `gh api "repos/${GITHUB_REPOSITORY}/pulls/${PR_NUMBER}" --jq .head.sha`
         を取得し、`TARGET_SHA` と一致するか確認する。不一致の場合は push や PR
         作成を行わず終了する。
       - 一致する場合、`git push --force origin "HEAD:refs/heads/automation/format-pr-${PR_NUMBER}"`
         で整形用ブランチを更新する。
       - `gh pr list --repo "${GITHUB_REPOSITORY}" --head "automation/format-pr-${PR_NUMBER}" --base "${TARGET_BRANCH}" --state open --json number --jq '.[0].number'`
         で既存の整形用PRを検索し、存在しなければ `gh pr create`、存在すれば
         `gh pr edit` で本文（対象SHAの記録）を更新する。

## Risks / Trade-offs

- **[Risk] 標準 `GITHUB_TOKEN` によるPR作成権限** →
  リポジトリ設定（`Settings > Actions > General > Workflow permissions`）で「Allow
  GitHub Actions to create and approve pull
  requests」が有効である必要がある。未有効の場合は `gh pr create`
  が権限エラーを返すため、手順を案内する。
- **[Risk] 標準 `GITHUB_TOKEN` で作成した整形用PR自体には `pull_request`
  イベントが発火しない** → GitHub Actions の仕様上、`GITHUB_TOKEN`
  によるアクションからは再帰防止のため新規ワークフローが起動しない。しかし Issue
  #1 の「作者が取り込んだ後の CI
  を基準にします」の通り、作者が整形用PRを作業ブランチにマージした時点で元のPRの
  `synchronize`
  イベントが発火し、元のPRに対して検証CIが再実行されるため運用上問題ない。
