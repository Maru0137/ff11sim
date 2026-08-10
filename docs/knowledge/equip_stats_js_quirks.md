# 装備ステータス抽出 (JS 実装) の既知の癖・バグ

`web/js/equip-stats.js` を Rust へ移植する過程 (docs/adr/0010) で見つかった、
JS 実装の誤った挙動をまとめる。

移植は「挙動を変えない」方針 (docs/adr/0010 の案 3) で進めているため、
**ここに挙げた挙動は Rust 側でもそのまま再現している**。既存テストを移植の
検証に使うには挙動を固定する必要があるため。修正は移植完了後に別途行う。

各項目には Rust 側の再現テスト名を記す。修正する際はそのテストを書き換える。

---

## 1. `Pet:` セグメント除去が後続のラベルを食う

**症状**: `Pet:` / `Avatar:` / `Wyvern:` / `Automaton:` の除去に使う
`/(?:Pet|Avatar|Wyvern|Automaton):[^:]*/g` は、`[^:]*` が次のコロン直前まで
消費する。そのため直後に来る `DEF:` や `DMG:` の **ラベル部分ごと消える**。

```
入力: "HP+100\nPet: Accuracy+15\nDEF:10"
出力: "HP+100\n:10"          ← DEF ラベルが消え、防御力が取得できない
```

**影響**: 実データで 2 件。

| id | 名前 | 失われる値 |
|---|---|---|
| 17961 | Lion Tamer | DEF |
| 21433 | Neo Animator | DMG |

**再現テスト**: `equip_stats::tests::normalize_pet_strip_eats_following_label`

**修正案**: 除去範囲を行単位、または「次の既知ラベル直前まで」に限定する。
ただし Pet 行は折り返しで複数行に分かれるため、単純な行単位では
`web/test/equip-stats-extraction.test.js` のアスプロピアス等のケースが壊れる。
条件セグメント対応 (下記 4) とまとめて設計するのが妥当。

---

## 2. `All BP -30 to +30` が最小値として扱われる

**症状**: `ALL\s*BP\s*([+-])\s*(\d+)` は最初の符号付き値だけを拾う。
範囲表記の上限は「All BP」が直前に無いためマッチしない。

```
入力: "Mastery Rank: All BP -30 to +30"
出力: str〜chr がすべて -30      ← 本来はマスタリーランクに応じて -30〜+30
```

Unity Ranking (`Unity Ranking: Attack+10～15`) には最大値を採用する専用処理が
あるのに対し、こちらには無い。

**影響**: 実データで 1 件 (id 26120 Hoxne Earring / ホクスニピアス)。

**再現テスト**: `equip_stats::tests::all_bp_range_notation_takes_first_match`

**修正案**: 範囲表記 (`A to B`) を Unity Ranking と同様に扱う。ただし
「どちらを採用するか」は条件 (マスタリーランク) 次第なので、条件セグメント
対応の一部として扱うのが妥当。

---

## 3. 条件付きセグメントが体系的に扱われていない

**症状**: 説明文には条件付きの効果が多数含まれるが、JS はこれを個別対応の
寄せ集めでしか扱っていない。

| 条件 | 現在の扱い |
|---|---|
| `Pet:` / `Avatar:` / `Wyvern:` / `Automaton:` | 除去する |
| `Latent effect:` | **WS ダメージのときだけ**除外する |
| `Unity Ranking:` | 最大値を取り出して**無条件に加算** |
| `In Dynamis:` / `Right ear:` / `Set:` / `Nighttime:` ほか 25 種以上 | **何もしない** |

そのため以下が起きる。

- `In Dynamis: DEF:22` が常時有効として扱われうる
  (ただし `DEF:` は先頭のみ採用する実装なので、実際には偶然拾われない)
- `Unity Ranking:` の効果がユニティ加入と無関係に常時加算される

**実データの条件プレフィックス出現数** (行頭のもの、上位):

```
1295  Set:                     543  Additional effect:
 393  Enchantment:             343  Aftermath:
 337  Latent effect:           253  Pet:
 240  Avatar:                  215  Unity Ranking:
 153  Automaton:               123  Wyvern:
  84  In areas under own nation's control:
  68  Right ear:                56  In areas outside own nation's control:
  48  In Dynamis:               42  Besieged:
  35  Campaign:                 31  Assault:
  23  Nighttime:                21  Daytime:
```

**修正案**: 条件を型で表現し、`Stats { unconditional, conditional: Vec<(Condition, Stats)> }`
の形で保持して、有効化する条件を呼び出し側が選べるようにする。
Unity ランクや In Dynamis を UI でトグルできるようになる。
`BonusStats` と WASM 境界の API に影響するため、別 ADR を立てて扱う。

---

## 4. `DEF:` / `DMG:` / `Delay:` は先頭の 1 件しか採用されない

**症状**: `matchColon` は非グローバル。同じラベルが複数回現れても先頭だけを見る。

```
Bulwark Shield (15067): "DEF:1\nIn Dynamis: DEF:22"  → def = 1
```

この結果は「条件を理解した」ものではなく、非グローバルだった偶然による。
条件が逆順に書かれていれば条件付きの値を拾ってしまう。

**再現テスト**: 戦闘系スライスで追加予定

**修正案**: 上記 3 と同じ。

---

## 5. 【意図的な非互換】名前ソートの並び順が JS と異なる

これは JS のバグではなく、**Rust 移植で意図的に受け入れた差異**である。

**内容**: JS の `search()` は文字列ソートに `String.prototype.localeCompare` を使う。
Rust 側は `String::cmp` (Unicode コードポイント順) で実装した。

`localeCompare` は全角英字を半角相当として扱うため `ＡＢキュイラス` は `A` として
並ぶが、コードポイント順では全角英字 (U+FF21〜) が漢字 (U+4E00〜) より後ろに来る。
そのため**全角英字で始まる装備群が一塊で末尾側へ移動する**。

**影響**: 装備検索ページ (`web/search.html`) のソート選択肢「名前」のみ。
`sortBy: 'ja'` が対象で、他の選択肢 (Lv / iLv / 説明文ステータス) は数値なので影響なし。
「カテゴリ」は値が `Weapon` / `Armor` の ASCII 2 種のみで一致する。
`web/index.html` の装備セット編集ドロップダウンは `sortBy: 'id'` 固定なので影響しない。

**実測** (`slot: 'body'`, `sortBy: 'ja'`, 昇順, 全 1,603 件):

```
最初に食い違う位置: 105 件目
位置が変わった件数: 1436/1603 (89.6%)

  位置   JS (localeCompare)   | Rust (コードポイント順)
   105  ＡＢキュイラス+2          | テチアンサイオ+2
   106  ＡＤオングルリヌ+2         | ユクシンコート+2
```

**判断**: 影響が装備検索の 1 ソート選択肢に限られること、厳密な五十音順は読み仮名
データがない以上どのみち実現できないことから、コードポイント順で妥協した。

**修正したくなった場合の案**:

1. 比較キーに `item_search::normalize_for_search()` を通す (全角英数を半角に畳む)。
   追加依存もサイズ増もなく、主因である全角英字ブロックの移動は解消する。
   漢字同士の順序差は残る。
2. 照合ライブラリ (ICU 相当) を導入する。`localeCompare` に近づくが WASM サイズが増える。
