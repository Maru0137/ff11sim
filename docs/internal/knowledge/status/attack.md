# 攻撃力 (Attack) 計算仕様

参考: [wiki.ffo.jp/html/1766.html](https://wiki.ffo.jp/html/1766.html)

メイン武器・サブ武器・遠隔武器の 3 種を別々に算出する。

## メイン攻撃力

```
メイン攻撃 = STR項 + 武器スキル + 8 + 装備攻撃 + attack_bonus
```

### STR項

| 武器種 | STR項 |
|---|---|
| 格闘 (HandToHand) | int(STR × 0.75) |
| 片手・両手武器 | STR |

### 武器スキル

メイン武器のスキル種別に応じた**有効スキル値**:

```
有効スキル = base + メインスロットボーナス + 全スロット共通ボーナス
```

| 項 | 内容 |
|---|---|
| `base` | min(キャラスキル値, ジョブキャップ + メリットボーナス) |
| `ジョブキャップ` | `job_skill_cap(main_job, skill, main_lv, master_lv)`、ML は +1/Lv |
| `メリットボーナス` | スキル別メリット rank × 2（rank 0-8、最大 +16） |
| `メインスロットボーナス` | メインスロット装備の同種武器スキル+ |
| `全スロット共通ボーナス` | 非武器スロットおよび武器スロット装備の非武器スキル+ |

### attack_bonus

```
attack_bonus = ジョブ特性 AttackBonus(main/sup 最大)
             + ギフト physical_attack
             + JPカテゴリ physical_attack
```

#### ジョブ特性 AttackBonus

ランクごとの累積値:

| Rank | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| 値 | 10 | 22 | 35 | 48 | 60 | 72 | 84 | 96 |

取得ジョブ・レベル:

- **DRK**: Lv10, 30, 50, 70, 76, 83, 91, 99（最大 rank8 = +96）
- **WAR**: Lv30, 65, 91（最大 rank3 = +35）
- **DRG**: Lv10, 91（最大 rank2 = +22）

#### ギフト physical_attack

メインジョブの累計 JP 量で解放されるティアの累積。
War 全解放（2100JP）= +70（[10, 15, 20, 25] の累積）。

各ジョブ値は `rust/src/job_points.rs:JOB_GIFTS` を参照。

#### JP カテゴリ physical_attack

特定ジョブの一部カテゴリが直接 physical_attack に寄与する場合のみ加算。
War のカテゴリは physical_attack に直接寄与しない（0）。

## サブ攻撃力

```
サブ攻撃 = STR項 + サブ武器スキル + 8 + 装備攻撃 + attack_bonus
```

### サブ STR項

| 武器種 | STR項 |
|---|---|
| 格闘 (HandToHand) | int(STR × 0.75) |
| 片手武器 | int(STR × 0.5) |

> ※ サブ格闘もメイン格闘と同じ 0.75 倍。両手武器はサブ装備不可のため対象外。

### サブ武器スキル

```
有効スキル = base + サブスロットボーナス + 全スロット共通ボーナス
```

`attack_bonus` はメインと同じ（ジョブ特性・ギフト・JPカテゴリ）。

> ⚠ 現実装 `calc_sub_attack` は武器種にかかわらず常に `int(STR × 0.5)` を使用。サブ格闘の 0.75 倍は未対応。

## 遠隔攻撃 (飛攻)

```
飛攻 = STR + 遠隔武器スキル + 8 + 装備飛攻 + attack_bonus
```

遠隔武器スキル:
```
有効スキル = base + レンジスロットボーナス + 全スロット共通ボーナス
```

> ※ メイン/サブ/レンジで `attack_bonus` の構成は同一。違いは STR 項の係数と装備分のみ。

## 計算例: Hum War99/Sam59 ML50, ラフリア装備セット

### 入力

- STR (装備込) = 161 + 247 = 408
- 武器スキル GreatAxe 有効値 = 490 + 277 (ラフリア main) + 21 (BIロリカ global) = 788
- 装備攻撃 = 448
- attack_bonus = 35 (War rank3 trait) + 70 (gift 2100JP) + 0 (JPカテゴリ) = 105

### 計算

```
メイン攻撃 = STR + skill + 8 + equip_atk + attack_bonus
           = 408 + 788 + 8 + 448 + 105
           = 1757
```

## 実装

| 場所 | 役割 |
|---|---|
| `rust/src/status.rs:calc_main_attack` | メイン攻撃の基本式 |
| `rust/src/status.rs:calc_sub_attack` | サブ攻撃の基本式 |
| `rust/src/status.rs:calc_ranged_attack` | 飛攻の基本式 |
| `rust/src/skills.rs:effective_skill` | 武器スキルキャップ計算（メリット込） |
| `rust/src/wasm.rs:chara_to_status_result` | スキルボーナス集約と attack_bonus 加算 |

## 未対応項目

- Attack+X% 系装備（パーセント補正）
- バーサク・ディフェンダー等のジョブアビリティ
- 食事
- 弱点属性に対する攻撃力ブースト
- フェンサー特性による攻撃ボーナス（一部装備で取得）
- サブ格闘の STR項 0.75 倍（実装は常に 0.5）
