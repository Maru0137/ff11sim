use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::job::Job;

/// ジョブポイントのカテゴリ数（各ジョブ10カテゴリ）
pub const JP_CATEGORY_COUNT: usize = 10;
/// ジョブポイントカテゴリの最大ランク
pub const JP_MAX_RANK: i32 = 20;

/// ギフトのステータス種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

// ギフト定義は data/job_gifts.json から `JOB_GIFTS` (data_loader) で読み込む。
use crate::data_loader::JOB_GIFTS;

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
    use GiftStatKind::*;
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
    for slot_def in &JOB_GIFTS[job] {
        let mut slot_total = 0;
        for tier in &slot_def.tiers {
            let threshold = tier[0];
            let value = tier[1];
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
