# Design

## Context

`format-pr.yml` の `propose-format-pr` ジョブは現在、ランナー上で `deno fmt`
を実行したのち `git commit` と `git push --force` で整形用ブランチ
`automation/format-pr-<PR番号>` を更新し、`gh pr create` / `gh pr edit`
でPRを管理している。`GITHUB_TOKEN`
環境ではこのコミットに署名が付かないため、`Require signed commits`
を要求する保護ブランチへ最終的にマージできない（動機は `proposal.md` の Why
を参照）。要求仕様は `specs/formatting-automation/spec.md` を参照する。

既存の挙動のうち、fork PR除外・`automation/format-`
始まりブランチの再帰防止（ジョブの `if`
条件）、元PRクローズ時のクリーンアップ（`cleanup-on-close`
ジョブ）、1つの元PRに対して整形用PRを1件に保つ制約は維持対象である。

## Goals / Non-Goals

**Goals:**

- 整形用コミットを署名検証済み（`Verified`）にする。
- 整形用ブランチ・PRのライフサイクル管理をアクションに委譲し、ワークフロー内のシェルスクリプトを縮小する。
- 公開直前の元PR整合性確認をSHAのみから状態・headブランチ名まで広げる。

**Non-Goals:**

- GitHub App トークンや PAT の導入（標準 `GITHUB_TOKEN` の範囲で解決する）。
- fork PR に対する整形用ブランチ自動作成（権限境界の都合上、従来どおり対象外）。
- `main` に既に存在する未署名コミットの履歴書き換え。
- 読み取り専用のフォーマット検証CI（`format-check.yml`）の変更。

## Decisions

### 1. `peter-evans/create-pull-request@v8` + `sign-commits: true` の採用

- **決定**: 整形用ブランチへのコミットとPRの作成・更新を
  `peter-evans/create-pull-request@v8` に委譲し、`sign-commits: true`
  を指定する。`token` は `secrets.GITHUB_TOKEN` を渡す。
- **理由**: このオプションはコミットをGitHub
  API経由で作成するため、`github-actions[bot]` の署名が付き `Verified`
  として検証される。ランナー上の `git commit` では `GITHUB_TOKEN`
  で署名を得る手段がない。v8系は現在も更新されている（最新 v8.1.3）。
- **代替案**:
  - **GPG鍵をシークレットに置いて自前署名**:
    鍵の管理・ローテーション責務が増え、`GITHUB_TOKEN`
    のみで完結する構成から外れるため却下。
  - **GitHub App トークンでの署名**:
    bot名を自前のものにできるが、App登録と秘密鍵管理が必要で、本件の目的（署名検証を通すこと）に対して過剰。

### 2. ベースブランチと浅いクローンの扱い

- **決定**: チェックアウトは `ref: ${{ github.event.pull_request.head.sha }}`
  を維持したまま `fetch-depth: 0` とし、アクションには
  `base: <元PRのheadブランチ>` を明示する。
- **理由**: `head.sha` 指定のチェックアウトは detached HEAD
  になり、アクションのベース推定（現在のブランチ名）が機能しないため `base`
  の明示が必須になる。またアクションはベースブランチとの差分計算と履歴操作を行うため、`fetch-depth: 1`
  では不足する。

### 3. 元PR整合性確認の位置と内容

- **決定**: `deno fmt`
  実行後、アクション実行前の独立したステップで元PRを再取得し、`state == "open"`・`head.sha == 開始時SHA`・`head.ref == 開始時ブランチ名`
  の3条件を確認する。いずれかが不一致なら以降のステップを実行しない（ステップ出力でスキップする）。
- **理由**:
  公開（整形用ブランチの更新とPR作成）より前に確認することで、古いコミットや別ブランチに基づく修正案の公開を防ぐ。headブランチ差し替えの確認を加えたのは、`base`
  を元PRのheadブランチ名で明示する構成では、ブランチ名が変わった場合に誤ったマージ先のPRを作り得るため。

### 4. クリーンアップの責務分担

- **決定**:
  - 元PRがクローズされた場合: 既存の `cleanup-on-close` ジョブ（`gh pr close` と
    ref 削除）を維持する。
  - 整形差分が解消された場合: アクションの `delete-branch: true` に委ねる。
- **理由**: アクションの `delete-branch`
  は「次回実行時に差分が無ければ整形用ブランチを削除し、オープン中のPRを閉じる」挙動として文書化されており、元PRの
  `synchronize`
  で再実行される本ワークフローの性質と一致する。一方、元PRのクローズ時は整形提案ジョブ自体が実行対象外（`if`
  条件で `closed`
  を除外）であり、アクションには検知手段がないため自前のジョブが必要。
- **リスク**:
  差分解消時のクローズはアクションの実行タイミングに依存するため、実PRでの統合検証（`tasks.md`
  の統合検証タスク）で確認する。

## Risks / Trade-offs

- **[Risk] アクションによるPR作成には「Allow GitHub Actions to create and
  approve pull requests」設定が必要** →
  現行実装（`gh pr create`）でも同じ設定を前提としているため追加作業はないが、整形用PRが作成されない場合はこの設定を最初に確認する。
- **[Risk] `GITHUB_TOKEN` で作成されたPRはワークフローを起動しない** →
  現行と同じ制約。作者が整形用PRを作業ブランチへマージした時点で元PRの
  `synchronize` が発火し検証CIが走るため運用上の影響はない。
- **[Risk] 外部アクションへの依存追加（サプライチェーン）** → メジャータグ `@v8`
  で追随する。より厳格にするならコミットSHA固定も選べるが、既存ワークフローが
  `actions/checkout@v4` 等のメジャータグ運用で揃っているため本変更では合わせる。
- **[Trade-off]
  ライフサイクル管理をアクションに委ねることで、挙動の詳細がアクションの実装に依存する**
  →
  シェルスクリプトの削減と引き換えに、差分解消時のクローズ等は実PRでの検証で担保する。
