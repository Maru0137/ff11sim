# インライン script のモジュール化（未着手の負債）

## 現状

| ファイル | インライン script | 関数 | イベントリスナ |
|---|---|---|---|
| `web/index.html`（全 3,804 行） | **1,637 行**（`<script type="module">` 1 本） | 45 個 | 44 箇所 |
| `web/search.html`（全 784 行） | 383 行 | — | — |

`web/js/` には既にモジュール群（constants / storage / status-display / auth-ui など
計 1,381 行）があり、インライン側はそれらを import して使っている。つまり
「移せない理由」があるのではなく、**移されていないだけ**の状態。

## なぜ問題か（実害が出た）

装備ロジックの Rust 移植時、削除した `equip-stats.js` の `WEAPON_SKILL_KEYS` を
index.html 内のインライン script が参照したままになり、**本番でステータスが
全て 0 になった**（修正: `ee48e90`）。Rust のテスト 177 件は全て通っていた。

インラインである限り:

- ユニットテストから import できない（テスト不能）
- ESLint は `eslint-plugin-html` 経由でしか見られない
- 3,804 行の HTML は編集時の見通しが悪く、diff レビューも難しい

## 対策の方向性

`web/index.html` のインライン script を `web/js/` 配下の ES モジュールに分割する。
既存の `web/js/` の構成（機能単位のモジュール + `repositories/`）に合わせる。

分割の候補粒度（インライン script の内容から）:

- キャラクター編集フォーム（ジョブレベル表・メリポ・JP・スキル入力）
- 装備セット管理（タブ・スロット UI・オーグメント選択）
- 装備ステータス集計（`calculateEquipSetBonuses` / `addSkillBonuses` — 事故が起きた箇所）
- WASM 初期化とエクスポートの取り回し
- 共有閲覧モード（`?share=` 分岐）

一括の書き直しではなく、**1 モジュールずつ切り出して都度検証**すること。

## 進め方の前提（重要）

- **安全網は整備済み**。各段階で以下を回すこと:
  - `npm run lint` — no-undef。切り出し時の参照漏れを静的に検出する
  - `npm run test:smoke` — Playwright 3 本。ページが実際に動くことを確認する
    （`WEAPON_SKILL_KEYS` 事故を検出できることを注入試験で実証済み。
    ただし検出できるのは**装備入りセットの経路のみ**。tests/smoke.spec.js 参照）
  - `cargo test --manifest-path rust/Cargo.toml` — Rust 側は触らないが回帰確認に
- スモークテストが薄い領域（オーグメント選択・共有モード・ログイン）は、
  切り出す前にスモークを足す方が安全
- `search.html` の 383 行は同じ手法の 2 周目として後回しでよい
- HTML 側に `onclick` 属性は無い（全て `addEventListener`）。ID 参照
  （`getElementById` 多数）が結合点なので、モジュール化しても DOM 構造は不変

## 完了条件

- インライン script が「モジュールの import と初期化呼び出しだけ」になる
- 切り出した各モジュールが `npm run lint` の対象になっている
- スモークテスト 3 本（+追加分）が通る
- ブラウザでの手動確認（キャラ作成 → 装備セット → ステータス表示 → 検索）
