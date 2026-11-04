# 再利用する部品群

完成図のコピーだけでなく、適用条件を持つ小さな構成を再利用する。正本は[examples/fragments](../../examples/fragments/README.md)、組み立ては[scripts/components.py](../../scripts/components.py)の`instantiate`を使う。

| 部品群 | 適用条件 | 差し替え箇所・接続口 | 注意 |
| --- | --- | --- | --- |
| `file-receipt` | ファイルを受けて確認・保存する構成がある | ファイル名・処理名・保存先名、`input`と`output` | 検査処理が存在しなければこの構成を採用しない |
| `raw-mart` | RawとMartを区別する構成がある | 保存先名・層名、`input`と`output` | Raw→Martの接続は抽象化であり、中間処理がないという保証ではない |
| `consumers` | BI・表計算・人への提供を検討する | 利用先名、`bi`・`spreadsheet`・`analyst` | 3利用先を必須にしない。外部への接続は含めない |

```python
part = instantiate(fragment, 'sales', offset=(64, 112),
                   labels={'file': '売上CSV'})
model['nodes'].extend(part['nodes'])
model['regions'].extend(part['regions'])
model['connections'].extend(part['connections'])
# part['ports']['output']はsales_storage。外部への経路は呼び出し側で指定する。
```

ローカルIDはインスタンス名で名前空間化し、所属・接続・fitの参照も更新する。座標、線ラベル、分岐点は同じ量だけ移動する。入力部品は変更しない。`labels`は表示名だけを差し替え、元の名前・役割を保持する。製品やデータ経路の自動選択は行わない。

接続口は部品中のノード・領域のIDである。特定の辺座標へ自動接続はしない。構成が異なる場合は展開後のモデルを明示的に編集し、通常の図の検証を行う。[見本のビルダー](../../examples/build_catalog.py)は毎回部品からJSONを生成し、そのJSONからSVGを描画する。
