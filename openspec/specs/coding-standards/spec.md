# coding-standards Specification

## Purpose

人間およびLLM（AIエージェント）がPeriodlessの開発において一貫したコード品質・API設計・型安全性・テスト・Git/PR運用を守るための開発規約およびエージェント指針を定義する。

## Requirements

### Requirement: 自己文書化コードとコメント・rustdocの境界

リポジトリ内のプロダクションコードおよびテストコードは、命名・型・構造によってコード自体で意図を示さなければならない（SHALL）。行コメント（`//`）は、外部制約や回避策など説明しないと背景意図が伝わらない特殊な事情がある場合にのみ使用を認め、処理内容（What
/ How）の説明には使用してはならない（SHALL NOT）。一方、公開API（`pub`
要素）には
`///`（rustdoc）の記述を推奨し、実行可能な仕様としてドキュメントテスト（doctest）を充実させなければならない（SHALL）。

#### Scenario: 処理内容を説明するだけの行コメントの禁止

- **WHEN** コードの意図を明確にする必要がある場合
- **THEN** `//`
  行コメントを追加するのではなく、説明的な型・変数・関数への抽出によって意図を表現する

#### Scenario: 背景事情がある場合の例外的な行コメント

- **WHEN**
  外部仕様や非自明な制約により、コード構造だけでは「なぜその実装を選んだか（Why）」が伝わらない場合
- **THEN** その背景理由に限定して `//` コメントを記述できる

#### Scenario: 公開APIにおけるrustdocとdoctestの提供

- **WHEN** 公開API（`pub`
  な型・トレイト・メソッド・関数）を定義または変更する場合
- **THEN** `///` によるドキュメントコメントを付与し、`cargo test --doc`
  で検証可能な doctest コード例を含める

### Requirement: 公開APIにおけるフルーエントインターフェースとメソッドチェーン

公開API（仕訳・伝票の構築、帳簿への記帳、期間や到達位置を指定した財務諸表のクエリ等）は、フルーエントインターフェースおよびメソッドチェーンを中心として設計しなければならない（SHALL）。

#### Scenario: 公開APIの組み立てと操作

- **WHEN**
  利用者が仕訳・伝票を構築する、または集計条件を指定して財務諸表を取得する公開APIを設計する場合
- **THEN**
  メソッドチェーンによって宣言的かつ連続的に記述できるインターフェースを提供する

### Requirement: 型による不変条件の表現とパニックの禁止

プロダクションコードにおいて `unwrap()`, `expect()`, `panic!()`, `todo!()`,
`unimplemented!()`,
およびパニックを引き起こすインデックス直アクセスを使用してはならない（SHALL
NOT）。会計上の不変条件（貸借一致、日時の非逆行、有効な勘定科目など）はNewtypeパターン等の型設計および明示的な
`Result<T, E>` によって表現しなければならない（SHALL）。

#### Scenario: ドメイン制約違反のハンドリング

- **WHEN**
  貸借不一致の仕訳や、最後の仕訳より前の日時を持つ仕訳など、ドメインルールに反する入力が与えられた場合
- **THEN** パニックを起こさず、違反内容を型で識別できる `Err` を返す

### Requirement: Agentic TDD と日本語テスト関数名の採用

開発は「OpenSpec（合意済み設計） → 失敗するテスト（Red） → 最小限の実装（Green）
→ リファクタリング → rustdoc /
doctest」のサイクルで進めなければならない（SHALL）。`#[test]`
関数の名前には、仕様や受入条件との対応を直感的に示す日本語識別子を推奨・許容しなければならない（SHALL）。

#### Scenario: 受入条件に対応するテストの記述

- **WHEN** ドメインの振る舞いや不変条件を検証するテスト関数を作成する場合
- **THEN** `fn 貸借が不一致の仕訳は拒否される()`
  のように日本語の関数名を用いて検証対象の仕様を明示できる

### Requirement: Git Commit ルール（Conventional Commits・Bodyなし）

すべてのGitコミットメッセージは Conventional Commits 形式（`<type>: <subject>`
または
`<type>(<scope>): <subject>`）の1行のみで記述し、BodyおよびFooterを含めてはならない（SHALL
NOT）。`<subject>` は日本語で簡潔に記述しなければならない（SHALL）。

#### Scenario: コミットの作成

- **WHEN** 変更をGitリポジトリにコミットする場合
- **THEN** Bodyを持たない1行の日本語 Conventional Commits メッセージ（例:
  `feat(journal): 貸借一致を検証する仕訳構築APIを追加`）で記録し、1行で表現できない変更は複数のコミットに分割する

### Requirement: Pull Request の最小記述と自明なコード差分

Pull Request は1つの明確な関心事に限定し、コード差分（Diff）だけで変更内容（What
/
How）が自明になるように構成しなければならない（SHALL）。PR本文は承認者が理解できる最低限とし、「背景・目的（Why）」と「関連する
OpenSpec / Issue」のみを簡潔に記述しなければならない（SHALL）。

#### Scenario: Pull Request の作成

- **WHEN** Pull Request を作成する場合
- **THEN**
  複数の無関係な変更を混在させず、本文にはコード差分の説明を繰り返さずに変更理由（Why）と関連仕様への参照のみを記載する

### Requirement: AIエージェント向け厳選ガイドライン（AGENTS.md）の提供

リポジトリ直下に `AGENTS.md`
を配置し、LLM（AIエージェント）が作業開始時に即座に把握すべきドメイン原則・コーディング規約・参照ドキュメント（`CONTRIBUTING.md`,
`docs/README.md`,
`docs/勘定科目一覧/*.md`）へのポインタを厳選して提供しなければならない（SHALL）。

#### Scenario: AIエージェントによる規約と用語の参照

- **WHEN** AIエージェントがリポジトリ内で設計・実装・テストを行う場合
- **THEN** `AGENTS.md`
  を起点としてコーディング規約（`CONTRIBUTING.md`）および勘定科目の日英対訳表（`docs/勘定科目一覧/*.md`）を参照し、一貫した識別子と設計原則を適用できる

### Requirement: Markdown・YAML・JSONのdeno fmt準拠

リポジトリ内のMarkdown、YAML、JSONファイル（`.agents/`, `.claude/`,
`.commandcode/`, `.opencode/`, `.github/` 配下を含む）はすべて `deno fmt`
によってフォーマットされていなければならない（SHALL）。

#### Scenario: ドキュメントや設定ファイルの作成・編集

- **WHEN**
  開発者またはAIエージェントがMarkdown、YAML、JSONファイルを作成または変更する場合
- **THEN** `deno fmt` を適用し、`deno fmt --check`
  がエラーなく通過する状態を維持する

### Requirement: Rustコード変更時のローカル事前検証

開発者およびAIエージェントは、Rustコードを変更した場合、コミットまたはPull
Request作成の前に、ワークスペース全体のテスト・ドキュメントテスト・lint・整形の検証をローカルで実行し、すべてが成功する状態を維持しなければならない（SHALL）。これらの検証は
`main` へのマージ前の必須チェックとしてCIでも強制される（SHALL）。

#### Scenario: Rustコードの変更をコミットする前

- **WHEN** 開発者またはAIエージェントがRustソースを追加または変更した場合
- **THEN**
  `cargo test --workspace --all-targets`、`cargo test --doc`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`
  をローカルで実行し、すべてエラーなく成功することを確認する

#### Scenario: lint警告が残っている場合

- **WHEN** `cargo clippy` が警告を1件以上報告する場合
- **THEN**
  警告を抑制属性で隠さずコード側を修正し、警告が0件の状態にしてからコミットする
