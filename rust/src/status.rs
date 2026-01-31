use core::panic;

pub use strum::{EnumCount, EnumIter, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray)]
pub enum Grade {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray)]
pub enum BpKind {
    Str,
    Dex,
    Vit,
    Agi,
    Int,
    Mnd,
    Chr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray)]
pub enum StatusKind {
    Hp,
    Mp,
    Str,
    Dex,
    Vit,
    Agi,
    Int,
    Mnd,
    Chr,
}

#[derive(Debug, Clone)]
pub struct Status {
    pub hp: i32,
    pub mp: i32,

    pub str: i32,
    pub dex: i32,
    pub vit: i32,
    pub agi: i32,
    pub int: i32,
    pub mnd: i32,
    pub chr: i32,
}

const GRADE_COEF_HPMP: [[f32; 5]; Grade::COUNT] = [
    // Base, 60, 75, 99, 30+
    [19.0, 9.0, 3.0, 3.0, 1.0],
    [17.0, 8.0, 3.0, 3.0, 1.0],
    [16.0, 7.0, 3.0, 3.0, 1.0],
    [14.0, 6.0, 3.0, 3.0, 0.0],
    [13.0, 5.0, 2.0, 2.0, 0.0],
    [11.0, 4.0, 2.0, 2.0, 0.0],
    [10.0, 3.0, 2.0, 2.0, 0.0],
];

const GRADE_COEF_BP: [[f32; 4]; Grade::COUNT] = [
    // Base, 60, 75, 99
    [5.0, 0.5, 0.11, 0.39],
    [4.0, 0.45, 0.21, 0.39],
    [4.0, 0.4, 0.29, 0.39],
    [3.0, 0.35, 0.34, 0.39],
    [3.0, 0.3, 0.34, 0.39],
    [2.0, 0.25, 0.39, 0.39],
    [2.0, 0.2, 0.42, 0.39],
];

// Master Level bonus per level for each stat
// HP: +7, MP: +2 (only if job has MP), BP stats: +1
const MASTER_LV_BONUS: [i32; StatusKind::COUNT] = [
    7, // HP
    2, // MP (only applied if main job has MP)
    1, // STR
    1, // DEX
    1, // VIT
    1, // AGI
    1, // INT
    1, // MND
    1, // CHR
];

pub fn calc_master_lv_bonus(kind: StatusKind, mlv: i32) -> i32 {
    MASTER_LV_BONUS[kind as usize] * mlv
}

// Merit point bonus per rank for each stat
// HP/MP: +10 per rank, other stats: +1 per rank
const MERIT_POINT_BONUS: [i32; StatusKind::COUNT] = [
    10, // HP
    10, // MP
    1,  // STR
    1,  // DEX
    1,  // VIT
    1,  // AGI
    1,  // INT
    1,  // MND
    1,  // CHR
];

#[derive(Debug, Clone, Copy, Default)]
pub struct MeritPoints {
    pub hp: i32,
    pub mp: i32,
    pub str_: i32,
    pub dex: i32,
    pub vit: i32,
    pub agi: i32,
    pub int: i32,
    pub mnd: i32,
    pub chr: i32,
}

impl MeritPoints {
    fn get(&self, kind: StatusKind) -> i32 {
        match kind {
            StatusKind::Hp => self.hp,
            StatusKind::Mp => self.mp,
            StatusKind::Str => self.str_,
            StatusKind::Dex => self.dex,
            StatusKind::Vit => self.vit,
            StatusKind::Agi => self.agi,
            StatusKind::Int => self.int,
            StatusKind::Mnd => self.mnd,
            StatusKind::Chr => self.chr,
        }
    }

    pub fn status_bonus(&self, kind: StatusKind) -> i32 {
        let rank = self.get(kind);
        assert!(rank >= 0 && rank <= 15, "merit point rank must be between 0 and 15");
        MERIT_POINT_BONUS[kind as usize] * rank
    }
}

impl Grade {
    pub fn base(&self, kind: StatusKind) -> f32 {
        match kind {
            StatusKind::Hp | StatusKind::Mp => GRADE_COEF_HPMP[*self as usize][0],
            _ => GRADE_COEF_BP[*self as usize][0],
        }
    }

    pub fn coef(&self, kind: StatusKind, lv: i32) -> f32 {
        let idx = match lv {
            2..=60 => 1,
            61..=75 => 2,
            76..=99 => 3,
            _ => panic!("lv must be between 0 and 99: {}", lv),
        };

        match kind {
            StatusKind::Hp | StatusKind::Mp => GRADE_COEF_HPMP[*self as usize][idx],
            _ => GRADE_COEF_BP[*self as usize][idx],
        }
    }

    pub fn coef_30plus(&self, kind: StatusKind) -> f32 {
        match kind {
            StatusKind::Hp | StatusKind::Mp => GRADE_COEF_HPMP[*self as usize][4],
            _ => panic!("coef_30plus is not applicable for BP"),
        }
    }
}

pub fn calc_status(kind: StatusKind, grade: Grade, lv: i32) -> f32 {
    if lv == 0 {
        return 0.0;
    }

    // truncate for each term with 0.5
    let mut ret = grade.base(kind);
    ret += (grade.coef(kind, 2) * std::cmp::min(lv - 1, 59) as f32 * 2.0).floor() / 2.0;
    ret += (grade.coef(kind, 61) * std::cmp::min(std::cmp::max(lv - 60, 0), 15) as f32 * 2.0)
        .floor()
        / 2.0;
    ret += (grade.coef(kind, 76) * std::cmp::max(lv - 75, 0) as f32 * 2.0).floor() / 2.0;

    if kind == StatusKind::Hp || kind == StatusKind::Mp {
        ret += (grade.coef_30plus(kind) * std::cmp::max(lv - 30, 0) as f32 * 2.0).floor() / 2.0;
    }
    return ret;
}
