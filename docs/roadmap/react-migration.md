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
- **StrictMode**: Phase 7 で導入済み (開発時のみの検査。本番ビルドは no-op)

## 各フェーズの実施状況(全フェーズ実施済み: 2026-08-11)

依存の少ない順・状態境界が切りやすい順に、フェーズごとに 1 コミットで実施した。
完了条件 (lint / typecheck / スモーク 9 本、セレクタ変更なし) は各フェーズで確認済み。

1. **モーダル類** — 済。`web/src/modals/`。開閉 API は modal-store に置き、
   レガシー側からも呼べるようにした
2. **ステータス表示** — 済。`web/src/status/`。status-display.js の約 200 の
   setText を「id → 表示値」レコードの組み立て (compute.ts) に変換し、
   テーブル構造は index.html から機械変換 (StatusTables.tsx)。
   サブタブ状態と魔法タブの表示可否も React 化
3. **装備エディタ** — 済。`web/src/equip/EquipSlots.tsx` + equip-store.ts。
   equipState はバージョン購読で React と接続し、data-slot セレクタ経由の
   DOM 参照 (DOM-as-database) を解消
4. **装備セット管理パネル** — 済。`web/src/equip/EquipSetControls.tsx` /
   EquipSetToolbar.tsx / equip-sets-store.ts
5. **キャラクター管理** — 済。`web/src/character/`。モジュール変数 5 つ +
   DOM に分散していた編集中状態を FormState に集約
6. **index.html の畳み込み** — 済。ページ全体が App.tsx の単一 root。
   main.js / tabs.js を削除し、共有閲覧モードの DOM 操作も store 化。
   インライン CSS は web/styles/index.css へ抽出
7. **仕上げ** — 一部済:
   - TS 化済み: utils / share / augments / equip-state (equip-store へ統合)
   - Vitest 導入済み (`npm run test:unit`、CI にステップ追加)。
     format / utils / store-utils の純関数テスト
   - StrictMode 導入済み (両ページ)

## 残課題

- **レガシー JS 層の TS 化**: `web/js/` に残るのは constants.js /
  storage.js / repositories / supabase-client.js / sync.js / share-ui.js /
  equip-bonuses.js / wasm.js / config.js。いずれも DOM 非依存で、
  変換は機械的。wasm.js を web/src/wasm.ts に統合する場合は、CI の
  typecheck が wasm-pack より先に走る前提 (web/pkg 不在) を崩さない工夫が要る
- **equip-bonuses のユニットテスト**: WASM 関数 (extract_all_stats 等) に
  依存するため、node での WASM 初期化 (バイト列を渡す init) か
  ブラウザモードの導入が必要。constants.js の top-level await fetch も
  node では動かないため、モックか読み込み方法の変更が要る
  ([docs/tech-debt/inline-script-monolith.md](../tech-debt/inline-script-monolith.md) の残課題)
- **search.html のインライン CSS 抽出** (index.html は抽出済み)
- **サポートジョブ選択肢からメインジョブを除外する条件の修正**: 旧実装から
  大文字小文字の不一致で機能しておらず、挙動維持のまま移植した
  (web/src/equip/EquipSetControls.tsx のコメント参照)。直すなら独立した修正として
