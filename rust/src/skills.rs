use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, VariantArray};

use crate::character_profile::JobLevel;
use crate::data_loader::{JOB_SKILL_RANKS, SKILL_CAP_CONTROL_POINTS};
use crate::job::Job;

// ---------------------------------------------------------------------------
// SkillKind / SkillRank
// ---------------------------------------------------------------------------

/// スキル種別（戦闘 19 + 魔法 14 = 33 種類）
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray, Enum, Serialize,
    Deserialize,
)]
pub enum SkillKind {
    // 武器スキル (15)
    HandToHand,
    Dagger,
    Sword,
    GreatSword,
    Axe,
    GreatAxe,
    Scythe,
    Polearm,
    Katana,
    GreatKatana,
    Club,
    Staff,
    Archery,
    Marksmanship,
    Throwing,
    // 防御スキル (4)
    Guarding,
    Evasion,
    Shield,
    Parrying,
    // 魔法スキル (14)
    Divine,
    Healing,
    Enhancing,
    Enfeebling,
    Elemental,
    Dark,
    Summoning,
    Ninjutsu,
    Singing,
    StringInstrument,
    WindInstrument,
    BlueMagic,
    Geomancy,
    Handbell,
}

impl SkillKind {
    /// スキルキー文字列（JS 側と共通のキー名）
    pub fn key(self) -> &'static str {
        match self {
            SkillKind::HandToHand => "HandToHand",
            SkillKind::Dagger => "Dagger",
            SkillKind::Sword => "Sword",
            SkillKind::GreatSword => "GreatSword",
            SkillKind::Axe => "Axe",
            SkillKind::GreatAxe => "GreatAxe",
            SkillKind::Scythe => "Scythe",
            SkillKind::Polearm => "Polearm",
            SkillKind::Katana => "Katana",
            SkillKind::GreatKatana => "GreatKatana",
            SkillKind::Club => "Club",
            SkillKind::Staff => "Staff",
            SkillKind::Archery => "Archery",
            SkillKind::Marksmanship => "Marksmanship",
            SkillKind::Throwing => "Throwing",
            SkillKind::Guarding => "Guarding",
            SkillKind::Evasion => "Evasion",
            SkillKind::Shield => "Shield",
            SkillKind::Parrying => "Parrying",
            SkillKind::Divine => "Divine",
            SkillKind::Healing => "Healing",
            SkillKind::Enhancing => "Enhancing",
            SkillKind::Enfeebling => "Enfeebling",
            SkillKind::Elemental => "Elemental",
            SkillKind::Dark => "Dark",
            SkillKind::Summoning => "Summoning",
            SkillKind::Ninjutsu => "Ninjutsu",
            SkillKind::Singing => "Singing",
            SkillKind::StringInstrument => "StringInstrument",
            SkillKind::WindInstrument => "WindInstrument",
            SkillKind::BlueMagic => "BlueMagic",
            SkillKind::Geomancy => "Geomancy",
            SkillKind::Handbell => "Handbell",
        }
    }

    /// 戦闘スキル（武器 15 + 防御 4）かどうか
    pub fn is_combat(self) -> bool {
        matches!(
            self,
            SkillKind::HandToHand
                | SkillKind::Dagger
                | SkillKind::Sword
                | SkillKind::GreatSword
                | SkillKind::Axe
                | SkillKind::GreatAxe
                | SkillKind::Scythe
                | SkillKind::Polearm
                | SkillKind::Katana
                | SkillKind::GreatKatana
                | SkillKind::Club
                | SkillKind::Staff
                | SkillKind::Archery
                | SkillKind::Marksmanship
                | SkillKind::Throwing
                | SkillKind::Guarding
                | SkillKind::Evasion
                | SkillKind::Shield
                | SkillKind::Parrying
        )
    }

    /// 魔法スキルかどうか
    pub fn is_magic(self) -> bool {
        !self.is_combat()
    }

    /// メリットポイントによるスキルキャップボーナス (+2/rank)
    pub fn merit_bonus(self, merit: &crate::status::MeritPoints, skill_key: &str) -> i32 {
        let rank = if self.is_combat() {
            merit.combat_skill_merits.get(skill_key).copied().unwrap_or(0)
        } else {
            merit.magic_skill_merits.get(skill_key).copied().unwrap_or(0)
        };
        rank * 2
    }
}

/// スキルランク。A+ が最も高い。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray, Enum, Serialize,
    Deserialize,
)]
pub enum SkillRank {
    #[serde(rename = "A+")]
    APlus,
    A,
    #[serde(rename = "B+")]
    BPlus,
    B,
    #[serde(rename = "B-")]
    BMinus,
    #[serde(rename = "C+")]
    CPlus,
    C,
    #[serde(rename = "C-")]
    CMinus,
    D,
    E,
    F,
    G,
}

// ---------------------------------------------------------------------------
// スキルキャップの計算
// ---------------------------------------------------------------------------

/// 指定ランク・レベルにおけるスキルキャップ値を計算する。
/// 制御点間を線形補間し、マスターレベル分を加算する。
/// データは `SKILL_CAP_CONTROL_POINTS` (data/skill_cap_control_points.json) を参照。
pub fn skill_cap(rank: SkillRank, lv: i32, master_lv: i32) -> i32 {
    if lv <= 0 {
        return 0;
    }
    let lv = lv.min(99);
    let table = &*SKILL_CAP_CONTROL_POINTS;
    let levels = &table.control_levels;
    let values = &table.ranks[rank];

    // 制御点間で線形補間
    let base = if lv <= levels[0] {
        values[0]
    } else if lv >= levels[3] {
        values[3]
    } else {
        // 2 区間のうちどれに該当するかを探す
        let mut result = values[3];
        for i in 0..3 {
            let x1 = levels[i];
            let x2 = levels[i + 1];
            if lv >= x1 && lv <= x2 {
                let y1 = values[i] as f32;
                let y2 = values[i + 1] as f32;
                let t = (lv - x1) as f32 / (x2 - x1) as f32;
                result = (y1 + (y2 - y1) * t).round() as i32;
                break;
            }
        }
        result
    };

    base + master_lv.max(0)
}

// ---------------------------------------------------------------------------
// ジョブ × スキルのランク行列
// ---------------------------------------------------------------------------
// データは `JOB_SKILL_RANKS` (data/job_skill_ranks.json) から読み込まれる。

/// 指定ジョブが指定スキルに持つランクを返す。None は未習得。
pub fn job_skill_rank(job: Job, skill: SkillKind) -> Option<SkillRank> {
    JOB_SKILL_RANKS[job][skill]
}

/// ジョブ・レベル・マスターレベルにおけるスキルキャップ値
pub fn job_skill_cap(job: Job, skill: SkillKind, lv: i32, master_lv: i32) -> i32 {
    match job_skill_rank(job, skill) {
        Some(rank) => skill_cap(rank, lv, master_lv),
        None => 0,
    }
}

/// アイテム JSON の `skill` フィールド値から SkillKind へ変換
pub fn weapon_skill_from_item_id(skill_id: i32) -> Option<SkillKind> {
    match skill_id {
        1 => Some(SkillKind::HandToHand),
        2 => Some(SkillKind::Dagger),
        3 => Some(SkillKind::Sword),
        4 => Some(SkillKind::GreatSword),
        5 => Some(SkillKind::Axe),
        6 => Some(SkillKind::GreatAxe),
        7 => Some(SkillKind::Scythe),
        8 => Some(SkillKind::Polearm),
        9 => Some(SkillKind::Katana),
        10 => Some(SkillKind::GreatKatana),
        11 => Some(SkillKind::Club),
        12 => Some(SkillKind::Staff),
        25 => Some(SkillKind::Archery),
        26 => Some(SkillKind::Marksmanship),
        27 => Some(SkillKind::Throwing),
        41 => Some(SkillKind::StringInstrument),
        42 => Some(SkillKind::WindInstrument),
        45 => Some(SkillKind::Handbell),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// CharacterSkills
// ---------------------------------------------------------------------------

/// キャラクターのスキル値。全ジョブで共通の 1 組を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSkills {
    #[serde(default)]
    pub values: EnumMap<SkillKind, i32>,
}

impl Default for CharacterSkills {
    fn default() -> Self {
        Self {
            values: EnumMap::default(),
        }
    }
}

impl CharacterSkills {
    pub fn get(&self, skill: SkillKind) -> i32 {
        self.values[skill]
    }

    pub fn set(&mut self, skill: SkillKind, value: i32) {
        self.values[skill] = value;
    }
}

/// 全ジョブのレベル情報から、あるスキルのデフォルト値（キャップの最大 + メリットボーナス）を計算する。
/// レベルが 0 のジョブは無視する。
pub fn default_skill_value(
    skill: SkillKind,
    job_levels: &EnumMap<Job, JobLevel>,
    merit: &crate::status::MeritPoints,
) -> i32 {
    let mut max = 0;
    for (job, jl) in job_levels.iter() {
        if jl.level == 0 {
            continue;
        }
        let cap = job_skill_cap(job, skill, jl.level, jl.master_lv);
        if cap > max {
            max = cap;
        }
    }
    if max > 0 {
        max += skill.merit_bonus(merit, skill.key());
    }
    max
}

/// 全スキルについてのデフォルト値を算出する。
pub fn default_skills(
    job_levels: &EnumMap<Job, JobLevel>,
    merit: &crate::status::MeritPoints,
) -> CharacterSkills {
    let mut skills = CharacterSkills::default();
    for skill in <SkillKind as VariantArray>::VARIANTS {
        skills.values[*skill] = default_skill_value(*skill, job_levels, merit);
    }
    skills
}

// ---------------------------------------------------------------------------
// Effective Skill (char value capped by job)
// ---------------------------------------------------------------------------

/// メイン/サポートジョブの組み合わせにおけるスキルの有効値を計算する。
/// キャラクターのスキル値とジョブ経由のキャップの最大値のうち、低い方を返す。
/// キャップはメインジョブ（+ ML）とサポートジョブ（support_lv で上限）のうち高い方 + メリットボーナス。
pub fn effective_skill(
    skill: SkillKind,
    main_job: Job,
    main_lv: i32,
    master_lv: i32,
    support_job: Option<Job>,
    support_lv: Option<i32>,
    char_value: i32,
    merit: &crate::status::MeritPoints,
) -> i32 {
    let main_cap = job_skill_cap(main_job, skill, main_lv, master_lv);
    let sup_cap = match (support_job, support_lv) {
        (Some(sj), Some(sl)) => job_skill_cap(sj, skill, sl, 0),
        _ => 0,
    };
    let mut max_cap = main_cap.max(sup_cap);
    if max_cap > 0 {
        max_cap += skill.merit_bonus(merit, skill.key());
    }
    char_value.min(max_cap)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_cap_control_points() {
        // A+: Lv1=6, Lv50=153, Lv75=276, Lv99=424
        assert_eq!(skill_cap(SkillRank::APlus, 1, 0), 6);
        assert_eq!(skill_cap(SkillRank::APlus, 50, 0), 153);
        assert_eq!(skill_cap(SkillRank::APlus, 75, 0), 276);
        assert_eq!(skill_cap(SkillRank::APlus, 99, 0), 424);
    }

    #[test]
    fn test_skill_cap_all_ranks_lv99() {
        assert_eq!(skill_cap(SkillRank::APlus, 99, 0), 424);
        assert_eq!(skill_cap(SkillRank::A, 99, 0), 417);
        assert_eq!(skill_cap(SkillRank::BPlus, 99, 0), 404);
        assert_eq!(skill_cap(SkillRank::B, 99, 0), 398);
        assert_eq!(skill_cap(SkillRank::BMinus, 99, 0), 388);
        assert_eq!(skill_cap(SkillRank::CPlus, 99, 0), 378);
        assert_eq!(skill_cap(SkillRank::C, 99, 0), 373);
        assert_eq!(skill_cap(SkillRank::CMinus, 99, 0), 368);
        assert_eq!(skill_cap(SkillRank::D, 99, 0), 334);
        assert_eq!(skill_cap(SkillRank::E, 99, 0), 300);
        assert_eq!(skill_cap(SkillRank::F, 99, 0), 265);
        assert_eq!(skill_cap(SkillRank::G, 99, 0), 228);
    }

    #[test]
    fn test_skill_cap_master_level() {
        // Master Level は加算される
        assert_eq!(skill_cap(SkillRank::APlus, 99, 50), 424 + 50);
        assert_eq!(skill_cap(SkillRank::G, 99, 30), 228 + 30);
    }

    #[test]
    fn test_skill_cap_interpolation() {
        // Lv25（Lv1 と Lv50 の間）: 6 + (50-1)/(50-1) * (153 - 6) ... 実際は線形補間で中間値
        // (25-1)/(50-1) = 24/49 ≈ 0.49
        // 6 + 0.49 * 147 ≈ 6 + 72.05 ≈ 78 (round)
        let cap = skill_cap(SkillRank::APlus, 25, 0);
        assert!(cap > 6 && cap < 153);
    }

    #[test]
    fn test_skill_cap_lv0() {
        assert_eq!(skill_cap(SkillRank::APlus, 0, 0), 0);
    }

    #[test]
    fn test_job_skill_rank_war() {
        // 戦士の両手斧は A+
        assert_eq!(
            job_skill_rank(Job::War, SkillKind::GreatAxe),
            Some(SkillRank::APlus)
        );
        // 戦士は魔法スキル無し
        assert_eq!(job_skill_rank(Job::War, SkillKind::Healing), None);
    }

    #[test]
    fn test_job_skill_cap() {
        // War 両手斧 @ Lv99 ML50 = 424 + 50 = 474
        assert_eq!(
            job_skill_cap(Job::War, SkillKind::GreatAxe, 99, 50),
            474
        );
        // War 魔法スキル = 0
        assert_eq!(job_skill_cap(Job::War, SkillKind::Healing, 99, 50), 0);
    }

    #[test]
    fn test_effective_skill_capped_by_cap() {
        let merit = crate::status::MeritPoints::default();
        // キャラクターがスキル値 500 を持っていても、War の両手斧 Lv99 ML0 の上限 424 で制限される
        let v = effective_skill(
            SkillKind::GreatAxe,
            Job::War,
            99,
            0,
            None,
            None,
            500,
            &merit,
        );
        assert_eq!(v, 424);
    }

    #[test]
    fn test_effective_skill_capped_by_char_value() {
        let merit = crate::status::MeritPoints::default();
        // キャラクターのスキル値 200 < cap 424 のとき、200 が使われる
        let v = effective_skill(
            SkillKind::GreatAxe,
            Job::War,
            99,
            0,
            None,
            None,
            200,
            &merit,
        );
        assert_eq!(v, 200);
    }

    #[test]
    fn test_effective_skill_support_job_higher() {
        let merit = crate::status::MeritPoints::default();
        // War/Nin@49: Nin は片手刀 A+ (@Lv49で cap ≈ 150ish), War は片手刀なし
        // よって Nin のキャップが使われる
        let v = effective_skill(
            SkillKind::Katana,
            Job::War,
            99,
            0,
            Some(Job::Nin),
            Some(49),
            500,
            &merit,
        );
        // Nin の片手刀 A+ @ Lv49, 線形補間で 1-50 の中
        // APlus: [6, 153, 276, 424] at [1, 50, 75, 99]
        // Lv49: 6 + (49-1)/(50-1) * (153-6) = 6 + 48/49 * 147 ≈ 6 + 144 = 150
        assert!(v >= 140 && v <= 160, "v = {}", v);
    }

    #[test]
    fn test_effective_skill_no_main_rank() {
        let merit = crate::status::MeritPoints::default();
        // War は魔法スキル持たないので 0
        let v = effective_skill(
            SkillKind::Healing,
            Job::War,
            99,
            50,
            None,
            None,
            300,
            &merit,
        );
        assert_eq!(v, 0);
    }

    #[test]
    fn test_default_skill_value_single_job() {
        let merit = crate::status::MeritPoints::default();
        let mut jl: EnumMap<Job, JobLevel> = EnumMap::default();
        jl[Job::War] = JobLevel { level: 99, master_lv: 50 };
        // War の両手斧 A+ @ 99 ML50 = 474
        assert_eq!(default_skill_value(SkillKind::GreatAxe, &jl, &merit), 474);
        // War は魔法なし
        assert_eq!(default_skill_value(SkillKind::Healing, &jl, &merit), 0);
    }

    #[test]
    fn test_default_skill_value_multiple_jobs() {
        let merit = crate::status::MeritPoints::default();
        let mut jl: EnumMap<Job, JobLevel> = EnumMap::default();
        jl[Job::War] = JobLevel { level: 99, master_lv: 0 }; // GreatAxe A+ = 424
        jl[Job::Drk] = JobLevel { level: 50, master_lv: 0 }; // GreatAxe B- @ 50 = 126
        // War のほうが大きい
        assert_eq!(default_skill_value(SkillKind::GreatAxe, &jl, &merit), 424);
    }

    #[test]
    fn test_default_skill_value_with_merit_bonus() {
        let mut merit = crate::status::MeritPoints::default();
        merit.combat_skill_merits.insert("GreatAxe".to_string(), 8); // +16
        merit.magic_skill_merits.insert("Enfeebling".to_string(), 5); // +10
        let mut jl: EnumMap<Job, JobLevel> = EnumMap::default();
        jl[Job::War] = JobLevel { level: 99, master_lv: 0 };
        // War の両手斧 A+ @ 99 = 424 + merit 16 = 440
        assert_eq!(default_skill_value(SkillKind::GreatAxe, &jl, &merit), 440);
        // War は魔法なし → 0（メリットボーナスも加算されない）
        assert_eq!(default_skill_value(SkillKind::Healing, &jl, &merit), 0);
        // メリット未設定の戦闘スキルはボーナスなし（War の片手剣 B @ 99 = 398）
        assert_eq!(default_skill_value(SkillKind::Sword, &jl, &merit), 398);
    }

    #[test]
    fn test_weapon_skill_from_item_id() {
        assert_eq!(
            weapon_skill_from_item_id(1),
            Some(SkillKind::HandToHand)
        );
        assert_eq!(weapon_skill_from_item_id(6), Some(SkillKind::GreatAxe));
        assert_eq!(
            weapon_skill_from_item_id(26),
            Some(SkillKind::Marksmanship)
        );
        assert_eq!(weapon_skill_from_item_id(99), None);
    }

    #[test]
    fn test_character_skills_default() {
        let skills = CharacterSkills::default();
        for skill in <SkillKind as VariantArray>::VARIANTS {
            assert_eq!(skills.get(*skill), 0);
        }
    }
}
