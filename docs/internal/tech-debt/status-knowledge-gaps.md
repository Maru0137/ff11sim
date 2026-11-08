# ステータスのナレッジと実装の食い違い

ステータスの計算について、ナレッジ（`docs/internal/knowledge/`）に書いたゲームの振る舞いと ff11sim の実装が食い違っている点と、ゲームにはあるが ff11sim が計算していない項目を記録する。
2026-10-31 にナレッジの書き方を [docs/internal/knowledge-format/README.md](../knowledge-format/README.md) に揃えたとき、各ナレッジの「実装」「未対応項目」の節と注記からここへ移した。直した項目はこの文書から消す。

## 実装がナレッジと食い違っている点

| 項目 | ナレッジ | 実装 |
|---|---|---|
| スキルのメリットポイント | スキル上限を広げず、スキル上限で抑えた後に足す。スキル上限が 0 のスキルにも足す（[skill.md](../knowledge/status/skill.md)。仕様書 0007 の AC22、2026-10-18 に決定） | `rust/src/skills.rs` の `effective_skill_parts` は、スキル上限をランク 1 につき 2 広げ、スキル上限が 0 のスキルには足さない |
| スキル値の入力上限 | メリットポイントを足さない（仕様書 0002 の「スキル値の入力上限」） | `rust/src/skills.rs` の `default_skill_value` はメリットポイントの分を足す |
| 全スロット共通のスキルボーナス | スロット専用でないすべてのスキル+。アイテムレベル装備でない武器のスキル+ も含む（[physical_attack.md](../knowledge/status/physical_attack.md)。仕様書 0007 の AC26、2026-10-25 に決定） | `rust/src/equip.rs:203-212` は、武器スロットの装備が持つ武器のスキル+ をすべてそのスロット専用にしている |
| スキルキャップ | ランクとレベル帯ごとの式で求める。レベル 1〜60 は A+ と A、B+〜B-、C+〜C- がそれぞれ共通の式（[skill.md](../knowledge/status/skill.md) の「スキルキャップ」） | `data/skill_cap_control_points.json` はレベル 1・50 の値を読み違えている（例: ランク C のレベル 50 は記事の式では 142、データは 101）。`rust/src/skills.rs` の `skill_cap` はレベル 1・50・75・99 の 4 点の間を直線で補間するので、4 点以外のレベルでは記事の式と値がずれる。メインジョブ 99 のスキルキャップは一致するが、サポートジョブのスキル上限と、仕様書 0002 のスキル値の入力上限（レベル 99 未満のジョブ）は誤った値になる。GitHub issue #100 |
| サブの格闘の STR の項 | int(STR × 0.75)（[melee_attack.md](../knowledge/status/melee_attack.md)） | `rust/src/status.rs` の `calc_sub_attack` は武器の種類によらず int(STR × 0.5) |
| スマイトの効果値 | 256 分率の [25, 38, 51, 64, 76]（[job_traits/smite.md](../knowledge/job_traits/smite.md)） | `rust/src/job.rs` の `SMITE` は 100 分率の [10, 15, 20, 25, 30] を持つ |
| 攻撃力の乗算枠と計算後加算枠 | `int(加算枠 × (256 + Σ乗算枠) ÷ 256) + 計算後加算枠`（[physical_attack.md](../knowledge/status/physical_attack.md)） | `rust/src/status.rs` の攻撃力は加算枠だけで求める。スマイトも、格闘武器・両手武器を装備しているかの判定も入っていない |
| ギフトを解放する累計ジョブポイント（枠の位置 0） | FF11用語辞典の戦士のギフトのページは、物理防御力アップの 3・4 段階目を 550 / 1200 とする（[defense.md](../knowledge/status/defense.md) の「未確認の事項」） | `rust/src/gift.rs:331-332` は 605 / 1280 とする。黒魔道士のギフトのページの魔法防御力アップ（同じ枠）は 605 / 1280 で、コードと一致する。戦士のページの誤記の可能性があるが、確かめていない |

乗算枠を入れるときは、256 分率の値を項目ごとに表で持ち、100 分率の表記から換算しない。同じ 100 分率の表記でも 256 分率の値が違う項目があり、丸め規則も項目ごとに違うためである（[physical_attack.md](../knowledge/status/physical_attack.md) の「表記揺れ」「丸め規則」）。加算枠を求めた後に乗算枠を掛け、食事と応援の攻撃力はその後に足す。

## ゲームにはあるが計算していない項目

食事・強化魔法・ジョブアビリティによる一時的な効果、ダメージの計算は、仕様書 [0007 ステータスの計算](../specs/0007-status-calculation.md) のスコープ外なのでここには挙げない。

| 項目 | 関係するナレッジ |
|---|---|
| 攻撃力+X%・防+X%・HP+X%・MP+X% のような、装備による割合の補正 | [physical_attack.md](../knowledge/status/physical_attack.md)、[defense.md](../knowledge/status/defense.md)、[base_stats.md](../knowledge/status/base_stats.md) |
| 弱点の属性に対する攻撃力の上昇 | [physical_attack.md](../knowledge/status/physical_attack.md) |
| 一部の装備で得るフェンサーによる攻撃力のボーナス | [physical_attack.md](../knowledge/status/physical_attack.md) |
| コルセアのジョブポイント「適正距離の遠隔攻撃力アップ」（カテゴリ idx 9。条件付きの効果） | [ranged_accuracy.md](../knowledge/status/ranged_accuracy.md) |
| 連携の状態とウェポンスキル中の命中の補正 | [melee_accuracy.md](../knowledge/status/melee_accuracy.md) |
| 暗闇などのステータス異常による命中・回避の増減 | [physical_evasion.md](../knowledge/status/physical_evasion.md) |
| 薬品・スピリットリンクによる一時的な補正 | [defense.md](../knowledge/status/defense.md) |
| インヒビットTP（敵の与TPダウン） | [tp.md](../knowledge/game_system/tp.md) |

## 方針

食い違いの表の項目は、ナレッジに合わせて実装を直す。計算していない項目は、計算に入れるかを仕様書 0007 の改訂で決め、入れないと決めたものは仕様書のスコープ外に移してこの文書から消す。
