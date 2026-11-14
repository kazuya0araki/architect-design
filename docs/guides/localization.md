# ローカライズ

英語を基準言語とし、日本語を翻訳ファイルで上書きする。既定の出力言語は日本語。構成（ID・接続・所属・公式素材・基本座標）は言語間で共通で、言語ごとにモデルを複製しない。

## ファイル構成

```text
config/diagram.json                  # 言語定義と既定の出力言語
examples/<example>/model.json        # 英語の基準文言を持つ構成モデル
examples/<example>/locales/ja_jp.json # ID別の日本語文言
examples/<example>/diagram.svg       # 英語の出力
examples/<example>/diagram.ja_jp.svg # 日本語の出力
```

`config/diagram.json`は`source_locale`（`en`）、`default_output_locale`（`ja-JP`）、言語ごとの`suffix`（ファイル名）・`lang`（SVGの`lang`／`xml:lang`）・任意の`font_family`を定義する。基準言語のサフィックスは空で、日本語は`ja_jp`、SVGの言語タグは`ja-JP`。

## 出力言語の決め方

優先順位は 明示指定 > 案件別設定 > リポジトリ既定設定。

| 指定 | 内容 |
| --- | --- |
| `--locale en` / `--locale ja-JP` | 明示指定。`ja_jp`も受け付ける。未定義の言語はエラー |
| `--all-locales` | 設定済みの全言語を生成する。`--locale`・`--output`とは併用できない |
| `<モデルの隣>/locale.json`の`default_output_locale` | 案件別設定。任意 |
| `config/diagram.json`の`default_output_locale` | リポジトリ既定（日本語） |

```sh
cargo run --release -- render path/to/model.json                  # 日本語 → diagram.ja_jp.svg
cargo run --release -- render path/to/model.json --locale en      # 英語   → diagram.svg
cargo run --release -- render path/to/model.json --all-locales    # 両方
cargo run --release -- examples --all-locales                     # 全examplesを全言語で再生成
```

## 出力形式

SVGに加え、モデルの`exports`で宣言したMermaid（`diagram.mmd`）・draw.io（`diagram.drawio`）も同じモデルと翻訳ファイルから生成する。言語サフィックスはSVGと同じ（`diagram.ja_jp.mmd`、`diagram.ja_jp.drawio`）。`render --format svg,mermaid,drawio`で個別に出力でき、`examples`は宣言済みの形式をすべて再生成する。`validate-diagrams`は宣言した形式が全言語にあり、全ノード・領域・接続を含むことを確認する。

## READMEの扱い

`docs/`と各READMEは説明文であり、図の文言とは別に日本語で保守する。READMEは日本語の図（`diagram.ja_jp.svg`）を表示し、英語の図・編集元・翻訳ファイルへリンクする。READMEを言語別に複製すると前提・未確認事項が言語間でずれるため、複製は英語の読み手が必要になった時点で`README.en.md`として追加する。図に表示しない前提・未確認事項は、READMEを正本とする。

## 翻訳ファイル

`locales/ja_jp.json`はIDをキーにして、表示する文言だけを上書きする。

```json
{
  "locale": "ja-JP",
  "title": "…",
  "purpose": "…",
  "regions": {"<region_id>": {"label": "…", "scope": "…"}},
  "nodes": {"<node_id>": {"name": "…", "role": "…", "display_name": "…", "display_role": "…"}},
  "connections": {"<connection_id>": {"label": "…", "data": "…", "method": "…", "frequency": "…"}},
  "untranslated": ["nodes.<node_id>.name", "brand_notice"],
  "adjustments": {"nodes": {"<node_id>": {"name_size": 24}}}
}
```

- 基準モデルに文言がある翻訳対象は、翻訳するか`untranslated`に列挙する。製品名・公式表記は`untranslated`へ`section.id.field`で明示し、訳さない。
- `untranslated`は意図した無翻訳の宣言であり、翻訳済みの項目や存在しない項目を載せるとエラーになる。基準と同じ文言を翻訳として書くこともエラー。
- 基準モデルの文言は英語に限る。日本語を含む基準文言はエラー。日本語出力の翻訳には日本語の文字を含める。
- 確定した表示訳は各図の翻訳ファイルへ記録する。[用語集](../foundations/glossary.md)は翻訳時の判断基準であり、描画時の文字列一括置換には使わない。
- 非表示のメタデータ（`scope`・`unknowns`・ノードの`purpose`等）は翻訳対象外で、基準言語のまま保持する。

## 局所的な調整

構成は変えず、表示文言・改行（`\n`）・文字サイズで収める。収まらない場合に限り、翻訳ファイルの`adjustments`で次の値だけを言語別に上書きできる。`geometry`・`points`・接続元／接続先・所属は変更できない。

| 区分 | 使える項目 |
| --- | --- |
| `regions` | `label_size` |
| `nodes` | `name_size`、`role_size` |
| `connections` | `label_position`、`label_width` |

基準モデル側に局所的な文字サイズを置いた場合（例：契約・責任境界の`name_size: 20`）、日本語側で同じ項目を`adjustments`で戻せる。

## 検証

`cargo run --release -- validate-diagrams`は全モデルを全言語で次の観点から確認する。

- 翻訳の欠落、未知のID・項目・キー、不要な`untranslated`、不正な言語指定
- 基準文言への日本語の混入、日本語出力での未翻訳の混在
- SVGの`lang`／`xml:lang`、言語別フォント（日本語はNoto Sans JP等を含むこと）
- XMLエスケープの往復、折り返しで文字が欠けないこと、英単語が行をまたいで分割されないこと
- 言語間でノード・接続・所属・アイコン・座標が一致すること
- 既存の領域・経路・文字範囲・線との交差の検査（言語ごと）
- 対応するモデルまたは設定済み言語のない翻訳ファイル

自動チェックの文字幅は概算である。最終成果物は両言語とも掲載幅（例：960px）のブラウザで、文字切れ・重なり・可読性を確認する。
