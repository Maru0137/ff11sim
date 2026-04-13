use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, VariantArray};

use crate::character_profile::JobLevel;
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

/// スキルランク。A+ が最も高い。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillRank {
    APlus,
    A,
    BPlus,
    B,
    BMinus,
    CPlus,
    C,
    CMinus,
    D,
    E,
    F,
    G,
}

// ---------------------------------------------------------------------------
// スキルキャップの計算
// ---------------------------------------------------------------------------

/// ランクごとの Lv1/Lv50/Lv75/Lv99 制御点
/// wiki.ffo.jp/html/2570.html の計算式および既知のキャップ値を参照。
const SKILL_CAP_CONTROL_POINTS: [[i32; 4]; 12] = [
    // A+
    [6, 153, 276, 424],
    // A
    [5, 147, 269, 417],
    // B+
    [5, 142, 256, 404],
    // B
    [4, 136, 250, 398],
    // B-
    [4, 126, 240, 388],
    // C+
    [4, 116, 230, 378],
    // C
    [4, 101, 225, 373],
    // C-
    [3, 90, 220, 368],
    // D
    [3, 78, 210, 334],
    // E
    [2, 65, 200, 300],
    // F
    [2, 55, 189, 265],
    // G
    [2, 45, 171, 228],
];

const CONTROL_LEVELS: [i32; 4] = [1, 50, 75, 99];

fn rank_index(rank: SkillRank) -> usize {
    match rank {
        SkillRank::APlus => 0,
        SkillRank::A => 1,
        SkillRank::BPlus => 2,
        SkillRank::B => 3,
        SkillRank::BMinus => 4,
        SkillRank::CPlus => 5,
        SkillRank::C => 6,
        SkillRank::CMinus => 7,
        SkillRank::D => 8,
        SkillRank::E => 9,
        SkillRank::F => 10,
        SkillRank::G => 11,
    }
}

/// 指定ランク・レベルにおけるスキルキャップ値を計算する。
/// 制御点間を線形補間し、マスターレベル分を加算する。
pub fn skill_cap(rank: SkillRank, lv: i32, master_lv: i32) -> i32 {
    if lv <= 0 {
        return 0;
    }
    let lv = lv.min(99);
    let values = &SKILL_CAP_CONTROL_POINTS[rank_index(rank)];

    // 制御点間で線形補間
    let base = if lv <= CONTROL_LEVELS[0] {
        values[0]
    } else if lv >= CONTROL_LEVELS[3] {
        values[3]
    } else {
        // 2 区間のうちどれに該当するかを探す
        let mut result = values[3];
        for i in 0..3 {
            let x1 = CONTROL_LEVELS[i];
            let x2 = CONTROL_LEVELS[i + 1];
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

use SkillRank::*;

type JobRanks = [Option<SkillRank>; SkillKind::COUNT];

/// ジョブごとのスキルランク。Job enum の順序に従う。
/// スキル順序は SkillKind の宣言順 (HandToHand, Dagger, ..., Handbell)
const JOB_SKILL_RANKS: [JobRanks; Job::COUNT] = [
    // War
    [
        Some(D),      // HandToHand
        Some(BMinus), // Dagger
        Some(B),      // Sword
        Some(BPlus),  // GreatSword
        Some(A),      // Axe
        Some(APlus),  // GreatAxe
        Some(BPlus),  // Scythe
        Some(BMinus), // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(BMinus), // Club
        Some(B),      // Staff
        Some(D),      // Archery
        Some(D),      // Marksmanship
        Some(D),      // Throwing
        None,         // Guarding
        Some(C),      // Evasion
        Some(CPlus),  // Shield
        Some(CMinus), // Parrying
        None,         // Divine
        None,         // Healing
        None,         // Enhancing
        None,         // Enfeebling
        None,         // Elemental
        None,         // Dark
        None,         // Summoning
        None,         // Ninjutsu
        None,         // Singing
        None,         // StringInstrument
        None,         // WindInstrument
        None,         // BlueMagic
        None,         // Geomancy
        None,         // Handbell
    ],
    // Mnk
    [
        Some(APlus),  // HandToHand
        None,         // Dagger
        None,         // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(CPlus),  // Club
        Some(B),      // Staff
        None,         // Archery
        None,         // Marksmanship
        Some(E),      // Throwing
        Some(A),      // Guarding
        Some(BPlus),  // Evasion
        None,         // Shield
        Some(E),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Whm
    [
        None,         // HandToHand
        None,         // Dagger
        None,         // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(BPlus),  // Club
        Some(CPlus),  // Staff
        None,         // Archery
        None,         // Marksmanship
        Some(E),      // Throwing
        None,         // Guarding
        Some(E),      // Evasion
        Some(D),      // Shield
        None,         // Parrying
        Some(A),      // Divine
        Some(APlus),  // Healing
        Some(CPlus),  // Enhancing
        Some(C),      // Enfeebling
        None, None, None, None, None, None, None, None, None, None,
    ],
    // Blm
    [
        None,         // HandToHand
        Some(D),      // Dagger
        None,         // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        Some(E),      // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(CPlus),  // Club
        Some(BMinus), // Staff
        None,         // Archery
        None,         // Marksmanship
        Some(D),      // Throwing
        None,         // Guarding
        Some(E),      // Evasion
        None,         // Shield
        Some(E),      // Parrying
        None,         // Divine
        None,         // Healing
        Some(E),      // Enhancing
        Some(CPlus),  // Enfeebling
        Some(APlus),  // Elemental
        Some(A),      // Dark
        None, None, None, None, None, None, None, None,
    ],
    // Rdm
    [
        None,         // HandToHand
        Some(B),      // Dagger
        Some(B),      // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(D),      // Club
        None,         // Staff
        Some(D),      // Archery
        None,         // Marksmanship
        Some(F),      // Throwing
        None,         // Guarding
        Some(D),      // Evasion
        Some(F),      // Shield
        Some(E),      // Parrying
        Some(E),      // Divine
        Some(CMinus), // Healing
        Some(BPlus),  // Enhancing
        Some(APlus),  // Enfeebling
        Some(CPlus),  // Elemental
        Some(E),      // Dark
        None, None, None, None, None, None, None, None,
    ],
    // Thf
    [
        Some(E),      // HandToHand
        Some(APlus),  // Dagger
        Some(D),      // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(E),      // Club
        None,         // Staff
        Some(CMinus), // Archery
        Some(CPlus),  // Marksmanship
        Some(D),      // Throwing
        None,         // Guarding
        Some(APlus),  // Evasion
        Some(APlus),  // Shield
        Some(A),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Pld
    [
        None,         // HandToHand
        Some(CMinus), // Dagger
        Some(APlus),  // Sword
        Some(B),      // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        Some(E),      // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(A),      // Club
        Some(A),      // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(C),      // Evasion
        Some(APlus),  // Shield
        Some(C),      // Parrying
        Some(BPlus),  // Divine
        Some(C),      // Healing
        Some(D),      // Enhancing
        None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Drk
    [
        None,         // HandToHand
        Some(C),      // Dagger
        Some(BMinus), // Sword
        Some(A),      // GreatSword
        Some(BMinus), // Axe
        Some(BMinus), // GreatAxe
        Some(APlus),  // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(CMinus), // Club
        None,         // Staff
        None,         // Archery
        Some(E),      // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(C),      // Evasion
        None,         // Shield
        Some(E),      // Parrying
        None,         // Divine
        None,         // Healing
        None,         // Enhancing
        Some(C),      // Enfeebling
        Some(BPlus),  // Elemental
        Some(A),      // Dark
        None, None, None, None, None, None, None, None,
    ],
    // Bst
    [
        None,         // HandToHand
        Some(CPlus),  // Dagger
        Some(E),      // Sword
        None,         // GreatSword
        Some(APlus),  // Axe
        None,         // GreatAxe
        Some(BMinus), // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(D),      // Club
        None,         // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(C),      // Evasion
        Some(E),      // Shield
        Some(C),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Brd
    [
        None,         // HandToHand
        Some(BMinus), // Dagger
        Some(CMinus), // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(D),      // Club
        Some(CPlus),  // Staff
        None,         // Archery
        None,         // Marksmanship
        Some(E),      // Throwing
        None,         // Guarding
        Some(D),      // Evasion
        None,         // Shield
        Some(E),      // Parrying
        None,         // Divine
        None,         // Healing
        None,         // Enhancing
        None,         // Enfeebling
        None,         // Elemental
        None,         // Dark
        None,         // Summoning
        None,         // Ninjutsu
        Some(C),      // Singing
        Some(C),      // StringInstrument
        Some(C),      // WindInstrument
        None, None, None,
    ],
    // Rng
    [
        None,         // HandToHand
        Some(BMinus), // Dagger
        Some(D),      // Sword
        None,         // GreatSword
        Some(BMinus), // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(E),      // Club
        None,         // Staff
        Some(APlus),  // Archery
        Some(APlus),  // Marksmanship
        Some(CMinus), // Throwing
        None,         // Guarding
        Some(E),      // Evasion
        None,         // Shield
        None,         // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Sam
    [
        None,         // HandToHand
        Some(E),      // Dagger
        Some(CPlus),  // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        Some(BMinus), // Polearm
        Some(APlus),  // Katana
        Some(APlus),  // GreatKatana
        Some(E),      // Club
        None,         // Staff
        Some(CPlus),  // Archery
        None,         // Marksmanship
        Some(CPlus),  // Throwing
        None,         // Guarding
        Some(BPlus),  // Evasion
        None,         // Shield
        Some(A),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Nin
    [
        Some(E),      // HandToHand
        Some(CPlus),  // Dagger
        Some(C),      // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        Some(APlus),  // Katana
        Some(CMinus), // GreatKatana
        Some(E),      // Club
        None,         // Staff
        Some(E),      // Archery
        Some(C),      // Marksmanship
        Some(APlus),  // Throwing
        None,         // Guarding
        Some(A),      // Evasion
        None,         // Shield
        Some(A),      // Parrying
        None,         // Divine
        None,         // Healing
        None,         // Enhancing
        None,         // Enfeebling
        None,         // Elemental
        None,         // Dark
        None,         // Summoning
        Some(A),      // Ninjutsu
        None, None, None, None, None, None,
    ],
    // Drg
    [
        None,         // HandToHand
        Some(E),      // Dagger
        Some(CMinus), // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        Some(APlus),  // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(E),      // Club
        Some(BMinus), // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(CMinus), // Evasion
        None,         // Shield
        Some(C),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Smn
    [
        None,         // HandToHand
        Some(E),      // Dagger
        None,         // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(CPlus),  // Club
        Some(B),      // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(E),      // Evasion
        None,         // Shield
        None,         // Parrying
        None,         // Divine
        None,         // Healing
        None,         // Enhancing
        None,         // Enfeebling
        None,         // Elemental
        None,         // Dark
        Some(A),      // Summoning
        None, None, None, None, None, None, None,
    ],
    // Blu
    [
        None,         // HandToHand
        None,         // Dagger
        Some(APlus),  // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(B),      // Club
        None,         // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(C),      // Evasion
        None,         // Shield
        Some(D),      // Parrying
        None,         // Divine
        None,         // Healing
        None,         // Enhancing
        None,         // Enfeebling
        None,         // Elemental
        None,         // Dark
        None,         // Summoning
        None,         // Ninjutsu
        None,         // Singing
        None,         // StringInstrument
        None,         // WindInstrument
        Some(APlus),  // BlueMagic
        None, None,
    ],
    // Cor
    [
        Some(APlus),  // HandToHand
        Some(BPlus),  // Dagger
        Some(BMinus), // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        None,         // Club
        None,         // Staff
        None,         // Archery
        Some(B),      // Marksmanship
        Some(CPlus),  // Throwing
        Some(B),      // Guarding
        Some(B),      // Evasion
        None,         // Shield
        Some(A),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Pup
    [
        Some(D),      // HandToHand
        Some(CMinus), // Dagger
        None,         // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(D),      // Club
        None,         // Staff
        None,         // Archery
        None,         // Marksmanship
        Some(CPlus),  // Throwing
        None,         // Guarding
        Some(BPlus),  // Evasion
        None,         // Shield
        Some(D),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Dnc
    [
        None,         // HandToHand
        Some(APlus),  // Dagger
        Some(D),      // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(CPlus),  // Club
        Some(CPlus),  // Staff
        None,         // Archery
        None,         // Marksmanship
        Some(D),      // Throwing
        None,         // Guarding
        Some(E),      // Evasion
        None,         // Shield
        Some(E),      // Parrying
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Sch
    [
        None,         // HandToHand
        Some(D),      // Dagger
        None,         // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(BPlus),  // Club
        Some(CPlus),  // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(D),      // Evasion
        None,         // Shield
        Some(E),      // Parrying
        Some(BPlus),  // Divine
        Some(BPlus),  // Healing
        Some(BPlus),  // Enhancing
        Some(BPlus),  // Enfeebling
        Some(BPlus),  // Elemental
        Some(BPlus),  // Dark
        None, None, None, None, None, None, None, None,
    ],
    // Geo
    [
        None,         // HandToHand
        None,         // Dagger
        None,         // Sword
        None,         // GreatSword
        None,         // Axe
        None,         // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(CMinus), // Club
        None,         // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(BPlus),  // Evasion
        None,         // Shield
        Some(APlus),  // Parrying
        None,         // Divine
        None,         // Healing
        None,         // Enhancing
        Some(CPlus),  // Enfeebling
        Some(BPlus),  // Elemental
        Some(CPlus),  // Dark
        None,         // Summoning
        None,         // Ninjutsu
        None,         // Singing
        None,         // StringInstrument
        None,         // WindInstrument
        None,         // BlueMagic
        Some(C),      // Geomancy
        Some(C),      // Handbell
    ],
    // Run
    [
        None,         // HandToHand
        None,         // Dagger
        Some(A),      // Sword
        Some(APlus),  // GreatSword
        Some(BMinus), // Axe
        Some(B),      // GreatAxe
        None,         // Scythe
        None,         // Polearm
        None,         // Katana
        None,         // GreatKatana
        Some(CMinus), // Club
        None,         // Staff
        None,         // Archery
        None,         // Marksmanship
        None,         // Throwing
        None,         // Guarding
        Some(BPlus),  // Evasion
        None,         // Shield
        None,         // Parrying
        Some(B),      // Divine
        None,         // Healing
        Some(BMinus), // Enhancing
        None, None, None, None, None, None, None, None, None, None, None,
    ],
];

/// 指定ジョブが指定スキルに持つランクを返す。None は未習得。
pub fn job_skill_rank(job: Job, skill: SkillKind) -> Option<SkillRank> {
    JOB_SKILL_RANKS[job as usize][skill as usize]
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

/// 全ジョブのレベル情報から、あるスキルのデフォルト値（キャップの最大）を計算する。
/// レベルが 0 のジョブは無視する。
pub fn default_skill_value(skill: SkillKind, job_levels: &EnumMap<Job, JobLevel>) -> i32 {
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
    max
}

/// 全スキルについてのデフォルト値を算出する。
pub fn default_skills(job_levels: &EnumMap<Job, JobLevel>) -> CharacterSkills {
    let mut skills = CharacterSkills::default();
    for skill in <SkillKind as VariantArray>::VARIANTS {
        skills.values[*skill] = default_skill_value(*skill, job_levels);
    }
    skills
}

// ---------------------------------------------------------------------------
// Effective Skill (char value capped by job)
// ---------------------------------------------------------------------------

/// メイン/サポートジョブの組み合わせにおけるスキルの有効値を計算する。
/// キャラクターのスキル値とジョブ経由のキャップの最大値のうち、低い方を返す。
/// キャップはメインジョブ（+ ML）とサポートジョブ（support_lv で上限）のうち高い方。
pub fn effective_skill(
    skill: SkillKind,
    main_job: Job,
    main_lv: i32,
    master_lv: i32,
    support_job: Option<Job>,
    support_lv: Option<i32>,
    char_value: i32,
) -> i32 {
    let main_cap = job_skill_cap(main_job, skill, main_lv, master_lv);
    let sup_cap = match (support_job, support_lv) {
        (Some(sj), Some(sl)) => job_skill_cap(sj, skill, sl, 0),
        _ => 0,
    };
    let max_cap = main_cap.max(sup_cap);
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
        // キャラクターがスキル値 500 を持っていても、War の両手斧 Lv99 ML0 の上限 424 で制限される
        let v = effective_skill(
            SkillKind::GreatAxe,
            Job::War,
            99,
            0,
            None,
            None,
            500,
        );
        assert_eq!(v, 424);
    }

    #[test]
    fn test_effective_skill_capped_by_char_value() {
        // キャラクターのスキル値 200 < cap 424 のとき、200 が使われる
        let v = effective_skill(
            SkillKind::GreatAxe,
            Job::War,
            99,
            0,
            None,
            None,
            200,
        );
        assert_eq!(v, 200);
    }

    #[test]
    fn test_effective_skill_support_job_higher() {
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
        );
        // Nin の片手刀 A+ @ Lv49, 線形補間で 1-50 の中
        // APlus: [6, 153, 276, 424] at [1, 50, 75, 99]
        // Lv49: 6 + (49-1)/(50-1) * (153-6) = 6 + 48/49 * 147 ≈ 6 + 144 = 150
        assert!(v >= 140 && v <= 160, "v = {}", v);
    }

    #[test]
    fn test_effective_skill_no_main_rank() {
        // War は魔法スキル持たないので 0
        let v = effective_skill(
            SkillKind::Healing,
            Job::War,
            99,
            50,
            None,
            None,
            300,
        );
        assert_eq!(v, 0);
    }

    #[test]
    fn test_default_skill_value_single_job() {
        let mut jl: EnumMap<Job, JobLevel> = EnumMap::default();
        jl[Job::War] = JobLevel { level: 99, master_lv: 50 };
        // War の両手斧 A+ @ 99 ML50 = 474
        assert_eq!(default_skill_value(SkillKind::GreatAxe, &jl), 474);
        // War は魔法なし
        assert_eq!(default_skill_value(SkillKind::Healing, &jl), 0);
    }

    #[test]
    fn test_default_skill_value_multiple_jobs() {
        let mut jl: EnumMap<Job, JobLevel> = EnumMap::default();
        jl[Job::War] = JobLevel { level: 99, master_lv: 0 }; // GreatAxe A+ = 424
        jl[Job::Drk] = JobLevel { level: 50, master_lv: 0 }; // GreatAxe B- @ 50 = 126
        // War のほうが大きい
        assert_eq!(default_skill_value(SkillKind::GreatAxe, &jl), 424);
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
