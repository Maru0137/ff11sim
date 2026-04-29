# 魔法攻撃力 (Magic Attack) 計算仕様

参考: [wiki.ffo.jp/html/3411.html](https://wiki.ffo.jp/html/3411.html)

## 総合計算式

```
魔法攻撃力 = 100 + 装備魔攻 + magic_attack_bonus
```

> ※ 物理攻撃と異なり STR 等のステータス項を持たず、固定値 100 をベースとする。

## magic_attack_bonus

```
magic_attack_bonus = ジョブ特性 MagicAttackBonus(main/sup 最大)
                   + ギフト magic_attack
                   + JPカテゴリ magic_attack
```

### ジョブ特性 MagicAttackBonus

ランクごとの累積値:

| Rank | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| 値 | 20 | 24 | 28 | 32 | 36 | 40 |

> ※ AttackBonus 等と異なり、rank1 で +20 から始まる線形増加（+4/rank）。

取得ジョブ・レベル:

- **BLM**: Lv10, 30, 50, 70, 81, 91（最大 rank6 = +40）
- **RDM**: Lv20, 40, 86（最大 rank3 = +28）

### ギフト magic_attack

メインジョブ累計 JP 量で解放されるティアの累積。
各ジョブ値は `rust/src/job_points.rs:JOB_GIFTS` を参照。

### JP カテゴリ magic_attack

特定ジョブの一部カテゴリが直接 magic_attack に寄与する場合のみ加算。

## 実装

| 場所 | 役割 |
|---|---|
| `rust/src/status.rs:calc_magic_attack` | 基本式（100 + 装備分） |
| `rust/src/wasm.rs:chara_to_status_result` | ジョブ特性・ギフト・JPカテゴリの加算 |

## 未対応項目

- 魔法スキル別の補正（精霊/暗黒/回復など）はステータス値ではなくダメージ計算側で扱う
- "Magic Attack Bonus" ジョブアビリティ（ボイル等）
- 食事
- 属性別 +Magic Damage 装備のダメージ寄与
