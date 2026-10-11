---
status: accepted
date: 2026-08-15
decision-makers: Akira Maruoka
---

# 0022. キャラクター一覧の並び順を配列順として永続化し、ドラッグで入れ替えられるようにする

## Context and Problem Statement

キャラクターを編集して保存し直すと、一覧の並び順が入れ替わっていた。
原因は `SupabaseCharacterRepo.list()` が `ORDER BY` 無しで `SELECT` していたことで、
ログイン中は**そもそも並び順が保存されていなかった**。Postgres は行の物理順で返し、
`UPDATE`（upsert）された行はヒープの末尾へ移動しがちなので、保存のたびに順番が変わる。
ゲスト（localStorage）は JSON 配列をそのまま読み書きするので順序が保たれており、
ログイン状態によって挙動が違っていた。

あわせて「一覧をドラッグで並び替えたい」という要求がある。並び順を保存できるように
することが前提なので、同じ変更で扱う。決めるべきは (a) 並び順をどこに持つか、
(b) 既存行の初期値をどう決めるか、(c) ドラッグの掴み方、(d) 並び替えを未保存ガードの
対象にするか。

## Decision Drivers

* ログイン / ゲストで並び順の挙動が変わらないこと
* 保存や編集といった別の操作の副作用で並び順が変わらないこと
* 装備セットのタブ並び替えと同じ規則・同じ実装パターンであること（[ADR 0008](0008-equipset-sharing.md) 以前からある既存 UI）
* UI 層が扱うレコードの形を変えないこと（`CharacterRecord` に表示都合の項目を混ぜない）
* migration は Supabase ダッシュボードでの手動適用（既存 migration と同じ運用）

## Considered Options

1. 並び順の持ち方
    - 1.1. `data` jsonb の中に順序フィールドを持たせる
    - 1.2. `position` カラムを追加し、配列インデックスを保存する
    - 1.3. 表示時にクライアントで名前順に並べ替える（並び順は保存しない）
2. 既存行の `position` の初期値
    - 2.1. 既定値 0 のままにして、次回保存時に確定させる
    - 2.2. migration で `created_at` 順に振り直す
3. ドラッグの掴み方
    - 3.1. 行全体を `draggable` にする
    - 3.2. 専用のドラッグハンドルだけを `draggable` にする
4. 並び替えと未保存ガード（[ADR 0020](0020-unsaved-changes-guard.md)）
    - 4.1. 並び替えも `guard()` を通す
    - 4.2. 対象外にする

## Decision Outcome

選択: **1.2 + 2.2 + 3.2 + 4.2**。

- **配列の順序が表示順であり保存順**、という規則にする。localStorage は JSON 配列なので
  そのまま、Supabase は `characters.position` に配列インデックスを書き、
  `list()` で `.order('position')` して復元する。
  `position` は行の列であって `data` jsonb には入れないため、
  `list()` が返すレコードの形は変わらない（UI 層は今までどおり配列順だけを見る）。
  `save()` は元から全件 upsert なので、インデックスの書き込みに追加のコストは無い。
- **migration** `supabase/migrations/004_characters_position.sql` で
  `position int not null default 0` を追加し、既存行を `user_id` 内で振り直す。
  `characters` テーブルはリポジトリ管理外で作成されており列構成を確定できないため、
  行の指定には常に存在するシステム列 `ctid` を使い、`created_at` は
  `information_schema` で有無を確認してから使う（無ければ名前順）。
- **ドラッグは `⠿` ハンドルだけ**を `draggable` にする。行には「編集」「削除」ボタンが
  あり、行全体を掴めるようにすると押しにくくなる。
  掴んだ要素を落とし先の位置へ挿入する規則で、`EquipSetControls` /
  `PropsetManageModal` と同じ HTML5 Drag and Drop を使う。
  配列操作は `web/src/character/reorder.ts` の `moveItem()` に切り出す。
- **並び替えは未保存ガードの対象外**。並び替えは一覧（保存済みデータ）に対する操作で、
  開いている編集フォームには触れないため、失われる編集が無い。
- 並び替えは確定操作として即座に永続化する（ドロップで `saveCharacters` →
  `reloadCharacterList`）。装備セットのタブ並び替えと同じ挙動。

### Consequences

* Good: 編集して保存し直しても並び順が変わらなくなる（本来のバグ修正）。
* Good: ログイン / ゲストで並び順の挙動が一致する。
* Good: `CharacterRecord` の形が変わらないので、未保存判定（ADR 0020）や
  共有・同期の比較対象に影響しない。
* Bad: **migration の適用が先に必要**。`position` カラムが無い状態で新しいコードが動くと
  `.order('position')` が失敗し、ログイン中のキャラクター一覧が空になる。
  main へのマージは `deploy.yml` で本番反映されるため、マージ前に SQL を適用すること。
* Bad: 並び替えのたびに全件 upsert が走る（既存の保存と同じ経路なので新しい負荷ではないが、
  ドロップ 1 回ごとにラウンドトリップが 2 回発生する: `list()` + `upsert`）。
* Neutral: 既存ユーザーの初期並び順は `created_at` 順になる。
  これまで不定だったので、何順であっても変化として現れる。

### Confirmation

* `web/src/character/reorder.test.ts`: `moveItem()` が前後どちらへ動かしても
  落とし先の位置へ挿入されること、同じ位置・範囲外の添字なら元の配列を
  そのまま返すこと（呼び出し側が保存を省ける）、元の配列を破壊しないことを検証する（8 件）。
* `tests/smoke.spec.js`「キャラクター一覧をドラッグで並び替えられ、編集して保存しても順番が変わらない」:
  ハンドルのドラッグで並びが変わること、リロード後も保たれること、
  キャラクターを編集して保存しても位置が動かないことをブラウザ上で確認する。
  Supabase 経路は CI から叩けないため、ここで見るのはゲスト（localStorage）経路。
* `npm run typecheck`: `moveItem()` が `readonly T[]` を返すことで、
  呼び出し側が結果を保存に渡すときの型を確認する。

## Pros and Cons of the Options

### 1.1. `data` jsonb に順序フィールドを持たせる

* Good: migration が要らない
* Bad: `list()` が返すレコードに表示都合のフィールドが混ざり、
  未保存判定や共有データの比較対象に入ってしまう
* Bad: 編集フォームは保存レコードを組み直すため、順序フィールドを明示的に
  引き継ぐ必要があり、忘れると並び順が飛ぶ

### 1.2. `position` カラムを追加する（採用）

* Good: 装備セット（migration 001）と同じ形になり、規則が 1 つで済む
* Good: 並び順が行の属性として分離され、UI のレコード形に影響しない
* Good: `ORDER BY` が DB 側で効くので、クライアントの並べ替えが要らない
* Bad: migration の適用が要る（適用前にコードが出ると一覧が壊れる）

### 1.3. クライアントで名前順に並べる

* Good: migration もカラム追加も要らず、順序が常に決定的になる
* Bad: ドラッグ並び替えができない（今回の要求を満たさない）
* Bad: 登録順で並べたいという通常の期待に反する

### 2.1. 既定値 0 のままにする

* Good: migration が `alter table` の 1 文で済む
* Bad: 全行が 0 のままだと `ORDER BY position` の中で順序が不定になり、
  次に保存するまで問題が直らない

### 2.2. migration で振り直す（採用）

* Good: 適用した時点で並び順が確定する
* Bad: `created_at` の有無を確認する分、migration が長くなる

### 3.1. 行全体を `draggable` にする

* Good: 掴める範囲が広い
* Bad: 行の中の「編集」「削除」ボタンがドラッグ開始と競合して押しにくくなる
* Bad: テキスト選択ができなくなる

### 3.2. ハンドルだけを `draggable` にする（採用）

* Good: ボタン操作と衝突しない
* Good: `PropsetManageModal` と同じ見た目・同じ操作
* Bad: 掴める範囲が狭く、ハンドルの存在に気づく必要がある

### 4.1. 並び替えもガードする

* Good: すべての永続化操作が同じ経路を通る
* Bad: 失われる編集が無いのに確認ダイアログが出る。ADR 0020 の
  「確認を出すのは実際にデータが失われる操作だけ」という方針に反する

### 4.2. ガード対象外（採用）

* Good: ADR 0020 の方針と一貫する
* Neutral: 並び替えと編集フォームは独立しており、同時に行っても互いに影響しない

## More Information

* [ADR 0005](0005-localstorage-default-supabase-optin.md) — 保存先の切り替え
  （ゲストと Supabase で挙動を揃える必要の根拠）
* [ADR 0020](0020-unsaved-changes-guard.md) — 未保存ガードの適用範囲
* [ADR 0021](0021-character-edit-density-and-sections.md) — キャラクター編集フォームの再編
* `supabase/migrations/001_equipsets_position.sql` — 装備セットの並び順を
  `position` カラムで持つ先行例
