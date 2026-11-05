# 用語集

作図と翻訳の際に表記を揃えるための判断基準。現在の出力は日本語を基本とする。英語の基準文言・日本語翻訳ファイルによるローカライズは今後の対応であり、この用語集だけでは出力言語やファイル名は変わらない。

## 適用ルール

- 同じ概念には同じ語を使う。図中の短いラベルと付随説明で意味を変えない。
- 図の意味に合う語だけを選ぶ。既存文言を機械的に一括置換しない。確定した表示文言は対象図のモデル、将来の翻訳ファイルで管理する。
- 製品名・商標・API名は公式表記を維持する。Google Cloud、BigQuery、Cloud Run、Apache Airflow、Apache Iceberg、Apache Beam、dbt等を独自に訳さない。
- Raw・Staging・Mart、Bronze・Silver・Gold等の層名は案件の定義を優先し、用語集を理由に層の追加・改名・同一視をしない。
- 未確認の方式・頻度・機能を、翻訳やラベル短縮の過程で補わない。
- 略語は読み手に応じ、付随説明で初出時に正式名称を添える。文字を縮小するためだけに新たな略語を作らない。

## 領域・役割

| 英語 | 日本語の標準表記 | 使い分け |
| --- | --- | --- |
| Data platform | データ基盤 | 本システムの図中の標準表記 |
| Data sources | 接続元 | システム・ファイル・イベント等を含む |
| Source systems | 接続元システム | システムに限る場合。ファイルの総称にはしない |
| Ingestion | 取り込み | データを基盤へ受け入れる処理 |
| File reception | ファイル受領 | ファイルを受け取る処理 |
| Storage | 保存 | 保存機能。保存先の名称とは区別する |
| Transformation | 変換 | データの変換処理。「加工」は複数処理の包括的説明に使える |
| Serving | データ提供 | 利用先へデータを提供する機能 |
| Consumption | データ活用 | 提供されたデータの利用 |
| Serving and consumption | 提供・活用 | 外側の機能領域に使う短い見出し |
| Destinations | 接続先 | 提供先となるツール・システム・人等 |
| Orchestration | 実行管理 | ジョブやワークフローの実行制御 |
| Data quality | データ品質 | 品質の検査・管理 |
| Data catalog | データカタログ | メタデータの探索・管理 |
| Monitoring | 監視 | 稼働状態や異常の監視 |
| Access control | アクセス制御 | 認証だけを意味しない |
| Cross-cutting capabilities | 横断機能 | 品質・実行管理・監視等 |
| Ad hoc analysis | アドホック分析 | 都度行う分析。手動SQLに限定しない |
| Manual SQL execution | 手動SQL実行 | 人がSQLを実行する操作 |

## データ経路・方式

| 英語 | 日本語の標準表記 | 使い分け |
| --- | --- | --- |
| Data transfer | データ転送 | データを移動すること |
| Replication | 複製 | コピーを作ること。照会と区別する |
| Batch ingestion | バッチ取り込み | まとめて取り込む方式。日次等の頻度は別情報 |
| Streaming ingestion | ストリーミング取り込み | 継続するイベント等の取り込み。遅延保証を意味しない |
| Change data capture (CDC) | 変更データキャプチャ（CDC） | 図中はCDCでもよい。実行頻度とは別概念 |
| Query request | クエリ実行要求 | 実行指示。データの結果とは別経路 |
| Query result | クエリ結果 | 実行によって返るデータ |
| Federated query | 連合クエリ | 外部への照会。永続コピーの有無は実構成で確認する |
| Metadata | メタデータ | スキーマ・カタログ等。業務データとは区別する |
| Data sharing | データ共有 | 具体的な方式が転送・照会・カタログ参照のどれかを別途示す |
| Table format | テーブル形式 | Apache Iceberg等。実行エンジンではない |
| Execution engine | 実行エンジン | 実際に処理を実行する仕組み |
| Runtime | 実行環境 | ツールが動く基盤。ツール自体と区別する |

## 所属・状態

| 英語 | 日本語の標準表記 | 使い分け |
| --- | --- | --- |
| Project | プロジェクト | クラウドの所属単位。ネットワーク配置ではない |
| Responsibility boundary | 責任境界 | 管理・責務の範囲 |
| Network boundary | ネットワーク境界 | 明示したネットワークの範囲 |
| Private endpoint | プライベートエンドポイント | マネージドサービス自体の配置と混同しない |
| Current state | 現状 | 確認済みの現在の構成 |
| Proposal | 提案 | 設計上の選択であり稼働事実ではない |
| Unconfirmed | 未確認 | 根拠が未確認の事項 |
| Out of scope | 対象外 | 図で説明しない範囲 |

用語を追加・変更する場合は、意味と適用条件を添え、[ラベル](../components/annotation.md)・[情報モデル](../guides/input-model.md)・関連する部品との整合を確認する。
