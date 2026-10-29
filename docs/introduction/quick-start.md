# 最初の1枚を作る

1. [概要](overview.md)と[正確性](../principles/accuracy.md)、[読みやすさ](../principles/clarity.md)を読む。
2. [情報モデル](../guides/input-model.md)に沿って目的、構成要素、接続、前提、未確認事項を整理する。
3. [構成パターン](../patterns/README.md)を選び、必要な[作図部品](../README.md)の規則を確認する。
4. [生成手順](../guides/generation.md)に従い、[論理構成例](../../examples/analytics-platform/README.md)または[公式アイコン例](../../examples/google-cloud-analytics/README.md)を参考に図を作る。
5. [レビュー](../guides/review.md)後、[出力ガイド](../guides/export.md)に従って編集元と説明を添える。

製品が決まっている場合は、作成・改訂・公開前に[公式アイコンの更新手順](../governance/icon-updates.md)を実施する。規則の全文を作業ファイルへ複製せず、必要なページを参照する。

SVGを生成する場合はサンプルのv2 JSONを編集し、`python3 scripts/diagram.py path/to/model.json --output path/to/diagram.svg`を実行する。`python3 scripts/validate_diagrams.py`で構造・配置を点検し、ブラウザでも確認する。
