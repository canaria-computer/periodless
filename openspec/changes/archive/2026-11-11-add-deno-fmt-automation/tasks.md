# Tasks

## 1. Deno フォーマット設定とドキュメントの更新

- [x] 1.1 `deno.json` を作成し、ドットディレクトリ（`.agents`, `.claude`,
      `.commandcode`, `.opencode`, `.github`）を含む全対象ファイルを
      `fmt.include` に含め、`target` および `.opencode/node_modules` を
      `fmt.exclude` に指定して `deno fmt` がこれらを走査することを確認する
- [x] 1.2 `CONTRIBUTING.md`, `AGENTS.md`, `openspec/config.yaml` に `deno fmt`
      によるフォーマット規約と CI/整形用 PR
      自動運用の要点を追記し、内容を確認する

## 2. GitHub Actions ワークフローの実装

- [x] 2.1 `.github/workflows/format-check.yml`
      を作成し、読み取り専用権限（`contents: read`）で PR および `main` への
      push 時に `deno fmt --check` を実行する必須チェック定義を検証する
- [x] 2.2 `.github/workflows/format-pr.yml` を作成し、同一リポジトリ内かつ
      `automation/format-` 以外の PR に対して整形差分検知・SHA
      競合確認・`automation/format-pr-<PR番号>` ブランチと整形用 PR
      の作成/更新・整形解消または元の PR
      クローズ時の自動クローズ/ブランチ削除を行うワークフローを定義して構文を検証する

## 3. リポジトリ全体へのフォーマット適用と統合検証

- [x] 3.1 リポジトリ全体に `deno fmt` を実行し、`deno fmt --check` および
      `openspec validate add-deno-fmt-automation --strict`
      がともにエラーなく成功することを確認する
