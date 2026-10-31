---
okf_version: "0.2"
---

# ステータス

* [命中 (Accuracy)](status/accuracy.md) - メインの命中と遠隔の命中（飛命）を、能力値・武器のスキル値・装備・命中ボーナスから求める式。
* [攻撃力 (Attack)](status/attack.md) - メイン・サブ・遠隔の攻撃力のうち、加算で求める部分（加算枠）の式。
* [攻撃力の乗算枠](status/attack_multiplier.md) - 攻撃力の加算枠に掛ける乗算枠の計算構造と、項目ごとの 256 分率の値。
* [基本ステータス (HP / MP / STR / DEX / VIT / AGI / INT / MND / CHR)](status/base_stats.md) - HP・MP と 7 つの能力値を、種族・ジョブのグレードとレベル・マスターレベル・メリットポイントから求める式。
* [防御力 (Defense)](status/defense.md) - 防御力を VIT・レベル・装備・防御ボーナスから求める式。
* [回避 (Evasion)](status/evasion.md) - 回避を AGI・回避スキル・装備・回避ボーナスから求める式。
* [魔命 (Magic Accuracy)](status/magic_accuracy.md) - 「魔命」が指す 2 つの値と、レジストの判定に使う実効の魔命の式。
* [魔法攻撃力 (Magic Attack)](status/magic_attack.md) - 魔法攻撃力を固定値 100・装備・魔攻ボーナスから求める式。
* [魔法防御力 (Magic Defense)](status/magic_defense.md) - 魔法防御力を固定値 100・装備・ジョブ特性・ギフト・ジョブポイントから求める式。
* [メリットポイントの割り振りの上限](status/merit_points.md) - メリットポイントを割り振れる量の、項目ごととグループの合計の上限。
* [スキル上限と有効スキル値](status/skill_cap.md) - スキル上限と、それを使って有効スキル値を求める式。
* [得TP (TP per hit)](status/tp.md) - 1 ヒットで得られる TP を、武器の隔とストアTP から求める式。

# ジョブ特性

* [スマイト (Smite)](job_traits/smite.md) - ジョブ特性「スマイト」の習得条件、256 分率の効果値、攻撃力への効き方。

# アイテム

* [日本語説明文の条件ラベル](items/description_labels.md) - アイテムの日本語の説明文に出る条件ラベルの意味・適用範囲・件数。
* [装備スロットの名称](items/equipment_slots.md) - 16 個の装備スロットの名称と、レンジウェポンと矢弾に着けられるものの関係。
