# 公式アイコンの取得と更新

## 公式の入口

| 提供元 | 確認先 | 選択ルール |
| --- | --- | --- |
| Google Cloud | [Product icons](https://cloud.google.com/icons?hl=en) | 現行のCore product iconsを優先し、該当製品がなければ公式カテゴリ素材とラベルを検討する。Legacy console iconsは通常使わない |
| AWS | [Architecture icons](https://aws.amazon.com/jp/architecture/icons/) | 公式ページから現在リンクされるIcon packageを使う。外部ツールの内蔵セットは最新版と照合する |
| Azure | [Azure Architecture icons](https://learn.microsoft.com/ja-jp/azure/architecture/icons/) | 公式SVGセットと一般ガイドラインを確認する |
| その他のSaaS | 各社公式のブランド、メディアキット、開発者ドキュメント | アーキテクチャ用アイコンがあれば優先し、なければ使用条件に沿う公式製品ロゴと製品名を使う |

Google Cloudは現行のCore・カテゴリの2セットを保管し、Legacyと初期登録時のルート直下ファイルは収録しない。Coreは`google-cloud/core/`、カテゴリは`google-cloud/categories/`に配布アーカイブ内のパスとバイト列を保持する。サンプルもこの正本を参照し、互換パス用の複製は作らない。

製品ごとの選択は[公式Product icons overview](https://services.google.com/fh/files/misc/google-cloud-product-icons.pdf)に従う。現行体系では複数製品が同じカテゴリ素材を共有するため、専用SVGがないことだけで欠落と判断しない。Dataflow・Dataproc・Pub/Sub等はData Analytics、Cloud SchedulerはDeveloper Tools、WorkflowsはIntegration Servicesとして製品名と併記する。公式に専用アイコンが指定されていればCoreを使う。

「全件収録」は公式ページがリンクする現行2セットの全SVGを意味し、全製品に専用素材があることを意味しない。ガイドに記載があっても配布ZIPに対応する素材がない場合は、名称だけの推測や旧素材への置換をせず、未確認として記録する。ZIPの内容と適用ガイドの更新は別々に確認する。

アイコン体系の変更と旧版の混入を考慮し、参考記事の掲載年や見た目の馴染みを最新版の判定基準にしない。ツールの内蔵素材も公式配布物と照合する。

## 作成・更新の手順

1. 図に登場する製品を確定する。論理層を製品と誤認してアイコンを付けない。
2. 公式ページを開き、現在の推奨セットと利用条件、変更履歴を確認する。
3. 公式ページから辿った配布URLから素材を取得する。SVGを優先し、既存キャッシュは配布物と照合する。
4. 全アイコン素材は`assets/icons/<provider>/`に保存する。名称・バイト列を保持する。公式コレクションを取り込む場合はSVGと付随する利用条件を対象とし、PNGの重複、OSの管理ファイル、不要なプレゼンテーション等は含めない。
5. `assets/icons/manifest.json`に提供元、製品名、公式ページ、ダウンロードURL、アーカイブ内パス、取得日、最終確認日、配布版、ファイルと配布物のSHA-256、利用条件の参照先を記録する。版が不明なら`null`とし、独自に版番号を付けない。
6. SVG・draw.ioには取得した素材を埋め込むかローカル参照する。完成SVGは単体表示できる埋め込みを優先し、公式ファイル自体は改変しない。図が使う素材をmanifestと対応付ける。
7. 更新があれば同一製品の素材と使用図を置換し、描画確認を行う。変更がなくても最終確認日を更新する。

リモートの「latest」URLを実行時参照せず、確認した素材を固定保存する。これにより出力を再現でき、次の改訂では最新配布物と比較できる。過去の公開図は当時の記録を残し、再生成・再公開時に更新する。常時の自動同期は現時点で実装していない。

地域別の公式ページで配布リンクや更新案内を確認できない場合は、提供元の別言語の公式ページを確認する。AWSでは[英語の公式ページ](https://aws.amazon.com/architecture/icons/)も利用できる。確認結果は素材のmanifestに記録する。

描画時の選択・寸法・形状は[アイコン](../foundations/iconography.md)に従う。

## 整合性の確認

リポジトリのルートで`python3 scripts/validate_assets.py`を実行する。manifestと実ファイルの過不足、重複、SHA-256、SVG構文、現行コレクションの件数、サンプルの素材参照と埋め込みを確認する。オンラインの最新版確認は別途必要。

## 個別製品の選択

- Data Studio：本システムでは既存の`assets/icons/google-cloud/categories/Category Icons/Data Analytics/SVG/DataAnalytics-512-color.svg`へ統一する。公式に専用ロゴがあることと、このシステムで採用する素材は分けて扱う。独立素材は追加しない。
- Apache Airflow・Iceberg・Beam：各プロジェクトの公式サイトまたは公式の配布リポジトリから取得し、`assets/icons/apache/<project>/`へ保存する。商標の扱いはApacheの公式ポリシーに従う。
- Microsoft Office / Microsoft 365：公式の製品ページが参照する現行SVGを`assets/icons/microsoft-office/`へ保存する。Azureのアイコンや第三者のOfficeアイコン集で代替しない。

取得元・取得日・確認日・ハッシュ・利用条件は既存manifestの形式で記録する。ツールがサービスに内包される場合の表示は[実行基盤と併用ソフトウェア](../components/runtime-software.md)に従う。

公式の文字なしアイコンがPNGのみの場合は、そのPNGを元のバイト列のまま採用できる。文字付きSVGからの切り抜きや独自のベクター化は行わない。manifestで形式と取得先を識別し、SVGへの埋め込みでは`image/png`を使う。Icebergはこの条件で文字なしPNGを採用する。
