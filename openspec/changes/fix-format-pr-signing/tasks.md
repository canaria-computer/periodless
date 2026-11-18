# Tasks

## 1. 整形提案ジョブの実装方式の置き換え

- [x] 1.1 `.github/workflows/format-pr.yml` の `propose-format-pr`
      ジョブで、チェックアウトを
      `ref: ${{ github.event.pull_request.head.sha }}` のまま `fetch-depth: 0`
      に変更し、`deno fmt`
      を独立したステップとして実行する形に整理して、ジョブの `if` 条件（`closed`
      除外・fork除外・`automation/format-`
      始まりブランチ除外）が変更されていないことを確認する
- [x] 1.2 元PRを再取得して
      `state == "open"`・`head.sha == ${{ github.event.pull_request.head.sha }}`・`head.ref == ${{ github.event.pull_request.head.ref }}`
      の3条件を検証し、不一致ならステップ出力で後続をスキップする確認ステップを定義する
- [x] 1.3 `peter-evans/create-pull-request@v8` を用いて
      `token: ${{ secrets.GITHUB_TOKEN }}`、`sign-commits: true`、`base: <元PRのheadブランチ>`、`branch: automation/format-pr-<PR番号>`、`delete-branch: true`、コミットメッセージ
      `style(format): deno fmtによる自動整形`、PRタイトル・本文を設定するステップを定義し、1.2
      の確認ステップが成功した場合のみ実行されるよう `if` を付ける
- [x] 1.4 旧実装のシェル処理（`git config` / `git checkout -B` / `git commit` /
      `git push --force` / `gh pr list` / `gh pr create` / `gh pr edit`
      および差分なし時のクローズ処理）を削除し、`cleanup-on-close`
      ジョブが元の内容のまま残っていることを `git diff` で確認する
- [x] 1.5
      ワークフローYAMLの構文と構造（ジョブ2件、`permissions: contents: write` /
      `pull-requests: write`、`concurrency`
      グループ）をパースして検証し、`deno fmt --check`
      を対象ファイルに対して通す

## 2. 規約ドキュメントの更新

- [x] 2.1 `CONTRIBUTING.md`
      のフォーマット自動運用の節に、整形用コミットが署名検証済み（`Verified`）であること、`Require signed commits`
      を有効にした保護ブランチでも整形用PRを取り込んだ内容がマージできることを追記し、`deno fmt --check`
      が通ることを確認する

## 3. 統合検証

- [x] 3.1 未整形のMarkdownを含む作業ブランチから `main`
      向けPRを作成し、`automation/format-pr-<PR番号>`
      の整形用PRが1件作成されること、およびその整形コミットがGitHub上で
      `Verified` と表示されることを確認する
- [x] 3.2 整形用PRを作業ブランチへ取り込み、元PRの検証CI（`deno fmt --check` と
      Rust 3ジョブ）が成功し、`Require signed commits` を有効にした `main`
      へマージ可能な状態（署名検証エラーが出ない状態）になることを確認する
- [x] 3.3 元PRで作者が自ら `deno fmt`
      を適用して整形差分を解消した場合に、次回のワークフロー実行で整形用PRがクローズされ
      `automation/format-pr-<PR番号>`
      ブランチが削除されることを確認する（アクションの `delete-branch`
      挙動の検証）
- [x] 3.4 元PRをクローズした場合に `cleanup-on-close`
      ジョブが整形用PRをクローズしブランチを削除することを確認し、確認に使った一時PR・ブランチを片付ける
