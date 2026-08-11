# React 段階的移行ロードマップ

[ADR 0012](../adr/0012-react-ui-framework.md)(React 採用)の移行順序の計画。
各フェーズは独立した PR として実施し、完了条件は共通で
`npm run lint` / `npm run typecheck` / `npm run test:smoke` の全通過
(スモークのセレクタ変更なし)。

## Phase 0 で確立した規約(実施済み: 2026-08-11)

最初の PR(基盤 + auth-ui + 検索ページ)で以下の規約を確立した。
以降のフェーズもこれに従う。

- **配置**: React / TS コードは `web/src/` 配下。レガシー層は `web/js/` に
  残り、移行の進行とともに縮小して最終的に消える
- **島マウント**: `createRoot(container).render(...)` によるウィジェット /
  ページ単位の部分マウント。マウント関数は旧実装と同名・同シグネチャを
  維持し、呼び出し側の差分を import パスに留める
- **既存ストアへの接続**: 購読可能な既存モジュール(`onAuthChange` 等)には
  `useSyncExternalStore` で接続する。ストア側は変更しない
- **WASM 境界**: `web/src/` からは型付きファサード `web/src/wasm.ts` を経由し、
  `../pkg/` を直接 import しない(CI では typecheck が wasm-pack より先に走る)。
  型は Rust 側の serde 属性と手で突き合わせる
- **TanStack Table**: データ処理(検索・ソート・ページング)が WASM / サーバ側で
  済んでいるテーブルは manual モード(`manualSorting` 等)で使い、
  クライアント側の再処理をさせない
- **TypeScript**: 新規コードは最初から `.tsx` / `.ts`。既存 `.js` は
  `allowJs + checkJs: false` で型検査対象外。typescript は `~5.9.3` に固定
  (typescript-eslint の peer が TypeScript 6.1 未満のため。`npm update` で
  7.x に上げると lint が壊れる)
- **StrictMode**: 未導入。effect の二重実行監査が済むまで見送り(Phase 7)

## 残フェーズ

依存の少ない順・状態境界が切りやすい順。番号は実施順の目安であり、
Phase 1〜2 は入れ替え可能。

### Phase 1: モーダル類

- 対象: `initCustomAugHelpModal`(equip-slots.js 内)、`#shareUrlModal` /
  `#importShareModal`(share-ui.js 内)
- 理由: 共有状態への依存が最小の葉ウィジェット
- スモーク: テスト「オーグメント選択…」+ 全テストの console エラー検査

### Phase 2: ステータス表示(equipStatusSection)

- 対象: `status-display.js`(583 行)+ `tabs.js` のサブタブ切替、
  index.html の `#equipStatusSection`(静的テーブル約 770 行)
- 理由: 入力が `deps` オブジェクトに集約された表示専用モジュールで、
  削除行数対リスク比が最大。status-display.js がサブタブの表示状態を
  直接触っているため、tabs.js のサブタブ側と**一体で**移行する
- スモーク: テスト「保存済みキャラクターのステータス…」(HP≠0、サブタブ切替)

### Phase 3: 装備エディタ単位

- 対象: `equip-slots.js` + `augments.js` の表示側 +
  `equip-sets.js#showEquipSetEditForm`
- 前提作業: `equip-state.js` の `equipState`(素の可変シングルトン)を
  pub/sub 化し、`useSyncExternalStore` ブリッジで React と旧コードの両方から
  読めるようにする。`data-slot` 属性による DOM 越しの参照
  (augments.js → equip-slots.js の DOM)があるため、この 3 つは分割しない
- スモーク: テスト「オーグメント選択…」「スロット検索で装備を選び…」

### Phase 4: 装備セットタブバー

- 対象: `equip-sets.js` 残部(タブバー、drag & drop 並べ替え、保存/コピー/削除)
- スモーク: テスト「保存済みキャラクター…」「スロット検索で…」

### Phase 5: キャラクター管理

- 対象: `character-list.js` + `character-form.js`
- 理由: form のモジュールレベル状態 5 つと list の `#jobLevelTable` 構築が
  相互依存しており、実質 1 ウィジェット。**一体で**移行する
- スモーク: テスト「キャラクターを UI から作成できる」

### Phase 6: index.html の畳み込み

- 対象: `main.js` / `tabs.js` 残部 / `share-ui.js` の `?share=` 分岐 /
  index.html のインライン `<style>`(約 1,070 行)の整理
- 理由: 全ウィジェットが React 化された後、ページを単一 root に統合する。
  共有閲覧モードは通常初期化をスキップする第 2 の起動経路なので、
  ここまで各フェーズでも `?share=` の動作確認を怠らないこと
- スモーク: テスト「トップページ…」「共有 URL…」+ 全体

### Phase 7: 純関数層の TS 化と仕上げ

- 対象: `utils.js` / `constants.js` / `equip-bonuses.js` / `storage.js` /
  `share.js` / `repositories/*` の TS 化、`web/js/wasm.js` の
  `web/src/wasm.ts` への統合と WASM 境界型の拡張
- 合わせて: Vitest 導入(equip-bonuses のユニットテスト —
  [docs/tech-debt/inline-script-monolith.md](../tech-debt/inline-script-monolith.md)
  の残課題)、`<StrictMode>` 導入の検討
