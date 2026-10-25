# 回避 (Evasion) 計算仕様

参考: [wiki.ffo.jp/html/1688.html](https://wiki.ffo.jp/html/1688.html)

## 総合計算式

```
回避合計 = int(AGI × 0.5) + 回避スキル区分値 + 装備回避 + evasion_bonus
```

## 回避スキル区分値

回避スキル値による区分的線形補間:

| スキル範囲 | 寄与 |
|---|---|
| ≤ 200 | スキル値そのまま |
| 201-400 | 200 + int((skill - 200) × 0.9) |
| 401- | 380 + int((skill - 400) × 0.8) |

具体値:

| skill | term |
|---|---|
| 100 | 100 |
| 200 | 200 |
| 300 | 290 |
| 400 | 380 |
| 500 | 460 |
| 600 | 540 |

> 命中の `accuracy_skill_term` と異なり、回避は **400 以上は 0.8 / Lv** で頭打ち（601 以上の特殊区分なし）。

## 回避スキル有効値

```
有効回避スキル = min(キャラ Evasion スキル値, スキル上限) + メリットボーナス + 全スロット共通スキルボーナス + ギフトのスキルボーナス
```

| 項 | 内容 |
|---|---|
| `スキル上限` | max(`job_skill_cap(main_job, Evasion, main_lv, master_lv)`, `job_skill_cap(support_job, Evasion, support_lv, 0)`)。求め方は [skill_cap.md](skill_cap.md) |
| `メリットボーナス` | Evasion メリット rank × 2（rank 0-8、最大 +16）。スキル上限で抑えた後に足す |
| `全スロット共通スキルボーナス` | 装備の Evasion skill+ 合計 |
| `ギフトのスキルボーナス` | ギフトの回避スキルのボーナス（無いジョブは 0） |

メインジョブとサポートジョブのどちらも回避スキルを持たない（スキル上限 0 の）ときも、メリットボーナス・全スロット共通スキルボーナス・ギフトのスキルボーナスは足す。

> 2026-10-18 にユーザーの判断で改めた（ゲームの一次情報では未確認）。改める前は `base = min(キャラ Evasion スキル値, ジョブキャップ + メリットボーナス)`
> （メリットボーナスはスキル上限を広げる）としていた。今の実装（`rust/src/skills.rs` の `effective_skill_parts`）は改める前の規則のままで、直す対象である。

> ※ 回避スキルは武器ではないので、装備のメイン/サブ/レンジスロット区別なく **すべて global** として加算。

## evasion_bonus

```
evasion_bonus = ジョブ特性 EvasionBonus(main/sup 最大)
              + ギフト physical_evasion
              + JPカテゴリ physical_evasion
```

### ジョブ特性 EvasionBonus

ランクごとの累積値:

| Rank | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| 値 | 10 | 22 | 35 | 48 | 60 | 72 |

取得ジョブ・レベル:

- **THF**: Lv10, 30, 50, 70, 76, 88（最大 rank6 = +72）
- **DNC**: Lv15, 45, 75, 86（最大 rank4 = +48）
- **PUP**: Lv20, 40, 60, 76（最大 rank4 = +48）

### ギフト physical_evasion

War 全解放（2100JP）= [5, 8, 10, 13] の累積 = **+36**

### JP カテゴリ physical_evasion

特定ジョブの一部カテゴリが直接 physical_evasion に寄与する場合のみ加算。

## 実装

| 場所 | 役割 |
|---|---|
| `rust/src/status.rs:calc_evasion` | 基本式 |
| `rust/src/wasm.rs:chara_to_status_result` | スキル合算と evasion_bonus 加算 |

## 未対応項目

- 食事
- ジョブアビリティ（流転の構え・パフォルマンス・サンバ等）
- ステータス異常（暗闇による命中-/回避±など）
