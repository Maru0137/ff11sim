use crate::data_loader::JOB_STATUS_GRADES;
use crate::status::{Grade, StatusKind};
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray, Enum, Serialize, Deserialize)]
pub enum Job {
    War,
    Mnk,
    Whm,
    Blm,
    Rdm,
    Thf,
    Pld,
    Drk,
    Bst,
    Brd,
    Rng,
    Sam,
    Nin,
    Drg,
    Smn,
    Blu,
    Cor,
    Pup,
    Dnc,
    Sch,
    Geo,
    Run,
}

impl Job {
    pub fn status_grade(&self, kind: StatusKind) -> Option<Grade> {
        JOB_STATUS_GRADES[*self][kind]
    }
}

// ---------------------------------------------------------------------------
// Job Traits (ジョブ特性)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobTrait {
    AttackBonus,
    DefenseBonus,
    MagicDefenseBonus,
    MaxHpBoost,
    MaxHpBoost2,
    MaxMpBoost,
    EvasionBonus,
    AccuracyBonus,
    MagicAttackBonus,
    StoreTp,
    DoubleAttack,
    SkillchainBonus,
}

// Cumulative bonus values per rank for each trait.
// Index 0 = rank 1, index 1 = rank 2, etc.
const ATTACK_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72, 84, 96];
const DEFENSE_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72];
const MAGIC_DEFENSE_BONUS: &[i32] = &[10, 12, 14, 16, 18, 20, 22];
const MAX_HP_BOOST: &[i32] = &[30, 60, 120, 180, 240, 280];
const MAX_HP_BOOST2: &[i32] = &[150, 300, 450];
const MAX_MP_BOOST: &[i32] = &[10, 20, 40, 60, 80, 100];
const EVASION_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72];
const ACCURACY_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72];
const MAGIC_ATTACK_BONUS: &[i32] = &[20, 24, 28, 32, 36, 40];
// Store TP I-V: SAM Lv10/30/50/70/90, cumulative +10/+15/+20/+25/+30
const STORE_TP: &[i32] = &[10, 15, 20, 25, 30];
// Double Attack: WAR Lv25/50/75/85/99, cumulative +10/+12/+14/+16/+18 (%)
// （2015年5月14日 VU で習得タイミングが大幅に増加）
const DOUBLE_ATTACK: &[i32] = &[10, 12, 14, 16, 18];
// Skillchain Bonus: 連携ダメージを増加させるジョブ特性。
// 全ジョブ共通の累積値: rank1=8%, rank2=12%, rank3=16%, rank4=20%, rank5=23%
// (https://wiki.ffo.jp/html/20337.html 参照)
// 装備の "Skillchain Bonus" / "連携ダメージ" 系と合算される。
const SKILLCHAIN_BONUS: &[i32] = &[8, 12, 16, 20, 23];

fn trait_cumulative(trait_kind: JobTrait) -> &'static [i32] {
    match trait_kind {
        JobTrait::AttackBonus => ATTACK_BONUS,
        JobTrait::DefenseBonus => DEFENSE_BONUS,
        JobTrait::MagicDefenseBonus => MAGIC_DEFENSE_BONUS,
        JobTrait::MaxHpBoost => MAX_HP_BOOST,
        JobTrait::MaxHpBoost2 => MAX_HP_BOOST2,
        JobTrait::MaxMpBoost => MAX_MP_BOOST,
        JobTrait::EvasionBonus => EVASION_BONUS,
        JobTrait::AccuracyBonus => ACCURACY_BONUS,
        JobTrait::MagicAttackBonus => MAGIC_ATTACK_BONUS,
        JobTrait::StoreTp => STORE_TP,
        JobTrait::DoubleAttack => DOUBLE_ATTACK,
        JobTrait::SkillchainBonus => SKILLCHAIN_BONUS,
    }
}

// Acquisition levels per job per trait.
// Empty slice means the job doesn't learn the trait.
// Each element is the level at which the next rank is learned.
fn trait_levels(job: Job, trait_kind: JobTrait) -> &'static [i32] {
    match (trait_kind, job) {
        // Attack Bonus: WAR(30,65,91), DRK(10,30,50,70,76,83,91,99), DRG(10,91)
        (JobTrait::AttackBonus, Job::War) => &[30, 65, 91],
        (JobTrait::AttackBonus, Job::Drk) => &[10, 30, 50, 70, 76, 83, 91, 99],
        (JobTrait::AttackBonus, Job::Drg) => &[10, 91],

        // Defense Bonus: PLD(10,30,50,70,76,91), WAR(10,45,86)
        (JobTrait::DefenseBonus, Job::Pld) => &[10, 30, 50, 70, 76, 91],
        (JobTrait::DefenseBonus, Job::War) => &[10, 45, 86],

        // Magic Defense Bonus: WHM(10,30,50,70,81,91), RDM(25,45,96), RUN(10,30,50,70,76,91,99)
        (JobTrait::MagicDefenseBonus, Job::Whm) => &[10, 30, 50, 70, 81, 91],
        (JobTrait::MagicDefenseBonus, Job::Rdm) => &[25, 45, 96],
        (JobTrait::MagicDefenseBonus, Job::Run) => &[10, 30, 50, 70, 76, 91, 99],

        // Max HP Boost: MNK(15,25,35,45,55,65,76), WAR(30,50,70,90), NIN(20,40,60,80,99), RUN(20,40,60,80,99), PLD(45,85)
        (JobTrait::MaxHpBoost, Job::Mnk) => &[15, 25, 35, 45, 55, 65],
        (JobTrait::MaxHpBoost, Job::War) => &[30, 50, 70, 90],
        (JobTrait::MaxHpBoost, Job::Nin) => &[20, 40, 60, 80, 99],
        (JobTrait::MaxHpBoost, Job::Run) => &[20, 40, 60, 80, 99],
        (JobTrait::MaxHpBoost, Job::Pld) => &[45, 85],

        // Max HP Boost II: MNK only (75,85,95)
        (JobTrait::MaxHpBoost2, Job::Mnk) => &[75, 85, 95],

        // Max MP Boost: SMN(10,30,50,70,76,96), SCH(30,88), GEO(30)
        (JobTrait::MaxMpBoost, Job::Smn) => &[10, 30, 50, 70, 76, 96],
        (JobTrait::MaxMpBoost, Job::Sch) => &[30, 88],
        (JobTrait::MaxMpBoost, Job::Geo) => &[30],

        // Evasion Bonus: THF(10,30,50,70,76,88), DNC(15,45,75,86), PUP(20,40,60,76)
        (JobTrait::EvasionBonus, Job::Thf) => &[10, 30, 50, 70, 76, 88],
        (JobTrait::EvasionBonus, Job::Dnc) => &[15, 45, 75, 86],
        (JobTrait::EvasionBonus, Job::Pup) => &[20, 40, 60, 76],

        // Accuracy Bonus: RNG(10,30,50,70,86,96), DRG(30,60,76), DNC(30,60,76), RUN(50,70,90)
        (JobTrait::AccuracyBonus, Job::Rng) => &[10, 30, 50, 70, 86, 96],
        (JobTrait::AccuracyBonus, Job::Drg) => &[30, 60, 76],
        (JobTrait::AccuracyBonus, Job::Dnc) => &[30, 60, 76],
        (JobTrait::AccuracyBonus, Job::Run) => &[50, 70, 90],

        // Magic Attack Bonus: BLM(10,30,50,70,81,91), RDM(20,40,86)
        (JobTrait::MagicAttackBonus, Job::Blm) => &[10, 30, 50, 70, 81, 91],
        (JobTrait::MagicAttackBonus, Job::Rdm) => &[20, 40, 86],

        // Store TP I-V: SAM only (Lv10, 30, 50, 70, 90)
        (JobTrait::StoreTp, Job::Sam) => &[10, 30, 50, 70, 90],

        // Double Attack: WAR (Lv25, 50, 75, 85, 99)
        (JobTrait::DoubleAttack, Job::War) => &[25, 50, 75, 85, 99],

        // Skillchain Bonus: MNK/NIN/SAM/BLU/DNC
        // (https://wiki.ffo.jp/html/20337.html)
        (JobTrait::SkillchainBonus, Job::Mnk) => &[85, 95],
        (JobTrait::SkillchainBonus, Job::Nin) => &[85, 95],
        (JobTrait::SkillchainBonus, Job::Sam) => &[78, 88, 98],
        (JobTrait::SkillchainBonus, Job::Blu) => &[83, 96, 99, 99, 99],
        (JobTrait::SkillchainBonus, Job::Dnc) => &[45, 58, 71, 84, 97],

        _ => &[],
    }
}

/// Calculate the job trait bonus for a given job at a given level.
pub fn job_trait_bonus(job: Job, trait_kind: JobTrait, lv: i32) -> i32 {
    let levels = trait_levels(job, trait_kind);
    let cumulative = trait_cumulative(trait_kind);
    let rank = levels.iter().filter(|&&req_lv| lv >= req_lv).count();
    if rank == 0 {
        0
    } else {
        let idx = std::cmp::min(rank, cumulative.len()) - 1;
        cumulative[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 連携ボーナス (Skillchain Bonus) ジョブ特性の値検証
    /// データソース: https://wiki.ffo.jp/html/20337.html
    /// 累積値: rank1=8, rank2=12, rank3=16, rank4=20, rank5=23
    #[test]
    fn test_skillchain_bonus_trait_sam() {
        // SAM: Lv78/88/98 で rank1/2/3
        assert_eq!(job_trait_bonus(Job::Sam, JobTrait::SkillchainBonus, 77), 0);
        assert_eq!(job_trait_bonus(Job::Sam, JobTrait::SkillchainBonus, 78), 8);
        assert_eq!(job_trait_bonus(Job::Sam, JobTrait::SkillchainBonus, 87), 8);
        assert_eq!(job_trait_bonus(Job::Sam, JobTrait::SkillchainBonus, 88), 12);
        assert_eq!(job_trait_bonus(Job::Sam, JobTrait::SkillchainBonus, 97), 12);
        assert_eq!(job_trait_bonus(Job::Sam, JobTrait::SkillchainBonus, 98), 16);
        assert_eq!(job_trait_bonus(Job::Sam, JobTrait::SkillchainBonus, 99), 16);
    }

    #[test]
    fn test_skillchain_bonus_trait_mnk_nin() {
        // MNK / NIN: Lv85/95 で rank1/2
        for &job in &[Job::Mnk, Job::Nin] {
            assert_eq!(job_trait_bonus(job, JobTrait::SkillchainBonus, 84), 0);
            assert_eq!(job_trait_bonus(job, JobTrait::SkillchainBonus, 85), 8);
            assert_eq!(job_trait_bonus(job, JobTrait::SkillchainBonus, 95), 12);
            assert_eq!(job_trait_bonus(job, JobTrait::SkillchainBonus, 99), 12);
        }
    }

    #[test]
    fn test_skillchain_bonus_trait_blu() {
        // BLU: Lv83/96/99/99/99 で rank1〜5 (Lv99 で 3 ランク同時習得)
        assert_eq!(job_trait_bonus(Job::Blu, JobTrait::SkillchainBonus, 82), 0);
        assert_eq!(job_trait_bonus(Job::Blu, JobTrait::SkillchainBonus, 83), 8);
        assert_eq!(job_trait_bonus(Job::Blu, JobTrait::SkillchainBonus, 95), 8);
        assert_eq!(job_trait_bonus(Job::Blu, JobTrait::SkillchainBonus, 96), 12);
        assert_eq!(job_trait_bonus(Job::Blu, JobTrait::SkillchainBonus, 98), 12);
        assert_eq!(job_trait_bonus(Job::Blu, JobTrait::SkillchainBonus, 99), 23);
    }

    #[test]
    fn test_skillchain_bonus_trait_dnc() {
        // DNC: Lv45/58/71/84/97 で rank1〜5
        assert_eq!(job_trait_bonus(Job::Dnc, JobTrait::SkillchainBonus, 44), 0);
        assert_eq!(job_trait_bonus(Job::Dnc, JobTrait::SkillchainBonus, 45), 8);
        assert_eq!(job_trait_bonus(Job::Dnc, JobTrait::SkillchainBonus, 58), 12);
        assert_eq!(job_trait_bonus(Job::Dnc, JobTrait::SkillchainBonus, 71), 16);
        assert_eq!(job_trait_bonus(Job::Dnc, JobTrait::SkillchainBonus, 84), 20);
        assert_eq!(job_trait_bonus(Job::Dnc, JobTrait::SkillchainBonus, 97), 23);
        assert_eq!(job_trait_bonus(Job::Dnc, JobTrait::SkillchainBonus, 99), 23);
    }

    #[test]
    fn test_skillchain_bonus_trait_other_jobs() {
        // 連携ボーナスを習得しないジョブは常に 0
        for &job in &[Job::War, Job::Whm, Job::Blm, Job::Rdm, Job::Thf, Job::Pld,
                      Job::Drk, Job::Bst, Job::Brd, Job::Rng, Job::Drg, Job::Smn,
                      Job::Cor, Job::Pup, Job::Sch, Job::Geo, Job::Run] {
            assert_eq!(
                job_trait_bonus(job, JobTrait::SkillchainBonus, 99),
                0,
                "{:?} should not have Skillchain Bonus trait",
                job,
            );
        }
    }
}
