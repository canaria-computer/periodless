# Design

## Context

リポジトリの既存CIは `deno fmt` 系の2ワークフロー（`format-check.yml` の検証と
`format-pr.yml` の整形用PR自動化）のみで、Rustの検証は存在しない（動機は
`proposal.md` の Why を参照）。

`main` の現状は単一パッケージ `periodless`（`edition = "2024"`、`src/`
直下）であり、ワークスペース化（`crates/periodless-core` /
`crates/periodless-journal-dsl`
への分離）と仕訳記述DSLは未マージのブランチ上で進行中である。本CIは
`--workspace`
を付けたコマンドで構成するため、単一パッケージ構成でもワークスペース化後の複数クレート構成でも同一のワークフロー定義がそのまま機能する。`rust-toolchain.toml`
は持たない。要求仕様は `specs/rust-verification-ci/spec.md` および
`specs/coding-standards/spec.md` を参照する。

## Goals / Non-Goals

**Goals:**

- Rust検証を単一の独立したワークフローファイルに閉じ込め、既存のフォーマット系ワークフローへ一切変更を加えない。
- 必須ステータスチェックとして安定して参照できるジョブ名の体系を定める。
- ドキュメントのみの変更でもチェックが保留にならない起動条件にする。

**Non-Goals:**

- GitHubのブランチ保護／ルールセットの設定操作自体（ワークフローファイルでは表現できないため、手順の文書化までを範囲とする）。
- `unwrap` / `expect` / `panic!` の静的禁止のlint化（下記 Decisions 5
  を参照し、別変更に切り出す）。
- リリース・公開（`cargo publish`）やカバレッジ測定などのCD領域。
- 複数OS・複数ツールチェインのマトリクス実行。

## Decisions

### 1. Rust専用ワークフロー `.github/workflows/rust.yml` の新設

- **決定**: `name: Rust` の新規ワークフローを1ファイル追加し、既存の
  `format-check.yml` にRustジョブを足す形は採らない。
- **理由**:
  関心事（ドキュメント整形の検証／Rustコードの検証）が異なり、ツールチェインもトリガー上の注意点も別である。ファイルを分けることで、Rust検証の設定変更が整形CIの挙動に波及せず、Actionsの実行一覧でも失敗原因が即座に判別できる。
- **代替案**: `format-check.yml`
  にジョブを追加する案は、ワークフロー名が実態と乖離し、必須チェックの管理単位も混在するため却下。

### 2. ジョブ分割（`test` / `clippy` / `fmt`）と固定ジョブ名

- **決定**: 3ジョブに分割し、`name` を固定文字列（`cargo test` / `cargo clippy`
  / `cargo fmt --check`）として維持する。
- **理由**:
  必須ステータスチェックはジョブ名で登録されるため安定名が必要であり、分割によって「テストは通るがlintが落ちた」といった失敗原因がチェック一覧上で区別できる。各ジョブは並行実行されるため総実行時間も短い。
- **代替案**:
  1ジョブ内で順次実行する案は、最初の失敗で後続の検証結果が見えなくなるため却下。

### 3. トリガーと起動条件

- **決定**: `push: branches: [main]` と
  `pull_request: [opened, synchronize, reopened]`（`format-check.yml`
  と同形）とし、`paths` フィルタは設定しない。`concurrency` は
  `rust-${{ github.ref }}` / `cancel-in-progress: true` とする。
- **理由**: `paths: ['crates/**']`
  のように絞ると、ドキュメントのみのPRでジョブが起動せず、必須ステータスチェックが「未実行」のまま保留されてマージがブロックされる（GitHubの既知の挙動）。全PRで起動させ、実行コストはキャッシュで抑える。`concurrency`
  グループは `format-pr.yml` と衝突しない独自名にする。

### 4. ツールチェインとキャッシュ

- **決定**: `actions/checkout@v4` +
  `dtolnay/rust-toolchain@stable`（`components: rustfmt, clippy`）+
  `Swatinem/rust-cache@v2` を使用し、`rust-toolchain.toml` は追加しない。
- **理由**: `edition = "2024"` / `resolver = "3"`
  には比較的新しいstableが必要で、stable追従により将来のedition要件にも自動的に追いつく。`actions-rs/*`
  は保守停止のため採用しない。`Swatinem/rust-cache` は `~/.cargo` と `target/`
  のキャッシュ戦略がワークスペース構成に最適化されている。
- **代替案**:
  ツールチェインを固定バージョンにピン留めする案は、新しいstableのlint追加でCIが突然失敗するリスクを避けられる一方、更新忘れで古い環境に固定される。本プロジェクトは依存が少なく破壊的影響が小さいためstable追従を選ぶ。

### 5. clippyの厳格度の段階導入

- **決定**: 本変更では `cargo clippy --workspace --all-targets -- -D warnings`
  までとし、`unwrap` / `expect` / `panic!` の禁止（`clippy.toml` の
  `disallowed_methods` や `[workspace.lints]` による `clippy::unwrap_used`
  等のdeny）は導入しない。
- **理由**:
  これらのlintはテストコードでも発火するため、テスト側への例外設定（`#![cfg_attr(test, allow(...))]`
  等）の方針決定を伴い、AGENTS.mdの規約（テストコードは対象外）との整合も設計が必要になる。1変更=1関心事を保ち、別変更として提案する。

### 6. `main` 保護設定の文書化

- **決定**: `CONTRIBUTING.md` の CI/PR 運用節に、ルールセットで「Require a pull
  request before merging」および「Require status checks to pass before
  merging」を有効化し、必須チェックとして `cargo test` / `cargo clippy` /
  `cargo fmt --check` / `deno fmt --check` の4件を登録する手順を追記する。
- **理由**: ブランチ保護はリポジトリ設定でありコードでは表現できない。`gh api`
  によるルールセット自動適用はリポジトリ管理権限を要し、CIの冪等な実行対象にもできないため、手順の明文化に留める。

## Risks / Trade-offs

- **[Risk] CI導入時点で既存コードが `clippy -D warnings`
  に違反していて初回から赤になる** → `origin/main`
  の内容では4コマンド（`cargo test --workspace --all-targets` /
  `cargo test --doc` / `cargo clippy --workspace --all-targets -- -D warnings` /
  `cargo fmt --all -- --check`）がすべて成功することを確認済み。実装タスクでも
  `origin/main`
  相当のコードに対し4コマンドをローカル実行して事前確認し、違反があれば本変更内で最小限修正する。修正規模が大きい場合はlintジョブのみ後続変更に切り出す。
- **[Risk]
  stable追従により、新しいstableリリース時の新規lintで無関係なPRが落ちる** →
  失敗はlintジョブに限定され、修正は該当lintへの対応または必要に応じたピン留めで対処できる。
- **[Risk]
  必須チェック登録後にジョブ名を変更すると、旧名のチェックが永久にpendingになりマージ不能になる**
  →
  ジョブ名を仕様（`specs/rust-verification-ci/spec.md`）で固定名として規定し、変更時はルールセットの登録名も同時更新する運用を
  `CONTRIBUTING.md` に明記する。
- **[Trade-off] `paths`
  フィルタを使わないため、ドキュメントのみのPRでもRust検証が走り実行時間とActions利用量を消費する**
  →
  必須チェックの保留によるマージ不能を避けることを優先し、キャッシュで実行時間を抑える。
