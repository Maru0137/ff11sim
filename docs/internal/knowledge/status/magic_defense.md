# 魔法防御 (Magic Defense) 計算仕様

参考: [wiki.ffo.jp/html/14a.html](https://wiki.ffo.jp/html/14a.html)

## 総合計算式

```
魔法防御 = 100 + 装備魔防 + ジョブ特性 MagicDefenseBonus(main/sup 最大)
        + ギフト magic_defense
        + JPカテゴリ magic_defense
```

> ※ 物理防御と異なり VIT/Lv 項を持たず、固定値 100 をベースとする。

## ジョブ特性 MagicDefenseBonus

ランクごとの累積値:

| Rank | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|
| 値 | 10 | 12 | 14 | 16 | 18 | 20 | 22 |

> ※ 他の Bonus 系と異なり、rank2 以降は +2/rank の線形増加。

取得ジョブ・レベル:

- **WHM**: Lv10, 30, 50, 70, 81, 91（最大 rank6 = +20）
- **RDM**: Lv25, 45, 96（最大 rank3 = +14）
- **RUN**: Lv10, 30, 50, 70, 76, 91, 99（最大 rank7 = +22）

## ギフト magic_defense

メインジョブ累計 JP 量で解放されるティアの累積。
各ジョブ値は `rust/src/job_points.rs:JOB_GIFTS` を参照。

## JP カテゴリ magic_defense

特定ジョブの一部カテゴリが直接 magic_defense に寄与する場合のみ加算。

## 実装

| 場所 | 役割 |
|---|---|
| `rust/src/status.rs:calc_magic_defense` | 基本式（100 + 装備分） |
| `rust/src/wasm.rs:chara_to_status_result` | ジョブ特性・ギフト・JPカテゴリの加算 |

## 未対応項目

- "Shell" 等の魔法防御アップ系強化魔法
- "Magic Defense Bonus" ジョブアビリティ
- 食事
- 属性耐性は魔法防御とは別軸（ダメージ計算側）
