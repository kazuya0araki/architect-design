# 出力例

すべて架空の提案であり、実企業の構成や稼働済みの事実ではない。名前付きの囲い、所属の入れ子、製品内の論理層、用途別レーン、横断機能を共通SVG部品で示す。

| 例 | 表現すること | 編集元（英語基準）・日本語文言・プレビュー |
| --- | --- | --- |
| [業務分析基盤](analytics-platform/README.md) | システムとCSV、Raw・Mart、複数利用先、実行制御 | [JSON](analytics-platform/model.json)・[日本語](analytics-platform/locales/ja_jp.json)・[SVG（日本語）](analytics-platform/diagram.ja_jp.svg)・[SVG（英語）](analytics-platform/diagram.svg) |
| [Google Cloudへの具体化](google-cloud-analytics/README.md) | 同じ接続へ公式アイコンと製品領域を適用 | [JSON](google-cloud-analytics/model.json)・[日本語](google-cloud-analytics/locales/ja_jp.json)・[SVG（日本語）](google-cloud-analytics/diagram.ja_jp.svg)・[SVG（英語）](google-cloud-analytics/diagram.svg) |
| [用途別の基盤](purpose-based-platform/README.md) | プロダクト用と社内分析用の並行レーン | [JSON](purpose-based-platform/model.json)・[日本語](purpose-based-platform/locales/ja_jp.json)・[SVG（日本語）](purpose-based-platform/diagram.ja_jp.svg)・[SVG（英語）](purpose-based-platform/diagram.svg) |
| [契約・責任境界](contract-boundary/README.md) | 提供側・分析側、公開インターフェース、品質検査 | [JSON](contract-boundary/model.json)・[日本語](contract-boundary/locales/ja_jp.json)・[SVG（日本語）](contract-boundary/diagram.ja_jp.svg)・[SVG（英語）](contract-boundary/diagram.svg) |
| [BI・AI・業務活用](data-ai-platform/README.md) | 共通データからの分岐、業務アプリへの結果提供 | [JSON](data-ai-platform/model.json)・[日本語](data-ai-platform/locales/ja_jp.json)・[SVG（日本語）](data-ai-platform/diagram.ja_jp.svg)・[SVG（英語）](data-ai-platform/diagram.svg) |
| [接続元・接続先の入力](endpoints/README.md) | 方式・形式等を記述する入力専用例 | [JSON](endpoints/model.json) |

## 表示レベル・部品・接続方式の見本

| 例 | 適用先 |
| --- | --- |
| [全体像](platform-context/README.md) | context。基盤内部を展開しない |
| [部品の状態](node-specimens/README.md) | 標準・長文・アイコンなし・横並び・入れ子 |
| [再利用部品の組み合わせ](component-catalog/README.md) | 部品JSONから同じ描画ルールで組み立てる |
| [取り込み方式](ingestion-modes/README.md) | バッチ・ストリーミング・CDCを区別する |
| [外部照会](federated-query/README.md) | データをコピーせず照会し、要求と結果を分ける |
| [ネットワーク配置](deployment-boundary/README.md) | detail/deployment。所属と実配置を分ける |
| [実行基盤と併用ソフトウェア](runtime-software/README.md) | Airflow・Beam・dbt・Icebergの役割を併記する |

再利用する部品は[fragments](fragments/README.md)。これらは説明用の架空の構成であり、案件に存在しない要素を追加するための必須構成ではない。

## 再生成と検証

```sh
cargo run --release -- examples                  # 日本語（diagram.ja_jp.svg）
cargo run --release -- examples --locale en      # 英語（diagram.svg）
cargo run --release -- examples --all-locales    # 全言語。コミット前はこれを使う
cargo run --release -- validate-diagrams
cargo run --release -- validate-assets
```

`model.json`は英語を基準とする[SVGモデル v2](../docs/guides/input-model.md)、`locales/ja_jp.json`はID別の日本語文言。`diagram.svg`（英語）と`diagram.ja_jp.svg`（日本語）は単体表示できる編集可能な図で、ノード・接続・所属・アイコン・座標は共通である。言語の扱いは[ローカライズ](../docs/guides/localization.md)を参照する。`cargo run --release -- examples`は[部品の組み立て](../src/catalog.rs)と[共有描画部品](../src/render.rs)を呼び出す。座標を持つ部品の組み合わせであり、自動配置エンジンではない。ネットワークアクセスは行わない。

再生成対象はSVGと、モデルの`exports`に宣言した形式（最初の2例のdraw.io・Mermaid）。いずれもv2 JSONと翻訳ファイルから生成し、言語ごとに`diagram.ja_jp.drawio`・`diagram.ja_jp.mmd`のようにサフィックスを付ける。構成の正本はv2 JSON。

公式素材は[assets/icons](../assets/icons/README.md)、描画値は[design-tokens.json](../assets/design-tokens.json)を参照する。再生成自体は最新版確認を代行しない。
