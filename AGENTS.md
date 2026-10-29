# AI Agent向け作業ルール

データプラットフォームのアーキテクチャ図を作成するためのデザインシステムを管理する。

## 最初に読むもの

1. `docs/README.md`: ナレッジの索引
2. `docs/introduction/overview.md`: 目的と対象範囲
3. `docs/principles/accuracy.md`: 事実・提案・未確認事項
4. `docs/principles/clarity.md`: 表現の優先順位

## 作業に応じた参照先

| 作業 | 参照先 |
| --- | --- |
| 初めて図を作る | `docs/introduction/quick-start.md` |
| 入力と構成を整理する | `docs/guides/input-model.md`、`docs/patterns/README.md` |
| 接続元・接続先を定義する | `docs/components/source.md`、`docs/components/destination.md`、`docs/guides/input-model.md` |
| 描画する | `docs/foundations/`、`docs/components/`（領域・横断機能を含む）、`docs/guides/generation.md`。v2の共有部品は`scripts/diagram.py` |
| レビューする | `docs/guides/review.md`と対象の部品・パターン |
| 画像へ出力する | `docs/guides/export.md` |
| 公式素材を取得・更新する | `docs/governance/icon-updates.md` |
| 規則を追加・変更する | `docs/governance/contribution.md` |
| 作図の見本を探す | `examples/README.md` |

## 作業ルール

- 共通知識の正本は`docs/`。作業手順や将来のSkillsに規則を複製しない。
- ユーザーの事実、確認済みの事実、提案、未確認事項を区別し、未提示の製品・接続・頻度・所有者を事実として補わない。
- 編集可能なソース、凡例、前提、未確認事項を成果物に添える。
- 規則の変更時は対応する例と索引も更新する。図を変えた場合は描画確認を行う。
- 外部調査メモや企業別の観察記録は保存せず、採用する知見を適用条件と具体的な作図規則へ整理する。
- 全アイコン素材は`assets/`に保存する。SaaS・クラウド製品は公式の最新素材を使い、作成・改訂・公開前に配布元を再確認する。
- 取得元・確認日・SHA-256を`assets/icons/manifest.json`に記録する。公式素材を取得できなければ文字で下書きを作り、未確認と明記する。
- ローカルリンクは相対パスとする。新たな知識を追加したら`docs/README.md`を更新する。
