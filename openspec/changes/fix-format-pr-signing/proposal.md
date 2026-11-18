# Proposal

## Why

`main` のブランチ保護で `Require signed commits`
を有効化したところ、整形用PR自動化（`.github/workflows/format-pr.yml`）が作る整形コミットが署名検証されない形式のため、整形用PRを取り込んだブランチを
`main` へマージできなくなった。現在の実装はランナー上で `git commit` /
`git push` を行っており、`GITHUB_TOKEN` では署名が付かない。

## What Changes

- `format-pr.yml` の整形提案ジョブを、ランナー上での `git commit` / `git push`
  から GitHub API
  経由でコミットを作成する方式へ置き換え、整形コミットを署名検証可能（`Verified`）にする。
- 整形処理の対象コミットと公開直前の元PR状態の照合を、コミットSHAに加えて元PRの
  `state`（open であること）と head ブランチ名の一致まで拡張する。
- 元PRのクローズ時の整形用PRクローズ・ブランチ削除は、既存の `cleanup-on-close`
  ジョブとして維持する（採用するアクションは自身が実行された時の差分有無でのみ整形用ブランチを削除するため、元PRのクローズは検知できない）。整形差分が解消された場合のクローズは、アクションの
  `delete-branch`
  の挙動（差分が無くなった整形用ブランチを次回実行時に削除し、オープン中のPRを閉じる）に委ね、実PRでの統合検証で確認する。
- `CONTRIBUTING.md`
  のフォーマット自動運用の節に、整形コミットが署名付きであることと
  `Require signed commits` との関係を追記する。

## Capabilities

### New Capabilities

（なし）

### Modified Capabilities

- `formatting-automation`:
  整形用コミットが署名検証可能であることの要件追加と、公開直前の元PR整合性確認（SHA・状態・headブランチ名）への要件拡張

## Impact

- 更新ファイル:
  - `.github/workflows/format-pr.yml`（整形提案ジョブの実装方式変更。`cleanup-on-close`
    ジョブは維持）
  - `CONTRIBUTING.md`
- 依存関係: 整形用PRの作成・更新に外部アクション
  `peter-evans/create-pull-request`（v8 系。`sign-commits`
  入力を持つ）を追加する。
- 運用への影響:
  - `Require signed commits`
    を有効にしたブランチ保護下でも、整形用PRを取り込んだブランチが `main`
    へマージできるようになる。
  - 既に `main`
    に存在する未署名の整形コミット（`15e7280`）は遡って変更しない。ブランチ保護は以後のpushにのみ適用される。
  - `GITHUB_TOKEN`
    で作成されたPRがワークフローを起動しない制約は現行と変わらず、作者が整形用PRを作業ブランチへマージした時点で元PRの
    `synchronize` により検証CIが再実行される。
