# デザインシステムのナレッジ

本ディレクトリを共通知識の正本とする。作業手順や将来のSkillsから規則を参照し、同じ規則を複製しない。

## はじめに

- [概要・対象範囲](introduction/overview.md)
- [最初の1枚を作る](introduction/quick-start.md)

## 原則

- [正確性](principles/accuracy.md)
- [読みやすさと情報の優先順位](principles/clarity.md)

## 基本要素

- [色](foundations/color.md)
- [文字・掲載幅での可読性](foundations/typography.md)
- [余白とレイアウト](foundations/spacing.md)
- [アイコンの選択と配置](foundations/iconography.md)

## 作図部品

- [製品・処理ノード](components/node.md)
- [接続元](components/source.md)
- [接続先](components/destination.md)
- [論理層](components/logical-layer.md)
- [名前付き領域・境界](components/boundary.md)
- [横断機能](components/cross-cutting.md)
- [接続](components/connector.md)
- [ラベル・凡例・注記](components/annotation.md)

## 構成パターン

- [選択ガイド](patterns/README.md)
- [段階的な変換](patterns/layered-pipeline.md)
- [用途別の基盤](patterns/purpose-based-platform.md)
- [契約による境界](patterns/contract-boundary.md)
- [データ・AI基盤](patterns/data-ai-platform.md)

## 作成ガイド

- [入力の情報モデル](guides/input-model.md)
- [構成図の生成](guides/generation.md)
- [レビュー](guides/review.md)
- [編集形式と画像出力](guides/export.md)

## 運用とサンプル

- [規則の追加・変更](governance/contribution.md)
- [公式アイコンの取得・更新](governance/icon-updates.md)
- [出力例の一覧・再生成方法](../examples/README.md)
- [公式素材と取得記録](../assets/icons/README.md)
- [素材と参照の検証スクリプト](../scripts/validate_assets.py)
- [共有SVG部品](../scripts/diagram.py)・[描画トークン](../assets/design-tokens.json)
- [図の構造・配置の検証](../scripts/validate_diagrams.py)

## 配置ルール

`docs/`には規則、`examples/`には説明と編集可能な出力例、`assets/`には全アイコン素材を格納する。`AGENTS.md`は作業に応じた参照先を案内する。

外部調査メモ、企業別の観察、取得時の経緯は保存しない。採用する知見は目的・適用条件・具体的な判断基準を持つ規則へ整理する。公式素材の取得元・確認日・ハッシュ・利用条件は更新に必要な運用情報として管理する。
