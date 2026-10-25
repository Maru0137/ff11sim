# インライン script のモジュール化（解消済み）

**2026-08-11 解消。** 本ドキュメントは経緯の記録として残す。

## 現状（解消後）

| ファイル | インライン script |
|---|---|
| `web/index.html` | 2 行（`js/main.js` の import と `startApp()` 呼び出しのみ） |
| `web/search.html` | 2 行（`js/search-page.js` の import と `startSearchPage()` 呼び出しのみ） |

かつて index.html に 1,637 行・search.html に 383 行あったインライン script は、
`web/js/` 配下の ES モジュールに分割された。モジュール構成・レイヤ・状態管理の
規約は各モジュール冒頭のコメントと git 履歴（`[Refactor]` 一連のコミット）を参照。

分割時に整備した検証:

- `npm run lint` — 全モジュールが no-undef の対象
- `npm run test:smoke` — 8 本（オーグメント選択・スロット検索/セット保存・
  キャラ作成フォーム・共有閲覧モード・ログイン配線を追加）
- 各段階で `cargo test` による回帰確認

## 元の問題（記録）

装備ロジックの Rust 移植時、削除した `equip-stats.js` の `WEAPON_SKILL_KEYS` を
index.html 内のインライン script が参照したままになり、本番でステータスが
全て 0 になった（修正: `ee48e90`）。Rust のテスト 177 件は全て通っていた。
インラインである限りユニットテストから import できず、ESLint も
`eslint-plugin-html` 経由でしか見られず、3,804 行の HTML は diff レビューも
難しかった。

## 残課題

- `js/equip-bonuses.js`（事故現場だった集計ロジック）は DOM 非依存の純関数に
  なったが、WASM 関数に直接依存するためユニットテストは未追加。
  Vite / Vitest 導入（[ADR 0011](../adr/0011-vite-build-toolchain.md)）後に
  追加するのが自然。
- `js/search-page.js` のジョブ・スロット和名マップ（戦/モ/…、遠隔/耳/…）は
  constants.js の共有テーブル（戦士/レンジ/左耳…）と**表示テキストが異なる**
  結果テーブル用の略記であり、共有テーブルへの置き換えは表示変更を伴うため
  見送った。統一するなら略記フィールドをデータ側に追加してから。
