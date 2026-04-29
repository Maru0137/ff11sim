# 命中 (Accuracy) 計算仕様

参考: [wiki.ffo.jp/html/223.html](https://wiki.ffo.jp/html/223.html)

メイン命中・遠隔命中 (飛命) を別々に算出する。

## メイン命中

```
メイン命中 = int(DEX × 0.75) + accuracy_skill_term(skill) + 装備命中 + accuracy_bonus
```

## 遠隔命中 (飛命)

```
飛命 = int(AGI × 0.5) + accuracy_skill_term(skill) + 装備飛命 + accuracy_bonus
```

> ※ メインは DEX × 0.75、遠隔は AGI × 0.5。係数が異なる。

## 武器スキル区分値 `accuracy_skill_term`

スキル値による区分的線形補間:

| スキル範囲 | 寄与 |
|---|---|
| ≤ 0 | 0 |
| 1-200 | スキル値そのまま |
| 201-400 | 200 + int((skill - 200) × 0.9) |
| 401-600 | 380 + int((skill - 400) × 0.8) |
| 601- | 540 + int((skill - 600) × 0.9) |

具体値:

| skill | term |
|---|---|
| 100 | 100 |
| 200 | 200 |
| 300 | 290 |
| 400 | 380 |
| 500 | 460 |
| 600 | 540 |
| 700 | 630 |
| 788 | 709 |
| 800 | 720 |

## accuracy_bonus

```
accuracy_bonus = ジョブ特性 AccuracyBonus(main/sup 最大)
               + ギフト physical_accuracy
               + JPカテゴリ physical_accuracy
```

### ジョブ特性 AccuracyBonus

ランクごとの累積値:

| Rank | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| 値 | 10 | 22 | 35 | 48 | 60 | 72 |

取得ジョブ・レベル:

- **RNG**: Lv10, 30, 50, 70, 86, 96（最大 rank6 = +72）
- **DRG**: Lv30, 60, 76（最大 rank3 = +35）
- **DNC**: Lv30, 60, 76（最大 rank3 = +35）
- **RUN**: Lv50, 70, 90（最大 rank3 = +35）

### ギフト physical_accuracy

War 全解放（2100JP）= [5, 8, 10, 13] の累積 = **+36**

### JP カテゴリ physical_accuracy

特定ジョブの一部カテゴリが直接 physical_accuracy に寄与する場合のみ加算。
War のカテゴリは physical_accuracy に直接寄与しない（0）。

## 計算例: Hum War99/Sam59 ML50, ラフリア装備セット

### 入力

- DEX (装備込) = 156 + 179 = 335
- 武器スキル GreatAxe 有効値 = 788
- accuracy_skill_term(788) = 540 + floor((788-600) × 0.9) = 540 + floor(169.2) = 540 + 169 = 709
- 装備命中 = 448
- accuracy_bonus = 0 (trait) + 36 (gift) + 0 (JPcat) = 36

### 計算

```
メイン命中 = int(DEX × 0.75) + accuracy_skill_term(skill) + equip_acc + accuracy_bonus
           = int(335 × 0.75=251.25) + 709 + 448 + 36
           = 251 + 709 + 448 + 36
           = 1444
```

## 実装

| 場所 | 役割 |
|---|---|
| `rust/src/status.rs:calc_accuracy` | メイン命中の基本式 |
| `rust/src/status.rs:calc_ranged_accuracy` | 飛命の基本式 |
| `rust/src/status.rs:accuracy_skill_term` | スキル区分値 |
| `rust/src/wasm.rs:chara_to_status_result` | accuracy_bonus 加算と最終出力 |

## 未対応項目

- 装備の "Weapon Skill Accuracy+X" は通常命中扱いしない（WS 専用、メイン命中に含めない）
  - 実装: `web/js/equip-stats.js` の Accuracy regex に `(?<![Ss]kill )` lookbehind あり
- 食事による命中ブースト
- ジョブアビリティ系（フォーカス・ダンスサンバ等）の一時補正
- 連携状態・ウェポンスキル中の補正
