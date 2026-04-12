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
            GiftStatKind::None => {}
        }
    }
}

/// ギフト 1 スロット分の定義（ステータス種別 + 4 ティアの効果量）
#[derive(Debug, Clone, Copy)]
struct GiftSlotDef {
    stat: GiftStatKind,
    values: [i32; 4],
}

const fn slot(stat: GiftStatKind, values: [i32; 4]) -> GiftSlotDef {
    GiftSlotDef { stat, values }
}

/// ギフトスロットの閾値（全ジョブ共通）。
/// 6 スロット × 4 ティアで、各スロットの 4 ティアの必要 JP は固定。
const GIFT_THRESHOLDS: [[i32; 4]; 6] = [
    [5, 180, 605, 1280],
    [10, 210, 660, 1360],
    [20, 245, 720, 1445],
    [30, 280, 780, 1530],
    [45, 320, 845, 1620],
    [60, 360, 910, 1710],
];

use GiftStatKind::*;

/// 22 ジョブ分のギフト（ステータス系）スロット定義。
/// Job enum の順序に従う: War, Mnk, Whm, Blm, Rdm, Thf, Pld, Drk, Bst, Brd, Rng, Sam,
///                       Nin, Drg, Smn, Blu, Cor, Pup, Dnc, Sch, Geo, Run
const JOB_GIFTS: [[GiftSlotDef; 6]; 22] = [
    // War
    [
        slot(PhysicalDefense, [10, 15, 20, 25]),
        slot(PhysicalAttack, [10, 15, 20, 25]),
        slot(PhysicalEvasion, [5, 8, 10, 13]),
        slot(PhysicalAccuracy, [5, 8, 10, 13]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Mnk
    [
        slot(PhysicalDefense, [5, 8, 10, 13]),
        slot(PhysicalAttack, [8, 12, 16, 20]),
        slot(PhysicalEvasion, [6, 9, 12, 15]),
        slot(PhysicalAccuracy, [6, 9, 11, 15]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Whm
    [
        slot(MagicDefense, [7, 11, 14, 18]),
        slot(MagicAttack, [3, 5, 6, 8]),
        slot(MagicEvasion, [7, 11, 14, 18]),
        slot(MagicAccuracy, [7, 11, 14, 18]),
        slot(PhysicalAccuracy, [2, 3, 4, 5]),
        slot(None, [0, 0, 0, 0]),
    ],
    // Blm
    [
        slot(MagicDefense, [2, 3, 4, 5]),
        slot(MagicAttack, [7, 11, 14, 18]),
        slot(MagicEvasion, [6, 9, 12, 15]),
        slot(MagicAccuracy, [6, 9, 12, 15]),
        slot(None, [0, 0, 0, 0]),
        slot(None, [0, 0, 0, 0]),
    ],
    // Rdm
    [
        slot(MagicDefense, [4, 6, 8, 10]),
        slot(MagicAttack, [4, 6, 8, 10]),
        slot(MagicEvasion, [8, 12, 16, 20]),
        slot(MagicAccuracy, [10, 15, 20, 25]),
        slot(PhysicalAccuracy, [3, 5, 6, 8]),
        slot(None, [0, 0, 0, 0]),
    ],
    // Thf
    [
        slot(PhysicalDefense, [4, 6, 8, 10]),
        slot(PhysicalAttack, [7, 11, 14, 18]),
        slot(PhysicalEvasion, [10, 15, 20, 25]),
        slot(PhysicalAccuracy, [5, 8, 10, 13]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Pld
    [
        slot(PhysicalDefense, [15, 23, 30, 38]),
        slot(PhysicalAttack, [4, 6, 8, 10]),
        slot(PhysicalEvasion, [3, 5, 6, 8]),
        slot(PhysicalAccuracy, [4, 6, 8, 10]),
        slot(MagicEvasion, [6, 9, 12, 15]),
        slot(MagicAccuracy, [6, 9, 12, 15]),
    ],
    // Drk
    [
        slot(PhysicalDefense, [4, 6, 8, 10]),
        slot(PhysicalAttack, [15, 23, 30, 38]),
        slot(PhysicalEvasion, [3, 5, 6, 8]),
        slot(PhysicalAccuracy, [3, 5, 6, 8]),
        slot(MagicEvasion, [6, 9, 12, 15]),
        slot(MagicAccuracy, [6, 9, 12, 15]),
    ],
    // Bst
    [
        slot(PhysicalDefense, [10, 15, 20, 25]),
        slot(PhysicalAttack, [10, 15, 20, 25]),
        slot(PhysicalEvasion, [5, 8, 10, 13]),
        slot(PhysicalAccuracy, [5, 8, 10, 13]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Brd (値は wiki から取得できなかった部分があり保守的な値)
    [
        slot(PhysicalDefense, [5, 8, 10, 13]),
        slot(PhysicalEvasion, [5, 8, 10, 13]),
        slot(PhysicalAccuracy, [5, 8, 10, 13]),
        slot(MagicDefense, [5, 8, 10, 13]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Rng
    [
        slot(PhysicalDefense, [3, 5, 8, 8]),
        slot(PhysicalAttack, [10, 15, 20, 25]),
        slot(PhysicalEvasion, [2, 3, 4, 5]),
        slot(PhysicalAccuracy, [10, 15, 20, 25]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Sam
    [
        slot(PhysicalDefense, [10, 15, 20, 25]),
        slot(PhysicalAttack, [10, 15, 20, 25]),
        slot(PhysicalEvasion, [5, 8, 10, 13]),
        slot(PhysicalAccuracy, [5, 8, 10, 13]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Nin
    [
        slot(PhysicalDefense, [8, 12, 16, 20]),
        slot(None, [0, 0, 0, 0]), // Capacity Point Bonus
        slot(PhysicalAttack, [10, 15, 20, 25]),
        slot(PhysicalEvasion, [9, 14, 18, 23]),
        slot(PhysicalAccuracy, [8, 12, 16, 20]),
        slot(MagicAttack, [4, 6, 8, 10]),
    ],
    // Drg
    [
        slot(PhysicalDefense, [10, 15, 20, 25]),
        slot(PhysicalAttack, [10, 15, 20, 25]),
        slot(PhysicalEvasion, [5, 8, 10, 13]),
        slot(PhysicalAccuracy, [9, 14, 18, 23]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Smn
    [
        slot(MagicDefense, [3, 5, 6, 8]),
        slot(MagicEvasion, [3, 5, 6, 8]),
        slot(PhysicalDefense, [3, 5, 6, 8]),
        slot(PhysicalEvasion, [3, 5, 6, 8]),
        slot(None, [0, 0, 0, 0]), // Avatar/Spirit Attack & Defense
        slot(None, [0, 0, 0, 0]), // Avatar/Spirit Accuracy & Evasion
    ],
    // Blu (wiki のデータが不完全、保守的なデフォルト)
    [
        slot(PhysicalDefense, [10, 15, 20, 25]),
        slot(None, [0, 0, 0, 0]),
        slot(PhysicalAttack, [10, 15, 20, 25]),
        slot(PhysicalEvasion, [5, 8, 10, 13]),
        slot(None, [0, 0, 0, 0]),
        slot(PhysicalAccuracy, [5, 8, 10, 13]),
    ],
    // Cor
    [
        slot(PhysicalDefense, [3, 5, 6, 8]),
        slot(PhysicalAttack, [5, 8, 10, 13]),
        slot(PhysicalEvasion, [3, 5, 6, 8]),
        slot(PhysicalAccuracy, [5, 8, 10, 13]),
        slot(MagicAttack, [2, 3, 4, 5]),
        slot(MagicEvasion, [5, 8, 10, 13]),
    ],
    // Pup
    [
        slot(PhysicalAttack, [6, 9, 12, 15]),
        slot(PhysicalEvasion, [8, 12, 16, 20]),
        slot(PhysicalAccuracy, [7, 11, 14, 18]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
        slot(None, [0, 0, 0, 0]), // Automaton Physical Attack/Defense
    ],
    // Dnc
    [
        slot(PhysicalDefense, [6, 9, 12, 15]),
        slot(PhysicalAttack, [6, 9, 12, 15]),
        slot(PhysicalEvasion, [9, 14, 18, 23]),
        slot(PhysicalAccuracy, [9, 14, 18, 23]),
        slot(MagicEvasion, [5, 8, 10, 13]),
        slot(MagicAccuracy, [5, 8, 10, 13]),
    ],
    // Sch (wiki のデータが不完全、保守的なデフォルト)
    [
        slot(MagicDefense, [3, 5, 6, 8]),
        slot(MagicAttack, [5, 8, 10, 13]),
        slot(MagicEvasion, [6, 9, 12, 15]),
        slot(MagicAccuracy, [6, 9, 12, 15]),
        slot(None, [0, 0, 0, 0]),
        slot(None, [0, 0, 0, 0]),
    ],
    // Geo
    [
        slot(MagicDefense, [4, 6, 8, 10]),
        slot(MagicAttack, [6, 9, 12, 15]),
        slot(MagicEvasion, [7, 11, 14, 18]),
        slot(MagicAccuracy, [7, 11, 14, 18]),
        slot(None, [0, 0, 0, 0]),
        slot(None, [0, 0, 0, 0]),
    ],
    // Run
    [
        slot(PhysicalDefense, [5, 8, 10, 13]),
        slot(PhysicalAttack, [7, 11, 14, 18]),
        slot(PhysicalEvasion, [8, 12, 16, 20]),
        slot(PhysicalAccuracy, [8, 12, 16, 20]),
        slot(MagicDefense, [8, 12, 16, 20]),
        slot(MagicEvasion, [10, 15, 20, 25]),
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

/// 累計 JP 量から得られるギフトのボーナス合計を計算する。
/// ギフトは各ティアのボーナスが累積加算される（例: 7 → 11 → 14 → 18 と解放されると 7+11+14+18=50）。
pub fn calc_gift_bonuses(job: Job, total_jp: i32) -> GiftBonuses {
    let mut bonuses = GiftBonuses::default();
    let job_gifts = &JOB_GIFTS[job as usize];
    for (slot_idx, slot_def) in job_gifts.iter().enumerate() {
        let thresholds = &GIFT_THRESHOLDS[slot_idx];
        let mut slot_total = 0;
        for (tier_idx, &threshold) in thresholds.iter().enumerate() {
            if total_jp >= threshold {
                slot_total += slot_def.values[tier_idx];
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
        // War は直接ステータス効果のあるカテゴリを持たない
        let cats = JobPointCategories::all_maxed();
        let bonuses = calc_jp_category_bonuses(Job::War, &cats);
        assert_eq!(bonuses, GiftBonuses::default());
    }

    #[test]
    fn test_job_points_default() {
        // デフォルト値はすべて 0 ランク
        let jp = JobPoints::default();
        assert_eq!(jp.categories[Job::War].total_jp_spent(), 0);
        assert_eq!(jp.categories[Job::Whm].total_jp_spent(), 0);
    }
}
