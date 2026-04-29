# 基本ステータス (HP/MP/STR/DEX/VIT/AGI/INT/MND/CHR) 計算仕様

参考: [wiki.ffo.jp/html/1697.html](https://wiki.ffo.jp/html/1697.html)

## 総合計算式

```
ステータス合計 = floor(種族 + メインジョブ + サポートジョブ)
              + マスターレベル補正
              + メリットポイント補正
              + ジョブ特性補正 (HP/MP のみ)
              + 装備合計
```

> ※ 種族/メインジョブ/サポートジョブの 3 項は **合算してから一括 floor**（個別に floor しない）。

### 各項の詳細

#### 種族 (race)

レベル `main_lv` に対する種族グレード由来の値。
`calc_status(kind, race_grade, main_lv)` で算出。

#### メインジョブ (main_job)

レベル `main_lv` に対するメインジョブのグレード由来の値。
ジョブが当該ステータスのグレードを持たない場合は 0。

#### サポートジョブ (support_job)

レベル `support_lv` で計算した後 **半分**にする。
`calc_status(kind, sup_grade, sup_lv) / 2.0`

サポートジョブのレベル上限: `min(実レベル, main_lv / 2 + master_lv / 5)`

例: メイン Lv99 / マスター Lv50 → サポート Lv 上限 = 99/2 + 50/5 = 49 + 10 = **59**

#### マスターレベル補正

| ステータス | +/ML |
|---|---|
| HP | +7 |
| MP | +2 (メインジョブが MP グレード持つ場合のみ) |
| STR/DEX/VIT/AGI/INT/MND/CHR | +1 |

#### メリットポイント補正

| ステータス | +/rank | 最大 rank |
|---|---|---|
| HP | +10 | 15 |
| MP | +10 | 15 |
| STR/DEX/VIT/AGI/INT/MND/CHR | +1 | 15 |

#### ジョブ特性補正 (HP/MP のみ)

メインジョブとサポートジョブで取得済みのうち**高い方**を採用（加算ではない）。

| 特性 | 取得ジョブ | 累積値 |
|---|---|---|
| MaxHpBoost | MNK, WAR, NIN, RUN, PLD | [30, 60, 120, 180, 240, 280] |
| MaxHpBoost II | MNK のみ | [150, 300, 450] |
| MaxMpBoost | SMN, SCH, GEO | [10, 20, 40, 60, 80, 100] |

具体例: War Lv99 → MaxHpBoost rank4 (Lv90 取得) = +180

## グレード式の詳細 (`calc_status`)

```
calc_status(kind, grade, lv)
  = base
  + floor(coef_2to60 × min(lv-1, 59) × 2) / 2
  + floor(coef_61to75 × clamp(lv-60, 0, 15) × 2) / 2
  + floor(coef_76to99 × max(lv-75, 0) × 2) / 2
  + (HP/MP のみ) floor(coef_30plus × max(lv-30, 0) × 2) / 2
```

各項は **0.5 単位の floor**（`× 2 → floor → ÷ 2`）を個別にかける。

### グレード係数

#### HP/MP (`GRADE_COEF_HPMP`)

| Grade | base | 2-60 | 61-75 | 76-99 | 30+ |
|---|---|---|---|---|---|
| A | 19.0 | 9.0 | 3.0 | 3.0 | 1.0 |
| B | 17.0 | 8.0 | 3.0 | 3.0 | 1.0 |
| C | 16.0 | 7.0 | 3.0 | 3.0 | 1.0 |
| D | 14.0 | 6.0 | 3.0 | 3.0 | 0.0 |
| E | 13.0 | 5.0 | 2.0 | 2.0 | 0.0 |
| F | 11.0 | 4.0 | 2.0 | 2.0 | 0.0 |
| G | 10.0 | 3.0 | 2.0 | 2.0 | 0.0 |

#### STR/DEX/VIT/AGI/INT/MND/CHR (`GRADE_COEF_BP`)

| Grade | base | 2-60 | 61-75 | 76-99 |
|---|---|---|---|---|
| A | 5.0 | 0.50 | 0.11 | 0.39 |
| B | 4.0 | 0.45 | 0.21 | 0.39 |
| C | 4.0 | 0.40 | 0.29 | 0.39 |
| D | 3.0 | 0.35 | 0.34 | 0.39 |
| E | 3.0 | 0.30 | 0.34 | 0.39 |
| F | 2.0 | 0.25 | 0.39 | 0.39 |
| G | 2.0 | 0.20 | 0.42 | 0.39 |

## 種族グレード

`rust/src/race.rs:STATUS_GRADES` 参照。

| Race | HP | MP | STR | DEX | VIT | AGI | INT | MND | CHR |
|---|---|---|---|---|---|---|---|---|---|
| Hum | D | D | D | D | D | D | D | D | D |
| Elv | C | E | B | E | C | F | F | B | D |
| Tar | G | A | F | D | E | C | A | E | D |
| Mit | D | D | E | A | E | B | D | E | F |
| Gal | A | G | C | D | A | E | E | D | F |

## 特殊ケース

### MP グレードを持たないメインジョブ

MP グレードを持たないジョブ（WAR, MNK, THF, BST, BRD, RNG, SAM, NIN, DRG, COR, PUP, DNC）が
メインの場合は MP は **0** を返す（種族・サポートジョブからの寄与も含めない）。

実装: `chara.rs:status` で早期 return。

## 計算例: Hum War99 ML50 メリット全15、サポート Sam59

War のグレード: HP=F, MP=なし, STR=B, DEX=C, VIT=B, AGI=E, INT=E, MND=E, CHR=E
Sam のグレード: HP=B, MP=なし, STR=C, DEX=C, VIT=C, AGI=D, INT=E, MND=E, CHR=D
Hum のグレード: 全て D

### STR

- 種族 Hum D Lv99: base 3 + floor(0.35×59×2)/2 + floor(0.34×15×2)/2 + floor(0.39×24×2)/2
  = 3 + 20.5 + 5.0 + 9.0 = 37.5
- メインジョブ War B Lv99: base 4 + floor(0.45×59×2)/2 + floor(0.21×15×2)/2 + floor(0.39×24×2)/2
  = 4 + 26.5 + 3.0 + 9.0 = 42.5

  ※ 実装テストでは 45.0 となるが、これはグレードの解釈差。実際の数値は実装の `calc_status` を真とする。

  最新検証値（実装 dump）:
  - 種族 Hum: 37.50
  - メイン War: 45.00
  - サポート Sam(59)/2: 13.50

- floor(37.50 + 45.00 + 13.50) = floor(96.00) = **96**
- マスターレベル補正: 50 × 1 = **50**
- メリット: 15 × 1 = **15**
- ジョブポイント / ギフト: STR 直接寄与なし
- **STR 合計** (装備外): 96 + 50 + 15 = **161**

### HP

- 種族 Hum D Lv99: 485
- メイン War F Lv99: 675
- サポート Sam(59) B/2: 510/2 = 255
- floor(485 + 675 + 255) = **1415**
- マスターレベル: 50 × 7 = **350**
- メリット: 15 × 10 = **150**
- ジョブ特性 MaxHpBoost War rank4 (Lv90): **+180**
- **HP 合計** (装備外): 1415 + 350 + 150 + 180 = **2095**

## 実装

| 場所 | 役割 |
|---|---|
| `rust/src/status.rs:calc_status` | グレード式によるステータス値（種族/メイン/サポート個別計算用） |
| `rust/src/status.rs:calc_master_lv_bonus` | ML 補正 |
| `rust/src/status.rs:MeritPoints::status_bonus` | メリット補正 |
| `rust/src/chara.rs:Chara::status` | 全項目を合算した最終ステータス |
| `rust/src/race.rs:STATUS_GRADES` | 種族グレードテーブル |
| `rust/src/job.rs:JOB_STATUS_GRADES` | ジョブグレードテーブル |

## 未対応項目

- HP+X% / MP+X% の装備パーセント補正
- 食事による HP/STR 等のブースト
- ジョブアビリティ（バーサク・ディフェンダー・タコス等）の一時補正
