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

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
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
    pub fn get(&self, kind: StatusKind) -> i32 {
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BonusStats {
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub mp: i32,
    #[serde(default)]
    pub str_: i32,
    #[serde(default)]
    pub dex: i32,
    #[serde(default)]
    pub vit: i32,
    #[serde(default)]
    pub agi: i32,
    #[serde(default)]
    pub int: i32,
    #[serde(default)]
    pub mnd: i32,
    #[serde(default)]
    pub chr: i32,
    #[serde(default)]
    pub def: i32,
    #[serde(default)]
    pub magic_def_bonus: i32,
    /// 装備品の回避（Evasion+X）合計
    #[serde(default)]
    pub evasion: i32,
    /// 装備品の魔法攻撃力（Magic Atk.+X）合計
    #[serde(default)]
    pub magic_attack: i32,
    /// 装備品の攻撃力合計
    #[serde(default)]
    pub attack: i32,
    /// 装備品の命中合計
    #[serde(default)]
    pub accuracy: i32,
    /// 装備品の飛攻合計
    #[serde(default)]
    pub ranged_attack: i32,
    /// 装備品の飛命合計
    #[serde(default)]
    pub ranged_accuracy: i32,
    /// 装備メイン武器のスキル種別 ID（アイテム JSON の `skill` フィールド値、未装備時は None）
    #[serde(default)]
    pub main_weapon_skill_id: Option<i32>,
    /// 装備サブ武器のスキル種別 ID
    #[serde(default)]
    pub sub_weapon_skill_id: Option<i32>,
    /// 装備レンジ武器のスキル種別 ID
    #[serde(default)]
    pub ranged_weapon_skill_id: Option<i32>,
    /// メインスロット装備の武器スキルボーナス（メインスロットの武器スキル計算にのみ加算）
    #[serde(default)]
    pub skill_bonus_main: std::collections::BTreeMap<String, i32>,
    /// サブスロット装備の武器スキルボーナス
    #[serde(default)]
    pub skill_bonus_sub: std::collections::BTreeMap<String, i32>,
    /// レンジスロット装備の武器スキルボーナス
    #[serde(default)]
    pub skill_bonus_ranged: std::collections::BTreeMap<String, i32>,
    /// 全スロット共通で加算されるスキルボーナス（非武器スロット装備、および武器スロット装備の非武器スキル）
    #[serde(default)]
    pub skill_bonus_global: std::collections::BTreeMap<String, i32>,
}

impl BonusStats {
    pub fn get(&self, kind: StatusKind) -> i32 {
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

/// 防御力を計算する。
/// DEF = int(VIT * 1.5) + Lv + α + equip_def
/// α: Lv1-50=8, Lv51-59=8+(Lv-50), Lv60-90=18, Lv91-99=18+int((Lv-89)/2)
pub fn calc_defense(vit: i32, lv: i32, equip_def: i32) -> i32 {
    let alpha = if lv <= 50 {
        8
    } else if lv <= 59 {
        8 + (lv - 50)
    } else if lv <= 90 {
        18
    } else {
        18 + (lv - 89) / 2
    };
    (vit as f32 * 1.5) as i32 + lv + alpha + equip_def
}

/// 魔法防御力を計算する。
/// Magic Defense = 100 + equip_mdef
pub fn calc_magic_defense(equip_mdef: i32) -> i32 {
    100 + equip_mdef
}

/// 回避値を計算する（wiki.ffo.jp/html/1688.html）。
/// スキル値による区分的な曲線が適用される:
///   スキル ≤ 200: 回避 = int(AGI × 0.5) + 回避スキル
///   スキル 201-400: 回避 = int(AGI × 0.5) + 200 + int((回避スキル - 200) × 0.9)
///   スキル 401+:   回避 = int(AGI × 0.5) + 380 + int((回避スキル - 400) × 0.8)
/// さらに装備品の回避ボーナスを加算する。
pub fn calc_evasion(agi: i32, evasion_skill: i32, equip_evasion: i32) -> i32 {
    let skill_term = if evasion_skill <= 200 {
        evasion_skill
    } else if evasion_skill <= 400 {
        200 + ((evasion_skill - 200) as f32 * 0.9).floor() as i32
    } else {
        380 + ((evasion_skill - 400) as f32 * 0.8).floor() as i32
    };
    (agi as f32 * 0.5).floor() as i32 + skill_term + equip_evasion
}

/// 魔法攻撃力を計算する（wiki.ffo.jp/html/3411.html）。
/// Magic Attack = 100（基準値）+ 装備品合計
pub fn calc_magic_attack(equip_matk: i32) -> i32 {
    100 + equip_matk
}

/// 命中における武器スキルによる寄与（wiki.ffo.jp/html/223.html）。
/// スキル 1-200: +1 / スキル
/// スキル 201-400: +0.9 / スキル
/// スキル 401-600: +0.8 / スキル
/// スキル 601-: +0.9 / スキル
pub fn accuracy_skill_term(weapon_skill: i32) -> i32 {
    if weapon_skill <= 0 {
        0
    } else if weapon_skill <= 200 {
        weapon_skill
    } else if weapon_skill <= 400 {
        200 + ((weapon_skill - 200) as f32 * 0.9).floor() as i32
    } else if weapon_skill <= 600 {
        380 + ((weapon_skill - 400) as f32 * 0.8).floor() as i32
    } else {
        540 + ((weapon_skill - 600) as f32 * 0.9).floor() as i32
    }
}

/// メイン武器の攻撃力を計算する（wiki.ffo.jp/html/1766.html）。
/// 片手/両手: 攻撃 = STR + 武器スキル + 8 + equip_attack
/// 格闘:      攻撃 = int(STR × 0.75) + 武器スキル + 8 + equip_attack
pub fn calc_main_attack(str_val: i32, weapon_skill: i32, is_h2h: bool, equip_attack: i32) -> i32 {
    let str_term = if is_h2h {
        (str_val as f32 * 0.75).floor() as i32
    } else {
        str_val
    };
    str_term + weapon_skill + 8 + equip_attack
}

/// サブ武器の攻撃力を計算する（wiki.ffo.jp/html/1766.html）。
/// サブ: 攻撃 = int(STR × 0.5) + 武器スキル + 8 + equip_attack
pub fn calc_sub_attack(str_val: i32, weapon_skill: i32, equip_attack: i32) -> i32 {
    (str_val as f32 * 0.5).floor() as i32 + weapon_skill + 8 + equip_attack
}

/// 遠隔武器の攻撃力（飛攻）を計算する。
/// 飛攻 = STR + 武器スキル + 8 + equip_ranged_attack
pub fn calc_ranged_attack(str_val: i32, weapon_skill: i32, equip_ranged_attack: i32) -> i32 {
    str_val + weapon_skill + 8 + equip_ranged_attack
}

/// 命中値を計算する（wiki.ffo.jp/html/223.html）。
/// 命中 = int(DEX × 0.75) + スキル補正 + equip_accuracy
pub fn calc_accuracy(dex: i32, weapon_skill: i32, equip_accuracy: i32) -> i32 {
    (dex as f32 * 0.75).floor() as i32 + accuracy_skill_term(weapon_skill) + equip_accuracy
}

/// 飛命を計算する。
/// 飛命 = int(AGI × 0.5) + スキル補正 + equip_ranged_accuracy
pub fn calc_ranged_accuracy(agi: i32, weapon_skill: i32, equip_ranged_accuracy: i32) -> i32 {
    (agi as f32 * 0.5).floor() as i32
        + accuracy_skill_term(weapon_skill)
        + equip_ranged_accuracy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_defense_lv99() {
        // VIT=100, Lv=99, equip=0 → floor(100*1.5)=150, α=18+(99-89)/2=23, total=150+99+23+0=272
        assert_eq!(calc_defense(100, 99, 0), 272);
    }

    #[test]
    fn test_calc_magic_defense() {
        assert_eq!(calc_magic_defense(0), 100);
        assert_eq!(calc_magic_defense(50), 150);
    }

    #[test]
    fn test_calc_evasion_skill_low() {
        // スキル ≤ 200: 回避 = int(AGI * 0.5) + skill + equip
        // AGI=100, skill=150, equip=0 → 50 + 150 + 0 = 200
        assert_eq!(calc_evasion(100, 150, 0), 200);
        // AGI=100, skill=200, equip=10 → 50 + 200 + 10 = 260
        assert_eq!(calc_evasion(100, 200, 10), 260);
    }

    #[test]
    fn test_calc_evasion_skill_mid() {
        // スキル 201-400: 回避 = int(AGI * 0.5) + 200 + int((skill - 200) * 0.9) + equip
        // AGI=100, skill=300, equip=0 → 50 + 200 + floor(100 * 0.9) + 0 = 50 + 200 + 90 = 340
        assert_eq!(calc_evasion(100, 300, 0), 340);
        // スキル 400 境界: 50 + 200 + floor(200 * 0.9) = 50 + 200 + 180 = 430
        assert_eq!(calc_evasion(100, 400, 0), 430);
    }

    #[test]
    fn test_calc_evasion_skill_high() {
        // スキル 401+: 回避 = int(AGI * 0.5) + 380 + int((skill - 400) * 0.8) + equip
        // AGI=150, skill=474, equip=20 → 75 + 380 + floor(74*0.8) + 20 = 75 + 380 + 59 + 20 = 534
        assert_eq!(calc_evasion(150, 474, 20), 534);
        // スキル 400 境界（上側）: 50 + 380 + 0 = 430 (middle の 400 と同じ)
        assert_eq!(calc_evasion(100, 400, 0), 430);
    }

    #[test]
    fn test_calc_evasion_boundary_consistency() {
        // スキル=200 の時、low/mid 式で同じ値になる
        // low: 50 + 200 + 0 = 250
        // mid: 50 + 200 + 0 + 0 = 250 (skill=200 なら mid 式は適用されない)
        assert_eq!(calc_evasion(100, 200, 0), 250);
    }

    #[test]
    fn test_calc_magic_attack() {
        assert_eq!(calc_magic_attack(0), 100);
        assert_eq!(calc_magic_attack(20), 120);
    }

    #[test]
    fn test_accuracy_skill_term() {
        assert_eq!(accuracy_skill_term(0), 0);
        assert_eq!(accuracy_skill_term(200), 200);
        // 201-400: 200 + floor((skill-200)*0.9)
        assert_eq!(accuracy_skill_term(300), 290); // 200 + floor(90)
        assert_eq!(accuracy_skill_term(400), 380);
        // 401-600: 380 + floor((skill-400)*0.8)
        assert_eq!(accuracy_skill_term(500), 460); // 380 + floor(80)
        assert_eq!(accuracy_skill_term(600), 540);
        // 601+: 540 + floor((skill-600)*0.9)
        assert_eq!(accuracy_skill_term(700), 630); // 540 + floor(90)
    }

    #[test]
    fn test_calc_main_attack_normal() {
        // STR=100, skill=400, equip=50 → 100 + 400 + 8 + 50 = 558
        assert_eq!(calc_main_attack(100, 400, false, 50), 558);
    }

    #[test]
    fn test_calc_main_attack_h2h() {
        // STR=100, skill=400, H2H: int(100*0.75) + 400 + 8 + 50 = 75 + 458 = 533
        assert_eq!(calc_main_attack(100, 400, true, 50), 533);
    }

    #[test]
    fn test_calc_sub_attack() {
        // STR=100, skill=300, equip=30 → int(50) + 300 + 8 + 30 = 388
        assert_eq!(calc_sub_attack(100, 300, 30), 388);
    }

    #[test]
    fn test_calc_ranged_attack() {
        // STR=100, skill=400, equip=30 → 100 + 400 + 8 + 30 = 538
        assert_eq!(calc_ranged_attack(100, 400, 30), 538);
    }

    #[test]
    fn test_calc_accuracy_low_skill() {
        // DEX=100, skill=200, equip=0 → 75 + 200 + 0 = 275
        assert_eq!(calc_accuracy(100, 200, 0), 275);
    }

    #[test]
    fn test_calc_accuracy_high_skill() {
        // DEX=100, skill=500, equip=20 → 75 + 460 + 20 = 555
        assert_eq!(calc_accuracy(100, 500, 20), 555);
    }

    #[test]
    fn test_calc_ranged_accuracy() {
        // AGI=120, skill=400, equip=10 → 60 + 380 + 10 = 450
        assert_eq!(calc_ranged_accuracy(120, 400, 10), 450);
    }
}
