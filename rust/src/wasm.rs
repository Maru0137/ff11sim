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
    match kind {
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
    let skills = default_skills(&profile.job_levels);
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
