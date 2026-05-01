use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::job::Job;

/// ジョブポイントのカテゴリ数（各ジョブ10カテゴリ）
pub const JP_CATEGORY_COUNT: usize = 10;
/// ジョブポイントカテゴリの最大ランク
pub const JP_MAX_RANK: i32 = 20;

/// ギフトのステータス種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GiftStatKind {
    PhysicalAttack,
    PhysicalDefense,
    PhysicalAccuracy,
    PhysicalEvasion,
    MagicAttack,
    MagicDefense,
    MagicAccuracy,
    MagicEvasion,
    StoreTp,
    /// 遠隔攻撃力（飛攻）。物理攻撃力には加算されない
    RangedAttack,
    /// 遠隔命中（飛命）。物理命中には加算されない
    RangedAccuracy,
    /// Skill bonuses, capacity points, automaton bonuses など、
    /// 現状のステータス計算には反映しないギフトの placeholder
    None,
}

/// 1 ジョブ分のジョブポイントカテゴリランク情報。
/// 各カテゴリは 0..=20 のランクを持ち、ランク r まで振るために必要な JP は r*(r+1)/2。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JobPointCategories {
    pub ranks: [i32; JP_CATEGORY_COUNT],
}

impl Default for JobPointCategories {
    fn default() -> Self {
        Self {
            ranks: [0; JP_CATEGORY_COUNT],
        }
    }
}

impl JobPointCategories {
    /// 各カテゴリが最大ランク（20）の状態を返す
    pub fn all_maxed() -> Self {
        Self {
            ranks: [JP_MAX_RANK; JP_CATEGORY_COUNT],
        }
    }

    /// このジョブに投入した累計 JP の合計を返す。
    /// ランク r のコスト = r*(r+1)/2 (1+2+...+r)
    pub fn total_jp_spent(&self) -> i32 {
        self.ranks
            .iter()
            .map(|&r| {
                assert!(
                    r >= 0 && r <= JP_MAX_RANK,
                    "JP category rank must be between 0 and {}: {}",
                    JP_MAX_RANK,
                    r
                );
                r * (r + 1) / 2
            })
            .sum()
    }
}

/// 全ジョブ分の JP カテゴリ情報
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobPoints {
    #[serde(default)]
    pub categories: EnumMap<Job, JobPointCategories>,
}

/// 単一ジョブのギフトによる戦闘系ステータスボーナス
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GiftBonuses {
    pub physical_attack: i32,
    pub physical_defense: i32,
    pub physical_accuracy: i32,
    pub physical_evasion: i32,
    pub magic_attack: i32,
    pub magic_defense: i32,
    pub magic_accuracy: i32,
    pub magic_evasion: i32,
    pub store_tp: i32,
    /// 遠隔系専用（メイン攻撃には加算しない）
    pub ranged_attack: i32,
    pub ranged_accuracy: i32,
}

impl GiftBonuses {
    fn add(&mut self, stat: GiftStatKind, value: i32) {
        match stat {
            GiftStatKind::PhysicalAttack => self.physical_attack += value,
            GiftStatKind::PhysicalDefense => self.physical_defense += value,
            GiftStatKind::PhysicalAccuracy => self.physical_accuracy += value,
            GiftStatKind::PhysicalEvasion => self.physical_evasion += value,
            GiftStatKind::MagicAttack => self.magic_attack += value,
            GiftStatKind::MagicDefense => self.magic_defense += value,
            GiftStatKind::MagicAccuracy => self.magic_accuracy += value,
            GiftStatKind::MagicEvasion => self.magic_evasion += value,
            GiftStatKind::StoreTp => self.store_tp += value,
            GiftStatKind::RangedAttack => self.ranged_attack += value,
            GiftStatKind::RangedAccuracy => self.ranged_accuracy += value,
            GiftStatKind::None => {}
        }
    }
}

/// ギフト 1 スロット分の定義（ステータス種別 + 4 ティア）。
/// 各ティアは `(累計 JP 閾値, 効果値)` の組で、閾値は単調増加する。
/// 同一ステータスでもジョブごとに閾値が異なる場合があるため、スロット側に閾値を埋め込む。
#[derive(Debug, Clone, Copy)]
struct GiftSlotDef {
    stat: GiftStatKind,
    tiers: [(i32, i32); 4],
}

const fn slot(stat: GiftStatKind, tiers: [(i32, i32); 4]) -> GiftSlotDef {
    GiftSlotDef { stat, tiers }
}

use GiftStatKind::*;

/// 22 ジョブ分のギフト（ステータス系）スロット定義。
/// Job enum の順序に従う: War, Mnk, Whm, Blm, Rdm, Thf, Pld, Drk, Bst, Brd, Rng, Sam,
///                       Nin, Drg, Smn, Blu, Cor, Pup, Dnc, Sch, Geo, Run
/// スロット数はジョブごとに異なる（最大 7）。
const JOB_GIFTS: [&'static [GiftSlotDef]; 22] = [
    // War
    &[
        slot(PhysicalDefense, [(5, 10), (180, 15), (605, 20), (1280, 25)]),
        slot(PhysicalAttack, [(10, 10), (210, 15), (660, 20), (1360, 25)]),
        slot(PhysicalEvasion, [(20, 5), (245, 8), (720, 10), (1445, 13)]),
        slot(PhysicalAccuracy, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Mnk
    &[
        slot(PhysicalDefense, [(5, 5), (180, 8), (605, 10), (1280, 13)]),
        slot(PhysicalAttack, [(10, 8), (210, 12), (660, 16), (1360, 20)]),
        slot(PhysicalEvasion, [(20, 6), (245, 9), (720, 12), (1445, 15)]),
        slot(PhysicalAccuracy, [(30, 6), (280, 9), (780, 11), (1530, 15)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Whm
    &[
        slot(MagicDefense, [(5, 7), (180, 11), (605, 14), (1280, 18)]),
        slot(MagicAttack, [(10, 3), (210, 5), (660, 6), (1360, 8)]),
        slot(MagicEvasion, [(20, 7), (245, 11), (720, 14), (1445, 18)]),
        slot(MagicAccuracy, [(30, 7), (280, 11), (780, 14), (1530, 18)]),
        slot(PhysicalAccuracy, [(45, 2), (320, 3), (845, 4), (1620, 5)]),
        slot(None, [(60, 0), (360, 0), (910, 0), (1710, 0)]),
    ],
    // Blm
    &[
        slot(MagicDefense, [(5, 2), (180, 3), (605, 4), (1280, 5)]),
        slot(MagicAttack, [(10, 7), (210, 11), (660, 14), (1360, 18)]),
        slot(MagicEvasion, [(20, 6), (245, 9), (720, 12), (1445, 15)]),
        slot(MagicAccuracy, [(30, 6), (280, 9), (780, 12), (1530, 15)]),
        slot(None, [(45, 0), (320, 0), (845, 0), (1620, 0)]),
        slot(None, [(60, 0), (360, 0), (910, 0), (1710, 0)]),
    ],
    // Rdm
    &[
        slot(MagicDefense, [(5, 4), (180, 6), (605, 8), (1280, 10)]),
        slot(MagicAttack, [(10, 4), (210, 6), (660, 8), (1360, 10)]),
        slot(MagicEvasion, [(20, 8), (245, 12), (720, 16), (1445, 20)]),
        slot(MagicAccuracy, [(30, 10), (280, 15), (780, 20), (1530, 25)]),
        slot(PhysicalAccuracy, [(45, 3), (320, 5), (845, 6), (1620, 8)]),
        slot(None, [(60, 0), (360, 0), (910, 0), (1710, 0)]),
    ],
    // Thf
    &[
        slot(PhysicalDefense, [(5, 4), (180, 6), (605, 8), (1280, 10)]),
        slot(PhysicalAttack, [(10, 7), (210, 11), (660, 14), (1360, 18)]),
        slot(PhysicalEvasion, [(20, 10), (245, 15), (720, 20), (1445, 25)]),
        slot(PhysicalAccuracy, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Pld
    &[
        slot(PhysicalDefense, [(5, 15), (180, 23), (605, 30), (1280, 38)]),
        slot(PhysicalAttack, [(10, 4), (210, 6), (660, 8), (1360, 10)]),
        slot(PhysicalEvasion, [(20, 3), (245, 5), (720, 6), (1445, 8)]),
        slot(PhysicalAccuracy, [(30, 4), (280, 6), (780, 8), (1530, 10)]),
        slot(MagicEvasion, [(45, 6), (320, 9), (845, 12), (1620, 15)]),
        slot(MagicAccuracy, [(60, 6), (360, 9), (910, 12), (1710, 15)]),
    ],
    // Drk
    &[
        slot(PhysicalDefense, [(5, 4), (180, 6), (605, 8), (1280, 10)]),
        slot(PhysicalAttack, [(10, 15), (210, 23), (660, 30), (1360, 38)]),
        slot(PhysicalEvasion, [(20, 3), (245, 5), (720, 6), (1445, 8)]),
        slot(PhysicalAccuracy, [(30, 3), (280, 5), (780, 6), (1530, 8)]),
        slot(MagicEvasion, [(45, 6), (320, 9), (845, 12), (1620, 15)]),
        slot(MagicAccuracy, [(60, 6), (360, 9), (910, 12), (1710, 15)]),
    ],
    // Bst
    &[
        slot(PhysicalDefense, [(5, 10), (180, 15), (605, 20), (1280, 25)]),
        slot(PhysicalAttack, [(10, 10), (210, 15), (660, 20), (1360, 25)]),
        slot(PhysicalEvasion, [(20, 5), (245, 8), (720, 10), (1445, 13)]),
        slot(PhysicalAccuracy, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Brd (値は wiki から取得できなかった部分があり保守的な値)
    &[
        slot(PhysicalDefense, [(5, 5), (180, 8), (605, 10), (1280, 13)]),
        slot(PhysicalEvasion, [(10, 5), (210, 8), (660, 10), (1360, 13)]),
        slot(PhysicalAccuracy, [(20, 5), (245, 8), (720, 10), (1445, 13)]),
        slot(MagicDefense, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Rng
    &[
        slot(PhysicalDefense, [(5, 3), (180, 5), (605, 8), (1280, 8)]),
        slot(PhysicalAttack, [(10, 10), (210, 15), (660, 20), (1360, 25)]),
        slot(PhysicalEvasion, [(20, 2), (245, 3), (720, 4), (1445, 5)]),
        slot(PhysicalAccuracy, [(30, 10), (280, 15), (780, 20), (1530, 25)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Sam
    &[
        slot(PhysicalDefense, [(5, 10), (180, 15), (605, 20), (1280, 25)]),
        slot(PhysicalAttack, [(10, 10), (210, 15), (660, 20), (1360, 25)]),
        slot(PhysicalEvasion, [(20, 5), (245, 8), (720, 10), (1445, 13)]),
        slot(PhysicalAccuracy, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Nin (魔法命中アップ: wikiwiki.jp/ffxi 忍者欄、80/405/980/1805 JP で +7/+11/+14/+18)
    &[
        slot(PhysicalDefense, [(5, 8), (180, 12), (605, 16), (1280, 20)]),
        slot(None, [(10, 0), (210, 0), (660, 0), (1360, 0)]), // Capacity Point Bonus
        slot(PhysicalAttack, [(20, 10), (245, 15), (720, 20), (1445, 25)]),
        slot(PhysicalEvasion, [(30, 9), (280, 14), (780, 18), (1530, 23)]),
        slot(PhysicalAccuracy, [(45, 8), (320, 12), (845, 16), (1620, 20)]),
        slot(MagicAttack, [(60, 4), (360, 6), (910, 8), (1710, 10)]),
        slot(MagicAccuracy, [(80, 7), (405, 11), (980, 14), (1805, 18)]),
    ],
    // Drg
    &[
        slot(PhysicalDefense, [(5, 10), (180, 15), (605, 20), (1280, 25)]),
        slot(PhysicalAttack, [(10, 10), (210, 15), (660, 20), (1360, 25)]),
        slot(PhysicalEvasion, [(20, 5), (245, 8), (720, 10), (1445, 13)]),
        slot(PhysicalAccuracy, [(30, 9), (280, 14), (780, 18), (1530, 23)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Smn
    &[
        slot(MagicDefense, [(5, 3), (180, 5), (605, 6), (1280, 8)]),
        slot(MagicEvasion, [(10, 3), (210, 5), (660, 6), (1360, 8)]),
        slot(PhysicalDefense, [(20, 3), (245, 5), (720, 6), (1445, 8)]),
        slot(PhysicalEvasion, [(30, 3), (280, 5), (780, 6), (1530, 8)]),
        slot(None, [(45, 0), (320, 0), (845, 0), (1620, 0)]), // Avatar/Spirit Attack & Defense
        slot(None, [(60, 0), (360, 0), (910, 0), (1710, 0)]), // Avatar/Spirit Accuracy & Evasion
    ],
    // Blu (wiki のデータが不完全、保守的なデフォルト)
    // 魔法命中アップは本来 125/450/1050/1900 JP の閾値で +5/+8/+10/+13 (累計 +36)
    &[
        slot(PhysicalDefense, [(5, 10), (180, 15), (605, 20), (1280, 25)]),
        slot(None, [(10, 0), (210, 0), (660, 0), (1360, 0)]),
        slot(PhysicalAttack, [(20, 10), (245, 15), (720, 20), (1445, 25)]),
        slot(PhysicalEvasion, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(None, [(45, 0), (320, 0), (845, 0), (1620, 0)]),
        slot(PhysicalAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
        slot(MagicAccuracy, [(125, 5), (450, 8), (1050, 10), (1900, 13)]),
    ],
    // Cor (ギフトデータは wikiwiki.jp/ffxi コルセア欄を参照、累計 1805JP まで列挙されている範囲)
    &[
        slot(PhysicalDefense, [(5, 3), (180, 5), (605, 6), (1280, 8)]),
        slot(PhysicalAttack, [(10, 5), (210, 8), (660, 10), (1360, 13)]),
        slot(PhysicalEvasion, [(20, 3), (245, 5), (720, 6), (1445, 8)]),
        slot(PhysicalAccuracy, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(MagicAttack, [(45, 2), (320, 3), (845, 4), (1620, 5)]),
        slot(MagicEvasion, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
        slot(MagicAccuracy, [(80, 5), (405, 8), (980, 10), (1805, 13)]),
    ],
    // Pup
    &[
        slot(PhysicalAttack, [(5, 6), (180, 9), (605, 12), (1280, 15)]),
        slot(PhysicalEvasion, [(10, 8), (210, 12), (660, 16), (1360, 20)]),
        slot(PhysicalAccuracy, [(20, 7), (245, 11), (720, 14), (1445, 18)]),
        slot(MagicEvasion, [(30, 5), (280, 8), (780, 10), (1530, 13)]),
        slot(MagicAccuracy, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(None, [(60, 0), (360, 0), (910, 0), (1710, 0)]), // Automaton Physical Attack/Defense
    ],
    // Dnc
    &[
        slot(PhysicalDefense, [(5, 6), (180, 9), (605, 12), (1280, 15)]),
        slot(PhysicalAttack, [(10, 6), (210, 9), (660, 12), (1360, 15)]),
        slot(PhysicalEvasion, [(20, 9), (245, 14), (720, 18), (1445, 23)]),
        slot(PhysicalAccuracy, [(30, 9), (280, 14), (780, 18), (1530, 23)]),
        slot(MagicEvasion, [(45, 5), (320, 8), (845, 10), (1620, 13)]),
        slot(MagicAccuracy, [(60, 5), (360, 8), (910, 10), (1710, 13)]),
    ],
    // Sch (wiki のデータが不完全、保守的なデフォルト)
    &[
        slot(MagicDefense, [(5, 3), (180, 5), (605, 6), (1280, 8)]),
        slot(MagicAttack, [(10, 5), (210, 8), (660, 10), (1360, 13)]),
        slot(MagicEvasion, [(20, 6), (245, 9), (720, 12), (1445, 15)]),
        slot(MagicAccuracy, [(30, 6), (280, 9), (780, 12), (1530, 15)]),
        slot(None, [(45, 0), (320, 0), (845, 0), (1620, 0)]),
        slot(None, [(60, 0), (360, 0), (910, 0), (1710, 0)]),
    ],
    // Geo
    &[
        slot(MagicDefense, [(5, 4), (180, 6), (605, 8), (1280, 10)]),
        slot(MagicAttack, [(10, 6), (210, 9), (660, 12), (1360, 15)]),
        slot(MagicEvasion, [(20, 7), (245, 11), (720, 14), (1445, 18)]),
        slot(MagicAccuracy, [(30, 7), (280, 11), (780, 14), (1530, 18)]),
        slot(None, [(45, 0), (320, 0), (845, 0), (1620, 0)]),
        slot(None, [(60, 0), (360, 0), (910, 0), (1710, 0)]),
    ],
    // Run (魔法命中アップ: wikiwiki.jp/ffxi 魔導剣士欄、80/405/980/1805 JP で +5/+8/+10/+13)
    &[
        slot(PhysicalDefense, [(5, 5), (180, 8), (605, 10), (1280, 13)]),
        slot(PhysicalAttack, [(10, 7), (210, 11), (660, 14), (1360, 18)]),
        slot(PhysicalEvasion, [(20, 8), (245, 12), (720, 16), (1445, 20)]),
        slot(PhysicalAccuracy, [(30, 8), (280, 12), (780, 16), (1530, 20)]),
        slot(MagicDefense, [(45, 8), (320, 12), (845, 16), (1620, 20)]),
        slot(MagicEvasion, [(60, 10), (360, 15), (910, 20), (1710, 25)]),
        slot(MagicAccuracy, [(80, 5), (405, 8), (980, 10), (1805, 13)]),
    ],
];

/// ジョブポイントカテゴリの直接ステータス効果（ランクあたり）。
/// ほとんどのカテゴリはアビリティ固有の効果だが、一部のカテゴリは
/// 戦闘ステータスを直接強化する。
/// Some((category_index, GiftStatKind, per_rank_value)) で定義。
#[derive(Debug, Clone, Copy)]
struct JpCategoryEffect {
    category_index: usize,
    stat: GiftStatKind,
    per_rank: i32,
}

/// 各ジョブの JP カテゴリによる直接ステータス効果
fn jp_category_effects(job: Job) -> &'static [JpCategoryEffect] {
    match job {
        // War: category 9 は「ダブルアタック効果」(物理攻撃力 +1/rank)
        Job::War => &[JpCategoryEffect {
            category_index: 9,
            stat: PhysicalAttack,
            per_rank: 1,
        }],
        // Cor:
        //   category 7 「遠隔命中アップ」: 飛命 +1/rank
        //   category 9 「適正距離の遠隔攻撃力アップ」: 飛攻 +2/rank は条件付き（適正距離）のため
        //   ステータス表示には反映しない
        Job::Cor => &[JpCategoryEffect {
            category_index: 7,
            stat: RangedAccuracy,
            per_rank: 1,
        }],
        // Whm: category 3 は Magic Accuracy Bonus (+1 MACC/rank)
        Job::Whm => &[JpCategoryEffect {
            category_index: 3,
            stat: MagicAccuracy,
            per_rank: 1,
        }],
        // Blm: category 5 は Magic Accuracy Bonus (+1 MACC/rank)
        Job::Blm => &[JpCategoryEffect {
            category_index: 5,
            stat: MagicAccuracy,
            per_rank: 1,
        }],
        // Rdm: category 3 は Magic Accuracy Bonus, category 5 は Magic Atk. Bonus
        Job::Rdm => &[
            JpCategoryEffect {
                category_index: 3,
                stat: MagicAccuracy,
                per_rank: 1,
            },
            JpCategoryEffect {
                category_index: 5,
                stat: MagicAttack,
                per_rank: 1,
            },
        ],
        _ => &[],
    }
}

/// 戦士の「ダブルアタック確率アップ」ギフトによる発動率 (%) を計算する。
/// 累計 JP の閾値で +2 / +2 / +3 / +3 (累計 +10%)。
/// 閾値: 125 / 450 / 1050 / 1900 JP
pub fn calc_war_da_gift_bonus(total_jp: i32) -> i32 {
    const THRESHOLDS: [(i32, i32); 4] = [(125, 2), (450, 2), (1050, 3), (1900, 3)];
    let mut bonus = 0;
    for &(req, val) in &THRESHOLDS {
        if total_jp >= req {
            bonus += val;
        } else {
            break;
        }
    }
    bonus
}

/// 累計 JP 量から得られるギフトのボーナス合計を計算する。
/// ギフトは各ティアのボーナスが累積加算される（例: 7 → 11 → 14 → 18 と解放されると 7+11+14+18=50）。
pub fn calc_gift_bonuses(job: Job, total_jp: i32) -> GiftBonuses {
    let mut bonuses = GiftBonuses::default();
    let job_gifts = JOB_GIFTS[job as usize];
    for slot_def in job_gifts {
        let mut slot_total = 0;
        for &(threshold, value) in &slot_def.tiers {
            if total_jp >= threshold {
                slot_total += value;
            } else {
                break;
            }
        }
        if slot_total != 0 {
            bonuses.add(slot_def.stat, slot_total);
        }
    }
    bonuses
}

/// JP カテゴリの直接ステータス効果を計算する
pub fn calc_jp_category_bonuses(job: Job, categories: &JobPointCategories) -> GiftBonuses {
    let mut bonuses = GiftBonuses::default();
    for effect in jp_category_effects(job) {
        let rank = categories.ranks[effect.category_index];
        bonuses.add(effect.stat, effect.per_rank * rank);
    }
    bonuses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_jp_spent_default() {
        let cats = JobPointCategories::default();
        assert_eq!(cats.total_jp_spent(), 0);
    }

    #[test]
    fn test_total_jp_spent_all_maxed() {
        // 10 categories × (20*21/2 = 210) = 2100
        let cats = JobPointCategories::all_maxed();
        assert_eq!(cats.total_jp_spent(), 2100);
    }

    #[test]
    fn test_total_jp_spent_partial() {
        // rank 1 = 1, rank 5 = 15, rank 10 = 55
        let mut cats = JobPointCategories::default();
        cats.ranks[0] = 1;
        cats.ranks[1] = 5;
        cats.ranks[2] = 10;
        assert_eq!(cats.total_jp_spent(), 1 + 15 + 55);
    }

    #[test]
    fn test_gift_bonuses_war_maxed() {
        // War at 2100 JP: 各スロットの 4 ティアがすべて解放され累積加算される
        // PDEF: 10+15+20+25 = 70, PATK: 同じく 70
        // PEVA/PACC/MEVA/MACC: 5+8+10+13 = 36
        let bonuses = calc_gift_bonuses(Job::War, 2100);
        assert_eq!(bonuses.physical_defense, 70);
        assert_eq!(bonuses.physical_attack, 70);
        assert_eq!(bonuses.physical_evasion, 36);
        assert_eq!(bonuses.physical_accuracy, 36);
        assert_eq!(bonuses.magic_evasion, 36);
        assert_eq!(bonuses.magic_accuracy, 36);
        assert_eq!(bonuses.magic_attack, 0);
    }

    #[test]
    fn test_gift_bonuses_war_zero() {
        let bonuses = calc_gift_bonuses(Job::War, 0);
        assert_eq!(bonuses, GiftBonuses::default());
    }

    #[test]
    fn test_gift_bonuses_war_tier1_only() {
        // War at 60 JP: tier 1 active for all 6 slots (slot 0..=5 tier 1 thresholds are 5,10,20,30,45,60)
        let bonuses = calc_gift_bonuses(Job::War, 60);
        assert_eq!(bonuses.physical_defense, 10);
        assert_eq!(bonuses.physical_attack, 10);
        assert_eq!(bonuses.physical_evasion, 5);
        assert_eq!(bonuses.physical_accuracy, 5);
        assert_eq!(bonuses.magic_evasion, 5);
        assert_eq!(bonuses.magic_accuracy, 5);
    }

    #[test]
    fn test_gift_bonuses_war_boundary() {
        // At exactly 4 JP: no gift active (slot 0 tier 1 needs 5 JP)
        let bonuses = calc_gift_bonuses(Job::War, 4);
        assert_eq!(bonuses.physical_defense, 0);
        // At 5 JP: slot 0 tier 1 (+10 PDEF) is active
        let bonuses = calc_gift_bonuses(Job::War, 5);
        assert_eq!(bonuses.physical_defense, 10);
        assert_eq!(bonuses.physical_attack, 0);
    }

    #[test]
    fn test_gift_bonuses_whm_maxed() {
        // Whm at 2100 JP: 各スロットの 4 ティアがすべて解放され累積加算される
        // MDEF: 7+11+14+18 = 50
        // MATK: 3+5+6+8 = 22
        // MEVA/MACC: 7+11+14+18 = 50
        // PACC: 2+3+4+5 = 14
        let bonuses = calc_gift_bonuses(Job::Whm, 2100);
        assert_eq!(bonuses.magic_defense, 50);
        assert_eq!(bonuses.magic_attack, 22);
        assert_eq!(bonuses.magic_evasion, 50);
        assert_eq!(bonuses.magic_accuracy, 50);
        assert_eq!(bonuses.physical_accuracy, 14);
        assert_eq!(bonuses.physical_attack, 0);
        assert_eq!(bonuses.physical_defense, 0);
    }

    #[test]
    fn test_jp_category_bonuses_whm() {
        // Whm の category 3 (Magic Accuracy Bonus) のランクが MACC に反映される
        let mut cats = JobPointCategories::default();
        cats.ranks[3] = 20;
        let bonuses = calc_jp_category_bonuses(Job::Whm, &cats);
        assert_eq!(bonuses.magic_accuracy, 20);
        assert_eq!(bonuses.magic_attack, 0);
    }

    #[test]
    fn test_jp_category_bonuses_rdm() {
        // Rdm の category 3 は MACC、category 5 は MATK
        let mut cats = JobPointCategories::default();
        cats.ranks[3] = 20;
        cats.ranks[5] = 20;
        let bonuses = calc_jp_category_bonuses(Job::Rdm, &cats);
        assert_eq!(bonuses.magic_accuracy, 20);
        assert_eq!(bonuses.magic_attack, 20);
    }

    #[test]
    fn test_jp_category_bonuses_war() {
        // War のカテゴリ idx 9「ダブルアタック効果」は物理攻撃力 +1/rank。
        // 全カテゴリ rank=20 で +20 物理攻撃力のみ反映される。
        let cats = JobPointCategories::all_maxed();
        let bonuses = calc_jp_category_bonuses(Job::War, &cats);
        assert_eq!(bonuses.physical_attack, 20);
        assert_eq!(bonuses.physical_defense, 0);
        assert_eq!(bonuses.physical_accuracy, 0);
    }

    #[test]
    fn test_war_da_gift_bonus() {
        // 戦士のダブルアタック確率アップギフト累計
        assert_eq!(calc_war_da_gift_bonus(0), 0);
        assert_eq!(calc_war_da_gift_bonus(124), 0);
        assert_eq!(calc_war_da_gift_bonus(125), 2);
        assert_eq!(calc_war_da_gift_bonus(450), 4);
        assert_eq!(calc_war_da_gift_bonus(1050), 7);
        assert_eq!(calc_war_da_gift_bonus(1900), 10);
        assert_eq!(calc_war_da_gift_bonus(2100), 10);
    }

    #[test]
    fn test_job_points_default() {
        // デフォルト値はすべて 0 ランク
        let jp = JobPoints::default();
        assert_eq!(jp.categories[Job::War].total_jp_spent(), 0);
        assert_eq!(jp.categories[Job::Whm].total_jp_spent(), 0);
    }
}
