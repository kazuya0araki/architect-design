# architect-design

AI Agentがデータプラットフォームのアーキテクチャ図を一貫した表現で作成するためのデザインシステム。

- [ナレッジの全体案内](docs/README.md)
- [最初の1枚を作る](docs/introduction/quick-start.md)
- [出力例](examples/README.md)
- [公式アイコン](assets/icons/README.md)
- [AI Agent向け参照手順](AGENTS.md)

## 構成

```text
docs/
  introduction/   目的・使い方
  principles/     正確性・読みやすさ
  foundations/    色・文字・余白・アイコン
  components/     ノード・論理層・境界・接続・注記
  patterns/       構成パターンと適用条件
  guides/         入力・生成・レビュー・出力
  governance/     規則と公式素材の更新
examples/         説明と編集可能な出力例
assets/           公式アイコン素材・取得記録・描画トークン
scripts/          共通SVG部品と素材・図の検証
tests/            所属・接続・描画部品の振る舞い検証
```

共通ルールは`docs/`を正本とし、作業手順や将来のSkillsから参照する。現時点ではドキュメントとMermaid・draw.io・SVGの例を提供し、座標を指定する共通SVG部品と図の検証を提供する。自動配置エンジンやSkillsは含まない。

案件ごとに生成した構成図や入力資料は`architectures/`に保存し、Git管理対象から除外する。デザインシステムの作例は`examples/`で管理する。
