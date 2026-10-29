# 出力例

すべて架空の提案であり、実企業の構成や稼働済みの事実ではない。名前付きの囲い、所属の入れ子、製品内の論理層、用途別レーン、横断機能を共通SVG部品で示す。

| 例 | 表現すること | 編集元・プレビュー |
| --- | --- | --- |
| [業務分析基盤](analytics-platform/README.md) | システムとCSV、Raw・Mart、複数利用先、実行制御 | [JSON](analytics-platform/model.json)・[SVG](analytics-platform/diagram.svg) |
| [Google Cloudへの具体化](google-cloud-analytics/README.md) | 同じ接続へ公式アイコンと製品領域を適用 | [JSON](google-cloud-analytics/model.json)・[SVG](google-cloud-analytics/diagram.svg) |
| [用途別の基盤](purpose-based-platform/README.md) | プロダクト用と社内分析用の並行レーン | [JSON](purpose-based-platform/model.json)・[SVG](purpose-based-platform/diagram.svg) |
| [契約・責任境界](contract-boundary/README.md) | 提供側・分析側、公開インターフェース、品質検査 | [JSON](contract-boundary/model.json)・[SVG](contract-boundary/diagram.svg) |
| [BI・AI・業務活用](data-ai-platform/README.md) | 共通データからの分岐、業務アプリへの結果提供 | [JSON](data-ai-platform/model.json)・[SVG](data-ai-platform/diagram.svg) |
| [接続元・接続先の入力](endpoints/README.md) | 方式・形式等を記述する入力専用例 | [JSON](endpoints/model.json) |

## 再生成と検証

```sh
python3 examples/render.py
python3 scripts/validate_diagrams.py
python3 scripts/validate_assets.py
```

`model.json`は[SVGモデル v2](../docs/guides/input-model.md)、`diagram.svg`は単体表示できる編集可能な図。[render.py](render.py)は[共有描画部品](../scripts/diagram.py)を呼び出す。座標を持つ部品の組み合わせであり、自動配置エンジンではない。ネットワークアクセスは行わない。

現行版の再生成対象はSVGのみ。最初の2例に残るdraw.io・Mermaidは移行前の保存版であり、現行SVGと配置・ラベルを同期していない。旧形式を更新する指示があるまで再生成しない。構成の正本はv2 JSONとそのSVG。

公式素材は[assets/icons](../assets/icons/README.md)、描画値は[design-tokens.json](../assets/design-tokens.json)を参照する。再生成自体は最新版確認を代行しない。
