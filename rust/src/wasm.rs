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
    default_skills, effective_skill, weapon_skill_from_item_id, SkillKind,
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
    /// 装備メイン武器のスキル有効値（主武器スキルによる attack/accuracy への寄与分）
    pub main_weapon_skill_value: i32,
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
    use crate::status::{calc_defense, calc_evasion, calc_magic_attack, calc_magic_defense};
    let vit = chara.status(StatusKind::Vit);
    let agi = chara.status(StatusKind::Agi);
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

    // 各スキルの有効値（main/support のキャップで制限されたキャラクター値）
    let mut effective_skills: BTreeMap<String, i32> = BTreeMap::new();
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
        effective_skills.insert(skill_kind_to_key(*skill).to_string(), v);
    }
    // 回避計算に使う回避スキル有効値
    let eff_evasion_skill = *effective_skills
        .get(skill_kind_to_key(SkillKind::Evasion))
        .unwrap_or(&0);

    // メイン武器のスキル種別があれば、そのスキル値を attack/accuracy に加算する
    let (main_weapon_skill, main_weapon_skill_value) = match chara
        .bonus_stats
        .main_weapon_skill_id
        .and_then(weapon_skill_from_item_id)
    {
        Some(skill) => {
            let v = effective_skill(
                skill,
                chara.main_job,
                chara.main_lv,
                chara.master_lv,
                chara.support_job,
                chara.support_lv,
                chara.skills.get(skill),
            );
            (Some(skill_kind_to_key(skill).to_string()), v)
        }
        None => (None, 0),
    };

    // トレイト系ボーナスとギフト/JPカテゴリ効果を合算
    let attack_bonus = attack_bonus_trait
        + gift.physical_attack
        + jp_cat.physical_attack
        + main_weapon_skill_value;
    let defense_bonus = defense_bonus_trait + gift.physical_defense + jp_cat.physical_defense;
    let evasion_bonus = evasion_bonus_trait + gift.physical_evasion + jp_cat.physical_evasion;
    let accuracy_bonus = accuracy_bonus_trait
        + gift.physical_accuracy
        + jp_cat.physical_accuracy
        + main_weapon_skill_value;
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

    StatusResult {
        hp: chara.status(StatusKind::Hp),
        mp: chara.status(StatusKind::Mp),
        str_: chara.status(StatusKind::Str),
        dex: chara.status(StatusKind::Dex),
        vit,
        agi,
        int: chara.status(StatusKind::Int),
        mnd: chara.status(StatusKind::Mnd),
        chr: chara.status(StatusKind::Chr),
        def: def_total,
        mdef: mdef_total,
        evasion: evasion_total,
        magic_attack: magic_attack_total,
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
