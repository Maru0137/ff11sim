# 命中 (Accuracy) 計算仕様

参考:
- メイン命中: [wiki.ffo.jp/html/223.html](https://wiki.ffo.jp/html/223.html)
- 飛命: [wiki.ffo.jp/html/2395.html](https://wiki.ffo.jp/html/2395.html)

メイン命中・遠隔命中 (飛命) を別々に算出する。係数とスキル区分が異なる。

## メイン命中

```
メイン命中 = int(DEX × 0.75) + accuracy_skill_term(skill) + 装備命中 + accuracy_bonus
```

## 遠隔命中 (飛命)

```
飛命 = int(AGI × 0.75) + ranged_accuracy_skill_term(skill) + 装備飛命
       + accuracy_bonus + ranged_accuracy_extra
```

> ※ メイン・遠隔とも能力値係数は **0.75**（DEX/AGI）。
> ※ スキル区分は近接 (`accuracy_skill_term`) と遠隔 (`ranged_accuracy_skill_term`) で別関数。

## 武器スキル区分値

### 近接 `accuracy_skill_term`

| スキル範囲 | 寄与 |
|---|---|
| ≤ 0 | 0 |
| 1-200 | スキル値そのまま |
| 201-400 | 200 + int((skill - 200) × 0.9) |
| 401-600 | 380 + int((skill - 400) × 0.8) |
| 601- | 540 + int((skill - 600) × 0.9) |

### 遠隔 `ranged_accuracy_skill_term`

近接と異なり、200 でしか曲折しない 2 段階のシンプルな曲線。

| スキル範囲 | 寄与 |
|---|---|
| ≤ 0 | 0 |
| 1-200 | スキル値そのまま |
| 201- | 200 + int((skill - 200) × 0.9) |

具体値（近接 vs 遠隔）:

| skill | 近接 | 遠隔 |
|---|---|---|
| 100 | 100 | 100 |
| 200 | 200 | 200 |
| 300 | 290 | 290 |
| 400 | 380 | 380 |
| 500 | 460 | 470 |
| 600 | 540 | 560 |
| 700 | 630 | 650 |
| 733 | 659 | 679 |
| 800 | 720 | 740 |

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

## 飛命専用ボーナス `ranged_accuracy_extra`

飛命のみに加算され、メイン命中には影響しない。

```
ranged_accuracy_extra = ギフト ranged_accuracy + JPカテゴリ ranged_accuracy
```

### COR JP カテゴリ「遠隔命中アップ」

- カテゴリ idx 7: ランクごとに **+1 飛命**
- 最大ランク 20 で +20

参考: [wiki.ffo.jp/html/31272.html](https://wiki.ffo.jp/html/31272.html)

## 計算例: Hum War99/Sam59 ML50, ラフリア装備セット (メイン命中)

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

## 計算例: Hum COR99/Sam59 ML50, 遠隔WS 装備セット (飛命)

### 入力

- AGI (装備込) = 378
- 武器スキル Marksmanship 有効値 = 733 (cap 448 + メリット 16 + 装備 269)
- ranged_accuracy_skill_term(733) = 200 + floor((733-200) × 0.9) = 200 + 479 = 679
- 装備飛命 = 460
- accuracy_bonus = 0 (trait) + 36 (gift slot3) + 0 (JPcat) = 36
- ranged_accuracy_extra = 0 (gift) + 20 (COR JP idx7「遠隔命中アップ」max rank20) = 20

### 計算

```
飛命 = int(AGI × 0.75) + ranged_accuracy_skill_term(skill) + equip_racc
       + accuracy_bonus + ranged_accuracy_extra
     = int(378 × 0.75=283.5) + 679 + 460 + 36 + 20
     = 283 + 679 + 460 + 36 + 20
     = 1478
```

## 実装

| 場所 | 役割 |
|---|---|
| `rust/src/status.rs:calc_accuracy` | メイン命中の基本式 |
| `rust/src/status.rs:calc_ranged_accuracy` | 飛命の基本式 |
| `rust/src/status.rs:accuracy_skill_term` | 近接スキル区分値（4段階） |
| `rust/src/status.rs:ranged_accuracy_skill_term` | 遠隔スキル区分値（2段階） |
| `rust/src/wasm.rs:chara_to_status_result` | accuracy_bonus / ranged_accuracy_extra 加算と最終出力 |

## 未対応項目

- 装備の "Weapon Skill Accuracy+X" は通常命中扱いしない（WS 専用、メイン命中に含めない）
  - 実装: `web/js/equip-stats.js` の Accuracy regex に `(?<![Ss]kill )` lookbehind あり
- 食事による命中ブースト
- ジョブアビリティ系（フォーカス・ダンスサンバ等）の一時補正
- 連携状態・ウェポンスキル中の補正
- COR JP idx 9「適正距離の遠隔攻撃力アップ」（条件付きのため未反映）
