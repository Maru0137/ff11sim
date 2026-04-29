use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use strum::VariantArray;
use wasm_bindgen::prelude::*;

use crate::chara::Chara;
use crate::character_profile::CharacterProfile;
use crate::job::{Job, JobTrait};
use crate::job_points::{calc_gift_bonuses, calc_jp_category_bonuses};
use crate::race::Race;
use crate::skills::{
    default_skills, effective_skill, job_skill_rank, weapon_skill_from_item_id, SkillKind,
};
use crate::status::{BonusStats, MeritPoints, StatusKind};

/// BTreeMap を JS Map ではなく plain object として出力するためのシリアライザ
fn object_serializer() -> Serializer {
    Serializer::new().serialize_maps_as_objects(true)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Deserialize)]
pub struct StatusResult {
    pub hp: i32,
    pub mp: i32,
    pub str_: i32,
    pub dex: i32,
    pub vit: i32,
    pub agi: i32,
    pub int: i32,
    pub mnd: i32,
    pub chr: i32,
    /// 防御力総合値 (int(VIT*1.5) + Lv + α + equip + トレイト/ギフト/JPカテゴリ)
    pub def: i32,
    /// 魔法防御力総合値 (100 + equip + トレイト/ギフト/JPカテゴリ)
    pub mdef: i32,
    /// 回避総合値 (int(AGI/2) + スキル区分値 + equip + トレイト/ギフト/JPカテゴリ)
    pub evasion: i32,
    /// 魔法攻撃力総合値 (100 + equip + トレイト/ギフト/JPカテゴリ)
    pub magic_attack: i32,
    /// メイン攻撃力総合値
    pub main_attack: i32,
    /// メイン命中総合値
    pub main_accuracy: i32,
    /// サブ攻撃力 (サブ武器装備時のみ、未装備は None)
    pub sub_attack: Option<i32>,
    /// サブ命中 (サブ武器装備時のみ、未装備は None)
    pub sub_accuracy: Option<i32>,
    /// 飛攻 (レンジ武器装備時のみ、未装備は None)
    pub ranged_attack: Option<i32>,
    /// 飛命 (レンジ武器装備時のみ、未装備は None)
    pub ranged_accuracy: Option<i32>,
    pub attack_bonus: i32,
    pub defense_bonus: i32,
    pub evasion_bonus: i32,
    pub accuracy_bonus: i32,
    pub magic_attack_bonus: i32,
    pub magic_accuracy_bonus: i32,
    pub magic_evasion_bonus: i32,
    pub total_jp_spent: i32,
    /// メインジョブ/サポートジョブで制限されたスキル有効値（キー: スキル名）
    pub effective_skills: BTreeMap<String, i32>,
    /// 装備メイン武器のスキル種別（取得できた場合のみ）
    pub main_weapon_skill: Option<String>,
    /// 装備メイン武器のスキル有効値
    pub main_weapon_skill_value: i32,
    /// 装備サブ武器のスキル種別
    pub sub_weapon_skill: Option<String>,
    /// 装備サブ武器のスキル有効値
    pub sub_weapon_skill_value: Option<i32>,
    /// 装備レンジ武器のスキル種別
    pub ranged_weapon_skill: Option<String>,
    /// 装備レンジ武器のスキル有効値
    pub ranged_weapon_skill_value: Option<i32>,
}

fn str_to_race(s: &str) -> Option<Race> {
    match s.to_lowercase().as_str() {
        "hum" | "hume" => Some(Race::Hum),
        "elv" | "elvaan" => Some(Race::Elv),
        "tar" | "tarutaru" => Some(Race::Tar),
        "mit" | "mithra" => Some(Race::Mit),
        "gal" | "galka" => Some(Race::Gal),
        _ => None,
    }
}

fn str_to_job(s: &str) -> Option<Job> {
    match s.to_lowercase().as_str() {
        "war" | "warrior" => Some(Job::War),
        "mnk" | "monk" => Some(Job::Mnk),
        "whm" | "white mage" => Some(Job::Whm),
        "blm" | "black mage" => Some(Job::Blm),
        "rdm" | "red mage" => Some(Job::Rdm),
        "thf" | "thief" => Some(Job::Thf),
        "pld" | "paladin" => Some(Job::Pld),
        "drk" | "dark knight" => Some(Job::Drk),
        "bst" | "beastmaster" => Some(Job::Bst),
        "brd" | "bard" => Some(Job::Brd),
        "rng" | "ranger" => Some(Job::Rng),
        "sam" | "samurai" => Some(Job::Sam),
        "nin" | "ninja" => Some(Job::Nin),
        "drg" | "dragoon" => Some(Job::Drg),
        "smn" | "summoner" => Some(Job::Smn),
        "blu" | "blue mage" => Some(Job::Blu),
        "cor" | "corsair" => Some(Job::Cor),
        "pup" | "puppetmaster" => Some(Job::Pup),
        "dnc" | "dancer" => Some(Job::Dnc),
        "sch" | "scholar" => Some(Job::Sch),
        "geo" | "geomancer" => Some(Job::Geo),
        "run" | "rune fencer" => Some(Job::Run),
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct MeritPointsInput {
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
    pub combat_skill_merits: std::collections::BTreeMap<String, i32>,
    #[serde(default)]
    pub magic_skill_merits: std::collections::BTreeMap<String, i32>,
    #[serde(default)]
    pub enmity_plus: i32,
    #[serde(default)]
    pub enmity_minus: i32,
    #[serde(default)]
    pub critical_hit_rate: i32,
    #[serde(default)]
    pub enemy_critical_hit_rate: i32,
    #[serde(default)]
    pub spell_interruption_rate: i32,
}

impl From<MeritPointsInput> for MeritPoints {
    fn from(input: MeritPointsInput) -> Self {
        MeritPoints {
            hp: input.hp,
            mp: input.mp,
            str_: input.str_,
            dex: input.dex,
            vit: input.vit,
            agi: input.agi,
            int: input.int,
            mnd: input.mnd,
            chr: input.chr,
            combat_skill_merits: input.combat_skill_merits,
            magic_skill_merits: input.magic_skill_merits,
            enmity_plus: input.enmity_plus,
            enmity_minus: input.enmity_minus,
            critical_hit_rate: input.critical_hit_rate,
            enemy_critical_hit_rate: input.enemy_critical_hit_rate,
            spell_interruption_rate: input.spell_interruption_rate,
        }
    }
}

#[wasm_bindgen]
pub fn calculate_status(
    race: &str,
    main_job: &str,
    main_lv: i32,
    support_job: Option<String>,
    support_lv: Option<i32>,
    master_lv: i32,
    merit_points_js: JsValue,
    bonus_stats_js: JsValue,
) -> Result<JsValue, JsValue> {
    let race = str_to_race(race).ok_or_else(|| JsValue::from_str("Invalid race"))?;
    let main_job = str_to_job(main_job).ok_or_else(|| JsValue::from_str("Invalid main job"))?;

    let merit_points: MeritPoints = if merit_points_js.is_undefined() || merit_points_js.is_null() {
        MeritPoints::default()
    } else {
        let input: MeritPointsInput = serde_wasm_bindgen::from_value(merit_points_js)
            .map_err(|e| JsValue::from_str(&format!("Invalid merit points: {}", e)))?;
        input.into()
    };

    let bonus_stats: BonusStats = if bonus_stats_js.is_undefined() || bonus_stats_js.is_null() {
        BonusStats::default()
    } else {
        serde_wasm_bindgen::from_value(bonus_stats_js)
            .map_err(|e| JsValue::from_str(&format!("Invalid bonus stats: {}", e)))?
    };

    let mut builder = Chara::builder()
        .race(race)
        .main_job(main_job, main_lv)
        .master_lv(master_lv)
        .merit_points(merit_points)
        .bonus_stats(bonus_stats);

    if let (Some(sj), Some(sl)) = (support_job, support_lv) {
        let support_job = str_to_job(&sj).ok_or_else(|| JsValue::from_str("Invalid support job"))?;
        builder = builder.support_job(support_job, sl);
    }

    let chara = builder
        .build()
        .map_err(|e| JsValue::from_str(e))?;

    let result = chara_to_status_result(&chara);
    result
        .serialize(&object_serializer())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn get_races() -> Vec<JsValue> {
    vec![
        JsValue::from_str("Hum"),
        JsValue::from_str("Elv"),
        JsValue::from_str("Tar"),
        JsValue::from_str("Mit"),
        JsValue::from_str("Gal"),
    ]
}

#[wasm_bindgen]
pub fn get_jobs() -> Vec<JsValue> {
    vec![
        JsValue::from_str("War"),
        JsValue::from_str("Mnk"),
        JsValue::from_str("Whm"),
        JsValue::from_str("Blm"),
        JsValue::from_str("Rdm"),
        JsValue::from_str("Thf"),
        JsValue::from_str("Pld"),
        JsValue::from_str("Drk"),
        JsValue::from_str("Bst"),
        JsValue::from_str("Brd"),
        JsValue::from_str("Rng"),
        JsValue::from_str("Sam"),
        JsValue::from_str("Nin"),
        JsValue::from_str("Drg"),
        JsValue::from_str("Smn"),
        JsValue::from_str("Blu"),
        JsValue::from_str("Cor"),
        JsValue::from_str("Pup"),
        JsValue::from_str("Dnc"),
        JsValue::from_str("Sch"),
        JsValue::from_str("Geo"),
        JsValue::from_str("Run"),
    ]
}

/// SkillKind を JSON キー用の文字列（Pascal ケース）に変換する。
fn skill_kind_to_key(kind: SkillKind) -> &'static str {
    kind.key()
}

fn chara_to_status_result(chara: &Chara) -> StatusResult {
    use crate::status::{
        calc_accuracy, calc_defense, calc_evasion, calc_magic_attack, calc_magic_defense,
        calc_main_attack, calc_ranged_accuracy, calc_ranged_attack, calc_sub_attack,
    };
    let vit = chara.status(StatusKind::Vit);
    let agi = chara.status(StatusKind::Agi);
    let str_val = chara.status(StatusKind::Str);
    let dex = chara.status(StatusKind::Dex);
    let defense_bonus_trait = chara.job_trait_total(JobTrait::DefenseBonus);
    let mdef_trait = chara.job_trait_total(JobTrait::MagicDefenseBonus);
    let attack_bonus_trait = chara.job_trait_total(JobTrait::AttackBonus);
    let evasion_bonus_trait = chara.job_trait_total(JobTrait::EvasionBonus);
    let accuracy_bonus_trait = chara.job_trait_total(JobTrait::AccuracyBonus);
    let magic_attack_bonus_trait = chara.job_trait_total(JobTrait::MagicAttackBonus);

    // ジョブポイント / ギフトによる戦闘ステータスボーナス
    let total_jp = chara.job_points.total_jp_spent();
    let gift = calc_gift_bonuses(chara.main_job, total_jp);
    let jp_cat = calc_jp_category_bonuses(chara.main_job, &chara.job_points);

    // 各スキルの「基本有効値」= min(char, cap)（装備ボーナスを含まない）
    let mut base_effective: std::collections::HashMap<SkillKind, i32> =
        std::collections::HashMap::new();
    for skill in <SkillKind as VariantArray>::VARIANTS {
        let v = effective_skill(
            *skill,
            chara.main_job,
            chara.main_lv,
            chara.master_lv,
            chara.support_job,
            chara.support_lv,
            chara.skills.get(*skill),
            &chara.merit_points,
        );
        base_effective.insert(*skill, v);
    }

    // 装備ボーナスを引いた上で effective_skills マップを組み立てる
    // 非武器スロットのスキルボーナスは全スロット共通(global)として加算される
    let get_bonus =
        |map: &std::collections::BTreeMap<String, i32>, kind: SkillKind| -> i32 {
            *map.get(skill_kind_to_key(kind)).unwrap_or(&0)
        };
    let global_bonus =
        |kind: SkillKind| get_bonus(&chara.bonus_stats.skill_bonus_global, kind);
    let main_slot_bonus =
        |kind: SkillKind| get_bonus(&chara.bonus_stats.skill_bonus_main, kind);
    let sub_slot_bonus = |kind: SkillKind| get_bonus(&chara.bonus_stats.skill_bonus_sub, kind);
    let ranged_slot_bonus =
        |kind: SkillKind| get_bonus(&chara.bonus_stats.skill_bonus_ranged, kind);

    // 該当ジョブ構成で対象スキルを習得しているか（メイン/サポートどちらかにランクあり）
    let job_has_skill = |kind: SkillKind| -> bool {
        job_skill_rank(chara.main_job, kind).is_some()
            || chara
                .support_job
                .map(|j| job_skill_rank(j, kind).is_some())
                .unwrap_or(false)
    };

    // effective_skills（表示用）: base + global のみ。スロット固有ボーナスは含めない。
    // ジョブ構成がスキルを持たない場合はボーナスも適用しない（未習得扱い）
    let mut effective_skills: BTreeMap<String, i32> = BTreeMap::new();
    for (skill, base) in base_effective.iter() {
        let v = if job_has_skill(*skill) {
            base + global_bonus(*skill)
        } else {
            0
        };
        effective_skills.insert(skill_kind_to_key(*skill).to_string(), v);
    }

    // 回避は非武器スキル → 装備ボーナスは global のみ
    let eff_evasion_skill = base_effective[&SkillKind::Evasion] + global_bonus(SkillKind::Evasion);

    // 指定スロットの武器種別と有効値（スロット固有 + global を加算）を取得するヘルパー
    // ジョブ構成がスキルを持たない場合はボーナスを適用せず、値は 0 となる
    let resolve_weapon_main = |id: Option<i32>| -> Option<(SkillKind, i32)> {
        id.and_then(weapon_skill_from_item_id).map(|skill| {
            let v = if job_has_skill(skill) {
                base_effective[&skill] + main_slot_bonus(skill) + global_bonus(skill)
            } else {
                0
            };
            (skill, v)
        })
    };
    let resolve_weapon_sub = |id: Option<i32>| -> Option<(SkillKind, i32)> {
        id.and_then(weapon_skill_from_item_id).map(|skill| {
            let v = if job_has_skill(skill) {
                base_effective[&skill] + sub_slot_bonus(skill) + global_bonus(skill)
            } else {
                0
            };
            (skill, v)
        })
    };
    let resolve_weapon_ranged = |id: Option<i32>| -> Option<(SkillKind, i32)> {
        id.and_then(weapon_skill_from_item_id).map(|skill| {
            let v = if job_has_skill(skill) {
                base_effective[&skill] + ranged_slot_bonus(skill) + global_bonus(skill)
            } else {
                0
            };
            (skill, v)
        })
    };

    let main_weapon = resolve_weapon_main(chara.bonus_stats.main_weapon_skill_id);
    let sub_weapon = resolve_weapon_sub(chara.bonus_stats.sub_weapon_skill_id);
    let ranged_weapon = resolve_weapon_ranged(chara.bonus_stats.ranged_weapon_skill_id);

    let main_weapon_skill = main_weapon.map(|(k, _)| skill_kind_to_key(k).to_string());
    let main_weapon_skill_value = main_weapon.map(|(_, v)| v).unwrap_or(0);
    let sub_weapon_skill = sub_weapon.map(|(k, _)| skill_kind_to_key(k).to_string());
    let sub_weapon_skill_value = sub_weapon.map(|(_, v)| v);
    let ranged_weapon_skill = ranged_weapon.map(|(k, _)| skill_kind_to_key(k).to_string());
    let ranged_weapon_skill_value = ranged_weapon.map(|(_, v)| v);

    // トレイト系ボーナスとギフト/JPカテゴリ効果を合算（武器スキルは含まない）
    let attack_bonus = attack_bonus_trait + gift.physical_attack + jp_cat.physical_attack;
    let defense_bonus = defense_bonus_trait + gift.physical_defense + jp_cat.physical_defense;
    let evasion_bonus = evasion_bonus_trait + gift.physical_evasion + jp_cat.physical_evasion;
    let accuracy_bonus = accuracy_bonus_trait + gift.physical_accuracy + jp_cat.physical_accuracy;
    let magic_attack_bonus = magic_attack_bonus_trait + gift.magic_attack + jp_cat.magic_attack;
    let magic_accuracy_bonus = gift.magic_accuracy + jp_cat.magic_accuracy;
    let magic_evasion_bonus = gift.magic_evasion + jp_cat.magic_evasion;

    // 総合値の計算
    let def_total = calc_defense(vit, chara.main_lv, chara.bonus_stats.def) + defense_bonus;
    let mdef_total = calc_magic_defense(chara.bonus_stats.magic_def_bonus)
        + mdef_trait
        + gift.magic_defense
        + jp_cat.magic_defense;
    let evasion_total =
        calc_evasion(agi, eff_evasion_skill, chara.bonus_stats.evasion) + evasion_bonus;
    let magic_attack_total =
        calc_magic_attack(chara.bonus_stats.magic_attack) + magic_attack_bonus;

    // メイン攻撃/命中
    // メイン武器未装備時は H2H 扱いで H2H スキル値を使う
    let (main_skill_value, is_h2h) = if let Some((kind, v)) = main_weapon {
        (v, kind == SkillKind::HandToHand)
    } else {
        // 武器なし = H2H. スロット固有ボーナスはメインスロット扱い
        let h2h_v = base_effective[&SkillKind::HandToHand]
            + main_slot_bonus(SkillKind::HandToHand)
            + global_bonus(SkillKind::HandToHand);
        (h2h_v, true)
    };
    let main_attack_total = calc_main_attack(
        str_val,
        main_skill_value,
        is_h2h,
        chara.bonus_stats.attack,
    ) + attack_bonus;
    let main_accuracy_total =
        calc_accuracy(dex, main_skill_value, chara.bonus_stats.accuracy) + accuracy_bonus;

    // サブ攻撃/命中 (サブ武器装備時のみ)
    let (sub_attack_total, sub_accuracy_total) = match sub_weapon {
        Some((_, skill_v)) => {
            let atk =
                calc_sub_attack(str_val, skill_v, chara.bonus_stats.attack) + attack_bonus;
            let acc =
                calc_accuracy(dex, skill_v, chara.bonus_stats.accuracy) + accuracy_bonus;
            (Some(atk), Some(acc))
        }
        None => (None, None),
    };

    // 飛攻/飛命 (レンジ武器装備時のみ)
    let (ranged_attack_total, ranged_accuracy_total) = match ranged_weapon {
        Some((_, skill_v)) => {
            let atk = calc_ranged_attack(str_val, skill_v, chara.bonus_stats.ranged_attack)
                + attack_bonus;
            let acc = calc_ranged_accuracy(agi, skill_v, chara.bonus_stats.ranged_accuracy)
                + accuracy_bonus;
            (Some(atk), Some(acc))
        }
        None => (None, None),
    };

    StatusResult {
        hp: chara.status(StatusKind::Hp),
        mp: chara.status(StatusKind::Mp),
        str_: str_val,
        dex,
        vit,
        agi,
        int: chara.status(StatusKind::Int),
        mnd: chara.status(StatusKind::Mnd),
        chr: chara.status(StatusKind::Chr),
        def: def_total,
        mdef: mdef_total,
        evasion: evasion_total,
        magic_attack: magic_attack_total,
        main_attack: main_attack_total,
        main_accuracy: main_accuracy_total,
        sub_attack: sub_attack_total,
        sub_accuracy: sub_accuracy_total,
        ranged_attack: ranged_attack_total,
        ranged_accuracy: ranged_accuracy_total,
        attack_bonus,
        defense_bonus,
        evasion_bonus,
        accuracy_bonus,
        magic_attack_bonus,
        magic_accuracy_bonus,
        magic_evasion_bonus,
        total_jp_spent: total_jp,
        effective_skills,
        main_weapon_skill,
        main_weapon_skill_value,
        sub_weapon_skill,
        sub_weapon_skill_value,
        ranged_weapon_skill,
        ranged_weapon_skill_value,
    }
}

/// CharacterProfile からデフォルトスキル値（全ジョブのキャップの最大）を算出する WASM 関数。
/// JS: calculate_default_skills(profile) → { HandToHand: 0, ..., Handbell: 0 }
#[wasm_bindgen]
pub fn calculate_default_skills(profile_js: JsValue) -> Result<JsValue, JsValue> {
    let profile: CharacterProfile = serde_wasm_bindgen::from_value(profile_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid profile: {}", e)))?;
    let skills = default_skills(&profile.job_levels, &profile.merit_points);
    let mut map: BTreeMap<String, i32> = BTreeMap::new();
    for skill in <SkillKind as VariantArray>::VARIANTS {
        map.insert(
            skill_kind_to_key(*skill).to_string(),
            skills.values[*skill],
        );
    }
    map.serialize(&object_serializer())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// CharacterProfile の JSON データからステータスを計算する。
/// profile_js: CharacterProfile を JSON シリアライズした JsValue
/// main_job: メインジョブ名（例: "War"）
/// support_job: サポートジョブ名（例: "Drg"）、なしの場合は None
#[wasm_bindgen]
pub fn calculate_status_from_profile(
    profile_js: JsValue,
    main_job: &str,
    support_job: Option<String>,
    bonus_stats_js: JsValue,
) -> Result<JsValue, JsValue> {
    let profile: CharacterProfile = serde_wasm_bindgen::from_value(profile_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid profile: {}", e)))?;

    let main_job = str_to_job(main_job)
        .ok_or_else(|| JsValue::from_str("Invalid main job"))?;

    let support_job = match support_job {
        Some(ref sj) => Some(
            str_to_job(sj).ok_or_else(|| JsValue::from_str("Invalid support job"))?,
        ),
        None => None,
    };

    let bonus_stats: BonusStats = if bonus_stats_js.is_undefined() || bonus_stats_js.is_null() {
        BonusStats::default()
    } else {
        serde_wasm_bindgen::from_value(bonus_stats_js)
            .map_err(|e| JsValue::from_str(&format!("Invalid bonus stats: {}", e)))?
    };

    let mut chara = profile
        .to_chara(main_job, support_job)
        .map_err(|e| JsValue::from_str(&e))?;
    chara.bonus_stats = bonus_stats;

    let result = chara_to_status_result(&chara);
    result
        .serialize(&object_serializer())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chara::Chara;
    use crate::job::Job;
    use crate::job_points::JobPointCategories;
    use crate::race::Race;
    use crate::skills::{CharacterSkills, SkillKind};
    use crate::status::{BonusStats, MeritPoints};
    use std::collections::BTreeMap;

    /// Hum War99/Sam59 ML50 + ラフリア装備セットの攻撃力・命中テスト
    /// サポート Lv は main_lv/2 + master_lv/5 = 99/2 + 50/5 = 49 + 10 = 59
    ///
    /// 装備:
    ///   メイン: ラフリア (両手斧, STR+35, ACC+35, 両手斧スキル+277)
    ///   サブ:   ウトゥグリップ (ATK+30, ACC+30)
    ///   矢弾:  コイストボダーA30 (ATK+15, STR+10, DEX+10)
    ///   頭:    サクパタヘルムA30 (STR+33, DEX+20, ATK+70, ACC+55)
    ///   胴:    BIロリカ+3 (STR+43, DEX+39, ATK+74, ACC+64, 両手斧スキル+21)
    ///   両手:  サクパタガントレA30 (STR+24, DEX+35, ATK+70, ACC+55)
    ///   両脚:  AGクウィス+4 (STR+48, ATK+74, ACC+44)
    ///   両足:  サクパタレギンスA30 (STR+29, DEX+20, ATK+70, ACC+55)
    ///   首:    戦士の数珠+2A25 (ATK+25, ACC+25, オグメ: HP+100, STR+15, DEX+15, DA+7%)
    ///   腰:    イオスケハベルト+1 (ACC+17)
    ///   耳1:   アスプロピアスA30 (HP+100, ACC+15, ALL BP+10)
    ///   耳2:   ボイイピアス+1 (オグメ: ACC+13)
    ///   指1:   シーリチリング+1 (ACC+10)
    ///   指2:   シーリチリング+1 (ACC+10)
    ///   背:    シコルマント (オグメ: DEX+30, ACC+20, ATK+20)
    ///
    /// 装備以外のステータス内訳（現行実装の期待値）:
    ///   ※ 通常ステータスは race+main_job+support_job を sum→floor 後に
    ///      ML/メリット/ジョブ特性を加算する仕様
    ///
    ///   STR=161  内訳: メインジョブ(Hum 37.50 + War99 45.00) + サポートジョブ(Sam59 27.00/2=13.50)
    ///                   = floor(96.00) + マスターレベル(50*1=50) + メリット(15*1=15)  [+ ジョブポイント=0, ギフト=0]
    ///   DEX=156  内訳: floor(37.50 + 40.50 + 13.50) + 50 + 15
    ///   VIT=153  内訳: floor(37.50 + 37.50 + 13.50) + 50 + 15
    ///   AGI=154  内訳: floor(37.50 + 40.50 + 11.50) + 50 + 15
    ///   INT=143  内訳: floor(37.50 + 31.00 + 10.00) + 50 + 15
    ///   MND=143  内訳: floor(37.50 + 31.00 + 10.00) + 50 + 15
    ///   CHR=148  内訳: floor(37.50 + 34.50 + 11.50) + 50 + 15
    ///   HP=2095  内訳: floor(485 + 675 + 255) + ML(50*7=350) + メリット(15*10=150)
    ///                   + ジョブ特性 MaxHpBoost(War90 rank4=180)
    ///   MP=0     War はメインで MP グレードを持たない（実装上 0 を返す）
    ///
    ///   戦闘ボーナス（ジョブポイント＋ギフト＋ジョブ特性、装備分は別途加算）:
    ///     物理攻撃 +105 = ジョブ特性 AttackBonus(War91 rank3=35) + ギフト(2100JP→70) + JPカテゴリ(0)
    ///     物理命中  +36 = ジョブ特性(0) + ギフト(2100JP→36) + JPカテゴリ(0)
    ///     ※ JP はカテゴリ全 max=2100JP 累計、War のカテゴリは攻撃/命中に直接寄与なし
    ///
    ///   メイン武器有効スキル(両手斧):
    ///     キャップ = job_skill_cap(War99 ML50 GreatAxe A+)=474 + メリット(8rank*2=16) = 490
    ///     キャラスキル値 490 を採用 → base 490
    ///     + メインスロット(ラフリア +277) + 全スロット共通(BIロリカ+3 +21) = 788
    ///
    ///   最終期待値:
    ///     メイン攻撃力 = STR + 武器スキル + 8 + 装備攻撃 + 戦闘ボーナス(攻撃)
    ///                  = (161+247) + 788 + 8 + 448 + 105
    ///                  = 408 + 788 + 8 + 448 + 105 = 1757
    ///       ※ STR 247 = 装備合計（222 + アスプロ ALL BP+10 + 戦士の数珠オグメ STR+15）
    ///
    ///     メイン命中 = floor(DEX × 0.75) + accuracy_skill_term(skill) + 装備命中 + 戦闘ボーナス(命中)
    ///                = floor((156+179) × 0.75) + accuracy_skill_term(788) + 448 + 36
    ///                = floor(335 × 0.75=251.25) + 709 + 448 + 36
    ///                = 251 + 709 + 448 + 36 = 1444
    ///       ※ DEX 179 = 装備合計（154 + アスプロ ALL BP+10 + 戦士の数珠オグメ DEX+15）
    ///       ※ accuracy_skill_term(788): skill>600 区分 → 540 + floor((788-600)×0.9=169.2) = 540 + 169 = 709
    ///       ※ 装備命中 448 = 431 + アスプロ ACC+15 + ボイイ補正 +2
    #[test]
    fn test_war_laphria_equipset_attack_accuracy() {
        // メリットポイント: ステータス全て 15、全スキル 8
        let mut merit = MeritPoints {
            hp: 15,
            mp: 15,
            str_: 15,
            dex: 15,
            vit: 15,
            agi: 15,
            int: 15,
            mnd: 15,
            chr: 15,
            ..Default::default()
        };
        for &key in &[
            "HandToHand", "Dagger", "Sword", "GreatSword", "Axe", "GreatAxe",
            "Scythe", "Polearm", "Katana", "GreatKatana", "Club", "Staff",
            "Archery", "Marksmanship", "Throwing", "Guarding", "Evasion",
            "Shield", "Parrying",
        ] {
            merit.combat_skill_merits.insert(key.to_string(), 8);
        }
        for &key in &[
            "Divine", "Healing", "Enhancing", "Enfeebling", "Elemental",
            "Dark", "Summoning", "Ninjutsu", "Singing", "StringInstrument",
            "WindInstrument", "BlueMagic", "Geomancy", "Handbell",
        ] {
            merit.magic_skill_merits.insert(key.to_string(), 8);
        }

        // スキル値: War99 ML50 両手斧 A+ cap = 474 + merit 16 = 490
        let mut skills = CharacterSkills::default();
        skills.set(SkillKind::GreatAxe, 490);

        // ジョブポイント: 全カテゴリ 20 (2100 JP 消費)
        let jp = JobPointCategories::all_maxed();

        // 装備ボーナス
        let mut skill_bonus_main: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_main.insert("GreatAxe".to_string(), 277); // ラフリア

        let mut skill_bonus_global: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_global.insert("GreatAxe".to_string(), 21); // BIロリカ+3

        // 装備ボーナス（アスプロピアスA30 ALL BP+10、戦士の数珠+2A25 オグメ STR/DEX+15 を反映）
        let bonus = BonusStats {
            hp: 200,       // アスプロ HP+100 + 数珠+2A25 オグメ HP+100
            str_: 247,     // 装備 STR 合計 (222 + アスプロ +10 + 数珠オグメ +15)
            dex: 179,      // 装備 DEX 合計 (154 + アスプロ +10 + 数珠オグメ +15)
            vit: 10,       // アスプロ ALL BP +10
            agi: 10,       // アスプロ ALL BP +10
            int: 10,       // アスプロ ALL BP +10
            mnd: 10,       // アスプロ ALL BP +10
            chr: 10,       // アスプロ ALL BP +10
            attack: 448,   // 装備 Attack 合計
            accuracy: 448, // 装備 Accuracy 合計 (431 + アスプロ ACC+15 + ボイイ +2 補正)
            main_weapon_skill_id: Some(6), // 両手斧 skill ID
            skill_bonus_main,
            skill_bonus_global,
            ..BonusStats::default()
        };

        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Sam, 59)
            .master_lv(50)
            .merit_points(merit)
            .job_points(jp)
            .skills(skills)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");

        let result = chara_to_status_result(&chara);

        assert_eq!(
            result.main_attack, 1757,
            "メイン攻撃力: got {} expected 1757",
            result.main_attack
        );
        assert_eq!(
            result.main_accuracy, 1444,
            "メイン命中: got {} expected 1444",
            result.main_accuracy
        );
    }

    /// Hum War99/Sam59 ML50 + 近接物理 WS 装備セット (ラフリア両手斧構成) の
    /// メイン攻撃力・命中テスト
    ///
    /// 装備:
    ///   メイン: ラフリア+3相当 (両手斧, STR+35 VIT+35 ACC+35 MAcc+35 両手斧+277 受け流し+277)
    ///   サブ:   ウトゥグリップ (HP+70 ATK+30 ACC+30)
    ///   矢弾:  ノブキエリ (ATK+23 WSダメ+6%)
    ///   頭:    AGマスク+4 (HP+68 STR+40 DEX+28 VIT+40 AGI+28 INT+31 MND+28 CHR+28
    ///                    ACC+42 ATK+93 MAcc+42 受け流し+22 WSダメ+12% etc.)
    ///   首:    戦士の数珠+2A25 (ATK+25 ACC+25 / オグメ A25: HP+100 STR+15 DEX+15 DA+7%)
    ///   耳1:   胡蝶のイヤリング (カスタム: 魔攻+4 TPボーナス+250)
    ///   耳2:   スラッドピアス (STR+10 VIT+10 WSダメ+3%)
    ///   胴:    ニャメメイル B30 (大量ステ+ペット系 / オグメ B30: ATK+35 RAtk+35 STR/VIT+10 etc.)
    ///   両手:  ニャメガントレ B30 (オグメ B30: ATK+35 RAtk+35 VIT+15 etc.)
    ///   指1:   コーネリアリング (WS命中+20 WSダメ+10%)
    ///   指2:   王将の指輪 (HP+50 STR+10 DEX+10 VIT+10 AGI+10 ATK+20 RAtk+20)
    ///   背:    シコルマント (カスタム: STR+30 ACC+20 ATK+20 WSダメ+10% 被物理-10%)
    ///   腰:    セールフィベルト+1 A15 (ヘイスト+9% TA+2% / オグメ A15: STR+15 DA+5%)
    ///   両脚:  BIクウィス+3 (HP+80 STR+53 VIT+40 AGI+30 INT+38 MND+27 CHR+25
    ///                       ACC+63 ATK+73 MAcc+63 フェンサー+3 物理ダメ上限+10% TPボ+100 etc.)
    ///   両足:  ニャメソルレット B30 (オグメ B30: ATK+35 RAtk+35 ACC+13 RAcc+13 etc.)
    ///
    /// 装備合計（Web パーサー検証済み、Pet: セクション除外、Unity Ranking 最大値、
    ///   Weapon Skill Accuracy 除外後）:
    ///   STR=293 DEX=145 VIT=258 AGI=159 INT=164 MND=158 CHR=150 HP=663 MP=205
    ///   ATK=494 ACC=348 RAtk=215 RAcc=133 MAtk=94 MAcc=260
    ///   skill_bonus_main: GreatAxe+277  / skill_bonus_global: Parrying+299
    ///   ※ ATK 494 = ベース 479 + Unity Ranking 攻+15 (max)
    ///   ※ ACC 348 はメイン命中用（Cornelia の WS命中+10 はメイン命中に含めない）
    ///
    /// 最終期待値（メイン攻撃/命中、装備以外の内訳は test_war_laphria を参照）:
    ///   メイン武器有効スキル(両手斧):
    ///     キャップ = job_skill_cap(War99 ML50 GreatAxe A+)=474 + メリット(8rank*2=16) = 490
    ///     キャラスキル値 490 採用 → base 490 + メインスロット(ラフリア +277) = 767
    ///
    ///   メイン攻撃力 = STR + 武器スキル + 8 + 装備攻撃 + 戦闘ボーナス(攻撃)
    ///                = (161+293) + 767 + 8 + 494 + 105
    ///                = 454 + 767 + 8 + 494 + 105 = 1828
    ///
    ///   メイン命中 = floor(DEX × 0.75) + accuracy_skill_term(skill) + 装備命中 + 戦闘ボーナス(命中)
    ///              = floor((156+145) × 0.75) + accuracy_skill_term(767) + 348 + 36
    ///              = floor(301 × 0.75=225.75) + (540 + floor((767-600)×0.9=150.3)) + 348 + 36
    ///              = 225 + 690 + 348 + 36 = 1299
    #[test]
    fn test_war_ws_set_attack_accuracy() {
        // メリットポイント: ステータス全て 15、全スキル 8
        let mut merit = MeritPoints {
            hp: 15,
            mp: 15,
            str_: 15,
            dex: 15,
            vit: 15,
            agi: 15,
            int: 15,
            mnd: 15,
            chr: 15,
            ..Default::default()
        };
        for &key in &[
            "HandToHand", "Dagger", "Sword", "GreatSword", "Axe", "GreatAxe",
            "Scythe", "Polearm", "Katana", "GreatKatana", "Club", "Staff",
            "Archery", "Marksmanship", "Throwing", "Guarding", "Evasion",
            "Shield", "Parrying",
        ] {
            merit.combat_skill_merits.insert(key.to_string(), 8);
        }
        for &key in &[
            "Divine", "Healing", "Enhancing", "Enfeebling", "Elemental",
            "Dark", "Summoning", "Ninjutsu", "Singing", "StringInstrument",
            "WindInstrument", "BlueMagic", "Geomancy", "Handbell",
        ] {
            merit.magic_skill_merits.insert(key.to_string(), 8);
        }

        // スキル値: War99 ML50 両手斧 A+ cap = 474 + merit 16 = 490
        let mut skills = CharacterSkills::default();
        skills.set(SkillKind::GreatAxe, 490);

        let jp = JobPointCategories::all_maxed();

        // 装備スキルボーナス（main: ラフリア両手斧+277、global: Parrying+299）
        let mut skill_bonus_main: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_main.insert("GreatAxe".to_string(), 277);

        let mut skill_bonus_global: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_global.insert("Parrying".to_string(), 299);

        let bonus = BonusStats {
            hp: 663,
            mp: 205,
            str_: 293,
            dex: 145,
            vit: 258,
            agi: 159,
            int: 164,
            mnd: 158,
            chr: 150,
            attack: 494,
            accuracy: 348,
            ranged_attack: 215,
            ranged_accuracy: 133,
            magic_attack: 94,
            main_weapon_skill_id: Some(6), // 両手斧 skill ID
            skill_bonus_main,
            skill_bonus_global,
            ..BonusStats::default()
        };

        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Sam, 59)
            .master_lv(50)
            .merit_points(merit)
            .job_points(jp)
            .skills(skills)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");

        let result = chara_to_status_result(&chara);

        assert_eq!(
            result.main_attack, 1828,
            "メイン攻撃力: got {} expected 1828",
            result.main_attack
        );
        assert_eq!(
            result.main_accuracy, 1299,
            "メイン命中: got {} expected 1299",
            result.main_accuracy
        );
    }

}
