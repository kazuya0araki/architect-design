# 編集元と画像出力の選択

本リポジトリでは、人が配置を調整する図の編集元にはdraw.io XML、AI Agentが描画位置を直接制御する画像生成にはSVGを推奨する。Markdown内の簡潔な構成共有にはMermaidを使う。以下は本リポジトリの推奨である。

| 形式 | 適した用途 | PNG・JPEGへの経路 | 注意点 |
| --- | --- | --- | --- |
| `.drawio`（非圧縮XML） | 人がノードや接続、配置を編集する構成図 | draw.ioからPNG・JPEGへ直接エクスポート | XMLとして差分を残せる。描画には対応エディタが必要 |
| `.svg` | 色、文字、座標をAI Agentが細かく制御する図 | SVG対応レンダラーでラスタライズ | 日本語フォントを揃える。移植性を重視する場合は外部画像やHTMLの埋め込みを避ける |
| Mermaid | Markdownで管理する構成図 | 対応レンダラーでSVG・PNG等に描画。JPEGは対応ツールで変換 | 自動配置のため座標の厳密な制御には向かない |

SVGは図形・線・文字をベクターとして保持するため、出力サイズを変えて描画できる。PNGはラスタ画像なので、拡大しても情報量は増えない。編集元を必ず残す。

## 出力画像の推奨

- 標準はPNG。文字と細い線を含む構成図では可逆圧縮が適しており、透明背景も扱える。
- JPEGは受け渡し先から指定された場合などに使う。非可逆圧縮による文字や線のにじみに注意し、透明背景は白で合成する。
- PNGの初期設定は掲載時の幅・高さの2倍。必要な読みやすさとファイルサイズに合わせて調整する。
- エクスポート後に、日本語の欠落、文字切れ、線の交差、背景、余白を実際の画像で確認する。
- draw.ioを編集元として使った場合は、PNG・SVGへの書き出し後も`.drawio`を保存する。v2のSVG専用生成ではモデルJSONとSVGを編集元とする。

v2の共有SVG部品はSVGのみを再生成する。draw.io・Mermaidの既存ファイルは保存版であり、現行SVGと同期する指定がある場合だけ更新する。SVGのみの成果物へdraw.ioの追加を必須にしない。

## draw.io例

[業務分析基盤.drawio](../../examples/analytics-platform/diagram.drawio)をdraw.ioで開き、「ファイル → 形式を指定してエクスポート」からPNGまたはJPEGを選択する。スケールや余白などの設定はエクスポート画面で調整する。目的・前提・未確認事項は[説明文](../../examples/analytics-platform/README.md)を参照する。

## SVG例

公開用のトンマナは[Google Cloudの構成例](../../examples/google-cloud-analytics/README.md)を参照する。公式アイコンを埋め込んだSVGと、元素材・取得記録を`assets/icons/`に保存している。

[業務分析基盤.svg](../../examples/analytics-platform/diagram.svg)は1760×1080の`viewBox`を持つ編集可能なソース。ブラウザで直接表示できる。外部画像、外部CSS、`foreignObject`は使わず、ノードと接続にはIDを付けている。日本語フォントにはNoto Sans JP、ヒラギノ角ゴ、游ゴシックなどを指定し、描画環境にも対応フォントを用意する。

PNG・JPEGを出力する際はSVGを対応レンダラーで描画する。2倍サイズの場合は3520×2160とし、JPEGは白背景で出力する。実際の変換ツールはまだリポジトリに同梱していない。

## 参照

保存・出力形式の対応は以下の公式ドキュメントで確認する。画質、運用、SVG設計は本デザインシステムの規則に従う。

- [draw.io: Supported export formats](https://www.drawio.com/docs/manual/export/export-diagram/)
- [draw.io: Choose where to store your diagram files](https://www.drawio.com/docs/getting-started/save-diagram-files/)
