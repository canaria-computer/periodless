# Spec Delta

## ADDED Requirements

### Requirement: Markdown・YAML・JSONのdeno fmt準拠

リポジトリ内のMarkdown、YAML、JSONファイル（`.agents/`, `.claude/`,
`.commandcode/`, `.opencode/`, `.github/` 配下を含む）はすべて `deno fmt`
によってフォーマットされていなければならない（SHALL）。

#### Scenario: ドキュメントや設定ファイルの作成・編集

- **WHEN**
  開発者またはAIエージェントがMarkdown、YAML、JSONファイルを作成または変更する場合
- **THEN** `deno fmt` を適用し、`deno fmt --check`
  がエラーなく通過する状態を維持する
