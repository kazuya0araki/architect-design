# 再利用部品のJSON

これは完成図の入力ではなく、製品を指定しない部品定義。適用条件・差し替え・接続口は[ルール](../../docs/patterns/reusable-fragments.md)を参照する。

- [ファイル受領](file-receipt.json)
- [Raw・Mart](raw-mart.json)
- [BI・表計算・人](consumers.json)

[ビルダー](../build_catalog.py)がこれらを読み込み、[組み合わせ見本](../component-catalog/README.md)のJSONを生成する。各部品には内部の接続だけを含め、部品間の接続は自動追加しない。
