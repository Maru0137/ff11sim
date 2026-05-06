//! `/data/*.json` から JSON 化したテーブルデータを読み込み、
//! `LazyLock` 経由でアクセス可能にする集約モジュール。
//!
//! 各 JSON は `{ "version": u32, "data": ... }` 形式。
//! Rust 側は `include_str!` でバイナリに埋め込み、初回アクセス時に
//! `serde_json::from_str` でパースする (panic on error)。
//!
//! 設計判断: enum-map の `serde` feature が有効なので、`EnumMap<Job, T>` 等は
//! 全バリアント存在を要求する deserialize となる → データ欠損は parse 段階で
//! 即エラーになり、強力な静的検証として機能する。

use std::sync::LazyLock;

use enum_map::EnumMap;
use serde::Deserialize;

use crate::job::Job;
use crate::race::Race;
use crate::skills::{SkillKind, SkillRank};
use crate::status::{Grade, StatusKind};

/// 全 JSON ファイル共通のラッパ形式
#[derive(Debug, Clone, Deserialize)]
pub struct DataFile<T> {
    #[allow(dead_code)]
    pub version: u32,
    pub data: T,
}

// ---------------------------------------------------------------------------
// Tier 1: 共有メタデータ
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct JobMeta {
    pub key: Job,
    pub name_ja: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RaceMeta {
    pub key: Race,
    pub name_ja: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillMeta {
    pub key: SkillKind,
    pub name_ja: String,
    pub category: SkillCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum SkillCategory {
    Weapon,
    Defense,
    Magic,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentSlotMeta {
    pub key: String,
    pub label_ja: String,
}

pub static JOBS_META: LazyLock<Vec<JobMeta>> = LazyLock::new(|| {
    serde_json::from_str::<DataFile<Vec<JobMeta>>>(include_str!("../../data/jobs.json"))
        .expect("jobs.json parse failed")
        .data
});

pub static RACES_META: LazyLock<Vec<RaceMeta>> = LazyLock::new(|| {
    serde_json::from_str::<DataFile<Vec<RaceMeta>>>(include_str!("../../data/races.json"))
        .expect("races.json parse failed")
        .data
});

pub static SKILLS_META: LazyLock<Vec<SkillMeta>> = LazyLock::new(|| {
    serde_json::from_str::<DataFile<Vec<SkillMeta>>>(include_str!("../../data/skills.json"))
        .expect("skills.json parse failed")
        .data
});

pub static EQUIPMENT_SLOTS_META: LazyLock<Vec<EquipmentSlotMeta>> = LazyLock::new(|| {
    serde_json::from_str::<DataFile<Vec<EquipmentSlotMeta>>>(include_str!(
        "../../data/equipment_slots.json"
    ))
    .expect("equipment_slots.json parse failed")
    .data
});

// ---------------------------------------------------------------------------
// Tier 3: ステータス grade / 係数
// ---------------------------------------------------------------------------

/// 種族別ステータス grade。`EnumMap<StatusKind, Grade>` の deserialize は
/// 全 9 stat の存在を要求するので、データ欠損は parse 段階で panic。
pub static RACE_STATUS_GRADES: LazyLock<EnumMap<Race, EnumMap<StatusKind, Grade>>> =
    LazyLock::new(|| {
        serde_json::from_str::<DataFile<EnumMap<Race, EnumMap<StatusKind, Grade>>>>(include_str!(
            "../../data/race_status_grades.json"
        ))
        .expect("race_status_grades.json parse failed")
        .data
    });

/// ジョブ別ステータス grade。MP を持たないジョブは `None`。
pub static JOB_STATUS_GRADES: LazyLock<EnumMap<Job, EnumMap<StatusKind, Option<Grade>>>> =
    LazyLock::new(|| {
        serde_json::from_str::<DataFile<EnumMap<Job, EnumMap<StatusKind, Option<Grade>>>>>(
            include_str!("../../data/job_status_grades.json"),
        )
        .expect("job_status_grades.json parse failed")
        .data
    });

#[derive(Debug, Clone, Deserialize)]
pub struct GradeCoefficients {
    /// HP/MP grade 係数 [Base, 60, 75, 99, 30+]
    pub hpmp: EnumMap<Grade, [f32; 5]>,
    /// 7 base parameter grade 係数 [Base, 60, 75, 99]
    pub bp: EnumMap<Grade, [f32; 4]>,
}

pub static GRADE_COEFFICIENTS: LazyLock<GradeCoefficients> = LazyLock::new(|| {
    serde_json::from_str::<DataFile<GradeCoefficients>>(include_str!(
        "../../data/grade_coefficients.json"
    ))
    .expect("grade_coefficients.json parse failed")
    .data
});

/// ジョブ × スキル ランク行列。未習得は `None`。
pub static JOB_SKILL_RANKS: LazyLock<EnumMap<Job, EnumMap<SkillKind, Option<SkillRank>>>> =
    LazyLock::new(|| {
        serde_json::from_str::<DataFile<EnumMap<Job, EnumMap<SkillKind, Option<SkillRank>>>>>(
            include_str!("../../data/job_skill_ranks.json"),
        )
        .expect("job_skill_ranks.json parse failed")
        .data
    });

#[derive(Debug, Clone, Deserialize)]
pub struct SkillCapControlPoints {
    /// 制御点を取る Lv (= [1, 50, 75, 99])
    pub control_levels: [i32; 4],
    /// rank ごとの制御点 (Lv1, Lv50, Lv75, Lv99)
    pub ranks: EnumMap<SkillRank, [i32; 4]>,
}

pub static SKILL_CAP_CONTROL_POINTS: LazyLock<SkillCapControlPoints> = LazyLock::new(|| {
    serde_json::from_str::<DataFile<SkillCapControlPoints>>(include_str!(
        "../../data/skill_cap_control_points.json"
    ))
    .expect("skill_cap_control_points.json parse failed")
    .data
});

// ギフト定義は src/gift.rs (Gift enum + Job::gift_tiers) に移行済み。
// 旧 JSON ベースの GiftSlotDef / JOB_GIFTS は削除した。

#[cfg(test)]
mod tests {
    use super::*;
    use strum::VariantArray;

    #[test]
    fn jobs_meta_covers_all_jobs() {
        let keys: Vec<Job> = JOBS_META.iter().map(|m| m.key).collect();
        assert_eq!(keys.len(), Job::VARIANTS.len(), "job count mismatch");
        for v in Job::VARIANTS {
            assert!(keys.contains(v), "missing job: {:?}", v);
        }
    }

    #[test]
    fn races_meta_covers_all_races() {
        let keys: Vec<Race> = RACES_META.iter().map(|m| m.key).collect();
        assert_eq!(keys.len(), Race::VARIANTS.len(), "race count mismatch");
        for v in Race::VARIANTS {
            assert!(keys.contains(v), "missing race: {:?}", v);
        }
    }

    #[test]
    fn skills_meta_covers_all_skills() {
        let keys: Vec<SkillKind> = SKILLS_META.iter().map(|m| m.key).collect();
        assert_eq!(keys.len(), SkillKind::VARIANTS.len(), "skill count mismatch");
        for v in SkillKind::VARIANTS {
            assert!(keys.contains(v), "missing skill: {:?}", v);
        }
    }

    #[test]
    fn equipment_slots_count() {
        assert_eq!(EQUIPMENT_SLOTS_META.len(), 16);
    }
}
