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

// Job status grades: [HP, MP, STR, DEX, VIT, AGI, INT, MND, CHR]
// None means the job doesn't contribute to that stat (e.g., no MP for melee jobs)
// Data from: https://wiki.ffo.jp/html/316.html
const JOB_STATUS_GRADES: [[Option<Grade>; StatusKind::COUNT]; Job::COUNT] = [
    // War: HP=B, MP=-, STR=A, DEX=C, VIT=D, AGI=C, INT=F, MND=F, CHR=E
    [Some(Grade::B), None, Some(Grade::A), Some(Grade::C), Some(Grade::D), Some(Grade::C), Some(Grade::F), Some(Grade::F), Some(Grade::E)],
    // Mnk: HP=A, MP=-, STR=C, DEX=B, VIT=A, AGI=F, INT=G, MND=D, CHR=E
    [Some(Grade::A), None, Some(Grade::C), Some(Grade::B), Some(Grade::A), Some(Grade::F), Some(Grade::G), Some(Grade::D), Some(Grade::E)],
    // Whm: HP=E, MP=C, STR=D, DEX=F, VIT=D, AGI=E, INT=E, MND=A, CHR=C
    [Some(Grade::E), Some(Grade::C), Some(Grade::D), Some(Grade::F), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::A), Some(Grade::C)],
    // Blm: HP=F, MP=B, STR=F, DEX=C, VIT=F, AGI=C, INT=A, MND=E, CHR=D
    [Some(Grade::F), Some(Grade::B), Some(Grade::F), Some(Grade::C), Some(Grade::F), Some(Grade::C), Some(Grade::A), Some(Grade::E), Some(Grade::D)],
    // Rdm: HP=D, MP=D, STR=D, DEX=D, VIT=E, AGI=E, INT=C, MND=C, CHR=D
    [Some(Grade::D), Some(Grade::D), Some(Grade::D), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::C), Some(Grade::C), Some(Grade::D)],
    // Thf: HP=D, MP=-, STR=D, DEX=A, VIT=D, AGI=B, INT=C, MND=G, CHR=G
    [Some(Grade::D), None, Some(Grade::D), Some(Grade::A), Some(Grade::D), Some(Grade::B), Some(Grade::C), Some(Grade::G), Some(Grade::G)],
    // Pld: HP=C, MP=F, STR=B, DEX=E, VIT=A, AGI=G, INT=G, MND=C, CHR=C
    [Some(Grade::C), Some(Grade::F), Some(Grade::B), Some(Grade::E), Some(Grade::A), Some(Grade::G), Some(Grade::G), Some(Grade::C), Some(Grade::C)],
    // Drk: HP=C, MP=F, STR=A, DEX=C, VIT=C, AGI=D, INT=C, MND=G, CHR=G
    [Some(Grade::C), Some(Grade::F), Some(Grade::A), Some(Grade::C), Some(Grade::C), Some(Grade::D), Some(Grade::C), Some(Grade::G), Some(Grade::G)],
    // Bst: HP=C, MP=-, STR=D, DEX=C, VIT=D, AGI=F, INT=E, MND=E, CHR=A
    [Some(Grade::C), None, Some(Grade::D), Some(Grade::C), Some(Grade::D), Some(Grade::F), Some(Grade::E), Some(Grade::E), Some(Grade::A)],
    // Brd: HP=D, MP=-, STR=D, DEX=D, VIT=D, AGI=F, INT=D, MND=D, CHR=B
    [Some(Grade::D), None, Some(Grade::D), Some(Grade::D), Some(Grade::D), Some(Grade::F), Some(Grade::D), Some(Grade::D), Some(Grade::B)],
    // Rng: HP=E, MP=-, STR=E, DEX=D, VIT=D, AGI=A, INT=E, MND=D, CHR=E
    [Some(Grade::E), None, Some(Grade::E), Some(Grade::D), Some(Grade::D), Some(Grade::A), Some(Grade::E), Some(Grade::D), Some(Grade::E)],
    // Sam: HP=B, MP=-, STR=C, DEX=C, VIT=C, AGI=D, INT=E, MND=E, CHR=D
    [Some(Grade::B), None, Some(Grade::C), Some(Grade::C), Some(Grade::C), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::D)],
    // Nin: HP=D, MP=-, STR=C, DEX=B, VIT=C, AGI=B, INT=D, MND=G, CHR=F
    [Some(Grade::D), None, Some(Grade::C), Some(Grade::B), Some(Grade::C), Some(Grade::B), Some(Grade::D), Some(Grade::G), Some(Grade::F)],
    // Drg: HP=B, MP=-, STR=B, DEX=D, VIT=C, AGI=D, INT=F, MND=E, CHR=C
    [Some(Grade::B), None, Some(Grade::B), Some(Grade::D), Some(Grade::C), Some(Grade::D), Some(Grade::F), Some(Grade::E), Some(Grade::C)],
    // Smn: HP=G, MP=A, STR=F, DEX=E, VIT=F, AGI=D, INT=B, MND=B, CHR=B
    [Some(Grade::G), Some(Grade::A), Some(Grade::F), Some(Grade::E), Some(Grade::F), Some(Grade::D), Some(Grade::B), Some(Grade::B), Some(Grade::B)],
    // Blu: HP=D, MP=D, STR=E, DEX=E, VIT=E, AGI=E, INT=E, MND=E, CHR=E
    [Some(Grade::D), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E)],
    // Cor: HP=D, MP=-, STR=E, DEX=C, VIT=E, AGI=B, INT=C, MND=E, CHR=E
    [Some(Grade::D), None, Some(Grade::E), Some(Grade::C), Some(Grade::E), Some(Grade::B), Some(Grade::C), Some(Grade::E), Some(Grade::E)],
    // Pup: HP=D, MP=-, STR=E, DEX=B, VIT=D, AGI=C, INT=E, MND=F, CHR=C
    [Some(Grade::D), None, Some(Grade::E), Some(Grade::B), Some(Grade::D), Some(Grade::C), Some(Grade::E), Some(Grade::F), Some(Grade::C)],
    // Dnc: HP=D, MP=-, STR=D, DEX=C, VIT=E, AGI=B, INT=F, MND=F, CHR=B
    [Some(Grade::D), None, Some(Grade::D), Some(Grade::C), Some(Grade::E), Some(Grade::B), Some(Grade::F), Some(Grade::F), Some(Grade::B)],
    // Sch: HP=E, MP=D, STR=F, DEX=D, VIT=E, AGI=D, INT=B, MND=D, CHR=C
    [Some(Grade::E), Some(Grade::D), Some(Grade::F), Some(Grade::D), Some(Grade::E), Some(Grade::D), Some(Grade::B), Some(Grade::D), Some(Grade::C)],
    // Geo: HP=D, MP=C, STR=F, DEX=D, VIT=D, AGI=E, INT=B, MND=B, CHR=E
    [Some(Grade::D), Some(Grade::C), Some(Grade::F), Some(Grade::D), Some(Grade::D), Some(Grade::E), Some(Grade::B), Some(Grade::B), Some(Grade::E)],
    // Run: HP=B, MP=F, STR=C, DEX=D, VIT=E, AGI=B, INT=D, MND=D, CHR=F
    [Some(Grade::B), Some(Grade::F), Some(Grade::C), Some(Grade::D), Some(Grade::E), Some(Grade::B), Some(Grade::D), Some(Grade::D), Some(Grade::F)],
];

impl Job {
    pub fn status_grade(&self, kind: StatusKind) -> Option<Grade> {
        JOB_STATUS_GRADES[*self as usize][kind as usize]
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
