# 入力の情報モデル

共通知識は本ディレクトリ、案件固有の構成・根拠・配置は案件のJSONへ保存する。ユーザー指定と確認済みの実装を区別し、不明な製品・頻度・権限を推定で埋めない。

## SVGモデル v2

[共有描画部品](../../scripts/diagram.py)が読み込む形式。実行時の構文・所属・接続チェックを提供する。座標は明示指定であり、自動配置エンジンではない。

| 要素 | 必須項目 | 任意項目 |
| --- | --- | --- |
| 図 | `schema_version: 2`、`id`、`title`、`canvas`、`regions`、`nodes`、`connections` | `purpose`、`status`、`source`、`outer_margin`、`publication_width`、`flow_direction`、`omitted` |
| 領域 | `id`、`label`、`kind`、`geometry`または`fit` | `parent_id`、`asset`、`header_height`、`label_size` |
| ノード | `id`、`name`、`role`、`geometry` | `region_id`、`stage_id`、`representation`、`icons`、`display_name`、`display_role`、`basis`、`project_id` |
| 接続 | `id`、`source`、`target`、`kind`、`points` | `purpose`、`data`、`method`、`frequency`、`label`、`label_position`、`label_width`、`junctions`、`return_path`、`basis` |

`geometry`は`[x,y,width,height]`、`points`は始点から終点の`[[x,y],...]`。IDは名前から独立した英小文字とアンダースコアを基本とし、領域・ノード・接続の間でも重複させない。

`region_id`は直接の所属、領域の`parent_id`は所属の入れ子を表す。`stage_id`は別軸の機能・工程分類で、`kind: stage`の領域を参照する。クラウド・プロジェクト・製品・チーム・工程を一つの値に混在させない。領域種別と見た目は[境界](../components/boundary.md)を正本とする。

`fit`は`{"members": ["node_id", "child_region_id"], "padding": 24}`。子領域から順に外接範囲を計算し、上部に見出し分の高さを加える。線の通路も必要な領域では明示的な`geometry`を使う。

`icons`は`[{"asset": "assets/icons/...svg"}]`。複数の場合も同じ一つの処理ノードであり、別の実行工程ではない。図形・ラベルの扱いは[ノード](../components/node.md)に従う。出典は`source`、指定による追加は`basis: user_requested`、図の対象外は`omitted`へ記録する。

`title`・`purpose`等はSVGの非表示情報と付随Markdownに使う。モデルに存在することは画像内へタイトルや注釈を追加する指定ではない。

## 接続元・接続先の情報

種類は[接続元](../components/source.md)・[接続先](../components/destination.md)に従う。

| 項目 | 格納先 | 意味 |
| --- | --- | --- |
| `endpoint_roles` | ノード | `source`、`destination`。同じ実体は両方の役割を持てる |
| `kind` | ノード | `system`、`database`、`file`、`storage`、`event`、`bi`、`spreadsheet`、`person`等。接続の`kind`とは別 |
| `product` / `purpose` | ノード | 製品名と用途。内製システムや人に製品は必須でない |
| `owner` / `interface` / `format` | ノード | 確認済みの担当、入出力口、ファイル形式 |
| `entity_id` | ノード | 左右へ複製して表示する同じ実体の共通ID |

原則は一実体一ノード。左右に複製する場合は一意な`id`と共通の`entity_id`を与える。見た目のグループ化と実体の統合を混同しない。既存の[入力だけの例](../../examples/endpoints/README.md)は描画を目的としないためv2への移行は必須ではない。

接続の`label`を省略した場合、指定済みの`data`・`method`・`frequency`を順に連結して表示する。空の`label`は表示を抑制する明示指定。表示する文字がある場合は、通路上の`label_position`も指定する。未指定の頻度や方式を自動生成しない。

## 表示レベル・配置・接続方式の追加項目

- 図の任意項目：`view_level`（`context`・`pipeline`・`detail`）、`diagram_type`（`context`・`data_flow`・`deployment`・`component_catalog`）。選択基準は[図の表示レベル](view-levels.md)。新規モデルでは両方を記載する。
- 領域の追加種別：`network`。`diagram_type: deployment`の場合にのみ使い、`scope`で対象を記録する。[境界](../components/boundary.md)に従う。
- 接続の任意項目：`interaction`（`transfer`・`query`・`result`・`metadata`）、`transfer_mode`（`batch`・`streaming`・`cdc`）。`transfer_mode`は`interaction: transfer`の場合のみ使う。
- `query`は`kind: control`、`transfer`・`result`・`metadata`は`kind: data`。未知の値や矛盾は検証エラーとする。未指定の旧v2はそのまま扱い、値を推定しない。

追加項目は非表示のSVGメタデータ・経路属性に保持する。図への見出し・説明の追加指示にはならない。方式等を表示する場合は既存の`label`・`method`・`frequency`を使う。

部品群を組み立てる場合は[再利用部品](../patterns/reusable-fragments.md)でモデルの一部を作成できる。展開後は同じv2モデルとして検証する。
