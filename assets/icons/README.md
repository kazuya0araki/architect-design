# 公式アイコン

[選択・配置ルール](../../docs/foundations/iconography.md)と[取得・更新手順](../../docs/governance/icon-updates.md)に従う。全アイコン素材は本ディレクトリを正本とし、作図例や将来のSkillsへ元素材を分散させない。図を単体表示するための埋め込みは許容する。

## 収録内容

| 提供元 | 配置 | 収録範囲 | 確認日 |
| --- | --- | --- | --- |
| Google Cloud | `google-cloud/` | 現行Core 19点・カテゴリ26点の全SVG（合計45点） | 2026-10-18 |
| AWS | `aws/` | 公式Icon package（07312026）のSVG 1,852点 | 2026-10-17 |
| Material Symbols | `material-symbols/` | 組織・汎用DB・人の3点（Outlined） | 2026-10-27 |
| Google Workspace | `google-workspace/` | ブランドロゴ1点・主要アプリ19点（計20点） | 2026-10-27 |
| Apache | `apache/` | Airflow・Beamの公式SVG各1点、Icebergの文字なしPNG 1点 | 2026-11-02 |
| Microsoft Office / Microsoft 365 | `microsoft-office/` | Word・Excel・PowerPoint・Outlook・OneNote・OneDrive・Teamsの公式SVG計7点 | 2026-11-02 |
| dbt | `dbt/` | 公式シンボル1点 | 2026-10-27 |
| Azure | `azure/` | 公式Public Service Icons V24のSVG 714点 | 2026-10-17 |

各提供元のアーカイブ内のディレクトリ構造・ファイル名・バイト列を保持する。同じ素材の重複画像、OSの管理ファイル、ツールキットは収録しない。公式の文字なし素材がPNGのみの場合は、PNGを変換・加工せず収録できる。Google Cloudは`core/`と`categories/`の2セットのみを収録し、Legacyやルート直下の互換コピーは置かない。

Google Cloudの45点は「現行配布セットに含まれる全SVG」であり、全製品の専用アイコンではない。複数の製品へ共通のカテゴリ素材を適用する。公式ガイドにはGeminiが記載される一方、現在リンクされるCore ZIPには対応するSVGがないため、Gemini専用素材は未収録として扱う。Vertex AIの素材をGeminiへ読み替えない。選択時は[公式ガイド](https://services.google.com/fh/files/misc/google-cloud-product-icons.pdf)と[運用ルール](../../docs/governance/icon-updates.md)に従う。

## 探し方と取得記録

[manifest.json](manifest.json)を提供元またはファイル名で絞り込み、`path`を使って参照する。`product`は素材を探すための名称であり、各提供元の新規収録分では配布ファイルの名前をもとに記録している。正式な製品表示名は提供元で確認する。

各ファイルの取得元・確認日・配布版・SHA-256、配布アーカイブのSHA-256はmanifestに記録する。元の公式素材は改変しない。リポジトリが独自の権利やライセンスを付与するものではない。

- [Google Cloud公式配布元](https://cloud.google.com/icons?hl=en)
- [AWS公式配布元・利用案内](https://aws.amazon.com/architecture/icons/)
- [Azure公式配布元・利用条件](https://learn.microsoft.com/ja-jp/azure/architecture/icons/)

自動更新は行わない。図の作成・改訂・公開前に公式配布元を再確認し、該当素材を照合する。取得済みという理由だけで最新版と判断しない。

## 検証

`python3 scripts/validate_assets.py`でファイルとmanifest、サンプルの参照を照合する。manifestの`collections`にはGoogle Cloudの配布セットごとの件数・配布物ハッシュ・確認日を記録する。オンラインの配布更新検知は行わない。

汎用素材は[Google Fonts](https://fonts.google.com/icons)のMaterial Symbolsを使い、[Apache 2.0ライセンス](material-symbols/LICENSE)を同梱する。製品ロゴは汎用素材と分け、Google Sheetsの取得元はmanifestの公式ページを参照する。

Data Studioは既存の`google-cloud/categories/Category Icons/Data Analytics/SVG/DataAnalytics-512-color.svg`を製品名と併記して使う。本システムの選択ルールとしてカテゴリ素材へ統一し、独立したData Studio素材や互換コピーを追加しない。Lookerの固有アイコンとは区別する。

Google Workspaceは[公式サイト](https://workspace.google.com/)のブランドロゴと、主要アプリ（Gmail、Calendar、Drive、Docs、Sheets、Slides、Meet、Chat、Forms、Sites、Keep、Tasks、Vids、NotebookLM、AppSheet、Workspace Studio、Admin Console、Voice、Gemini）のSVGを収録する。全Workspace関連製品の網羅を意味しない。

dbtのシンボルは[公式ドキュメントのリポジトリ](https://github.com/dbt-labs/docs.getdbt.com/tree/current/website/static/img/icons)から取得し、[商標ポリシー](https://www.getdbt.com/dbt-assets/dbt-trademark-guidelines)のデータスタック図での利用条件を適用する。

Apacheの素材は[Airflow公式](https://airflow.apache.org/community/)・[Iceberg公式](https://iceberg.apache.org/)・[Beamのロゴ配布ページ](https://beam.apache.org/community/logos/)から取得する。[Apacheの商標ポリシー](https://www.apache.org/foundation/marks/)を参照し、素材のバイト列・名称を保持する。Icebergはテーブル形式であり、実行エンジンとして表示しない。

Microsoft Office / Microsoft 365の素材は[Microsoft公式の製品一覧](https://www.microsoft.com/en-us/microsoft-365/products-apps-services)が参照するFY26アイコンを取得し、元SVGのバイト列を保持する。拡張子のない配信URLをmanifestに記録する。上記7製品の収録であり、Office関連製品全体の網羅ではない。[Microsoftの商標ガイドライン](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks)を参照する。

Icebergは公式サイトで配布する文字なしの[iceberg-logo-icon.png](apache/iceberg/iceberg-logo-icon.png)（600×600）を採用する。文字付きロゴから切り抜いたり、独自にSVG化したりしない。共有SVG描画部品は元PNGを`image/png`として埋め込む。
