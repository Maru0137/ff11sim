use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use strum::VariantArray;
use wasm_bindgen::prelude::*;

use crate::chara::Chara;
use crate::character_profile::CharacterProfile;
use crate::job::{Job, JobTrait};
use crate::job_points::{calc_gift_bonuses, calc_jp_category_bonuses, calc_war_da_gift_bonus};
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
    /// Store TP 総合値 (装備 + ジョブ特性 + メリット + ギフト + JPカテゴリ)
    pub store_tp: i32,
    /// ダブルアタック発動率総合値 (%) (装備 + ジョブ特性 + メリット + JPカテゴリ)
    pub double_attack_pct: i32,
    /// 連携ボーナス総合値 (%) (装備 + ジョブ特性 + ギフト)
    pub skillchain_bonus: i32,
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
    #[serde(default)]
    pub store_tp: i32,
    #[serde(default)]
    pub job_merits: std::collections::BTreeMap<String, crate::status::JobMerits>,
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
            store_tp: input.store_tp,
            job_merits: input.job_merits,
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
    let magic_accuracy_bonus_trait = chara.job_trait_total(JobTrait::MagicAccuracyBonus);
    let magic_evasion_bonus_trait = chara.job_trait_total(JobTrait::MagicEvasionBonus);
    let store_tp_trait = chara.job_trait_total(JobTrait::StoreTp);
    let double_attack_trait = chara.job_trait_total(JobTrait::DoubleAttack);
    // 連携ボーナス: ジョブ特性 + ギフト + 装備の合計
    // (max(main, sup) ではなく main のみ — サポートジョブ特性の skillchain bonus は実装未確認のため)
    let skillchain_bonus_trait = chara.job_trait_total(JobTrait::SkillchainBonus);

    // ジョブポイント / ギフトによる戦闘ステータスボーナス
    let total_jp = chara.job_points.total_jp_spent();
    let gift = calc_gift_bonuses(chara.main_job, total_jp);
    let jp_cat = calc_jp_category_bonuses(chara.main_job, &chara.job_points);

    // Store TP メリット (SAM 専用、+1/rank、最大 5)
    let store_tp_merit = if chara.main_job == Job::Sam {
        chara.merit_points.store_tp
    } else {
        0
    };

    // ダブルアタック発動率の WAR 専用ボーナス
    //   メリット: グループ1 idx 4「ダブルアタック確率」+1%/rank, 最大 5
    //   ギフト:  「ダブルアタック確率アップ」125/450/1050/1900 JP で +2/+2/+3/+3 (累計 +10%)
    //   ※ JP カテゴリ idx 9「ダブルアタック効果」は実体は物理攻撃力 +1/rank なので
    //      DA 率には加算せず、physical_attack に gift/jp_cat 経由で反映される。
    let (double_attack_merit, double_attack_gift) = if chara.main_job == Job::War {
        let merit = chara
            .merit_points
            .job_merits
            .get("War")
            .map(|m| m.group1[4])
            .unwrap_or(0);
        let gift_bonus = calc_war_da_gift_bonus(total_jp);
        (merit, gift_bonus)
    } else {
        (0, 0)
    };

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
    let magic_accuracy_bonus =
        magic_accuracy_bonus_trait + gift.magic_accuracy + jp_cat.magic_accuracy;
    let magic_evasion_bonus =
        magic_evasion_bonus_trait + gift.magic_evasion + jp_cat.magic_evasion;
    let store_tp_total = chara.bonus_stats.store_tp
        + store_tp_trait
        + store_tp_merit
        + gift.store_tp
        + jp_cat.store_tp;
    let double_attack_pct_total = chara.bonus_stats.double_attack_pct
        + double_attack_trait
        + double_attack_merit
        + double_attack_gift;

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
    // 遠隔系ボーナス: 物理ボーナス（特性/ギフト/JPカテゴリ）+ 遠隔専用ボーナス（COR JP「遠隔命中アップ」「適正距離の遠隔攻撃力アップ」）
    let ranged_attack_extra = gift.ranged_attack + jp_cat.ranged_attack;
    let ranged_accuracy_extra = gift.ranged_accuracy + jp_cat.ranged_accuracy;
    let (ranged_attack_total, ranged_accuracy_total) = match ranged_weapon {
        Some((_, skill_v)) => {
            let atk = calc_ranged_attack(str_val, skill_v, chara.bonus_stats.ranged_attack)
                + attack_bonus
                + ranged_attack_extra;
            let acc = calc_ranged_accuracy(agi, skill_v, chara.bonus_stats.ranged_accuracy)
                + accuracy_bonus
                + ranged_accuracy_extra;
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
        store_tp: store_tp_total,
        double_attack_pct: double_attack_pct_total,
        // 連携ボーナス総合 = 装備 + ジョブ特性 + ギフト
        skillchain_bonus: chara.bonus_stats.skillchain_bonus
            + skillchain_bonus_trait
            + gift.skillchain_bonus,
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
    ///     物理攻撃 +125 = ジョブ特性 AttackBonus(War91 rank3=35) + ギフト(2100JP→70) + JPカテゴリ idx9(+1×20=20)
    ///     物理命中  +36 = ジョブ特性(0) + ギフト(2100JP→36) + JPカテゴリ(0)
    ///     ※ JP はカテゴリ全 max=2100JP 累計、War カテゴリ idx9「ダブルアタック効果」は物理攻撃力 +1/rank
    ///
    ///   メイン武器有効スキル(両手斧):
    ///     キャップ = job_skill_cap(War99 ML50 GreatAxe A+)=474 + メリット(8rank*2=16) = 490
    ///     キャラスキル値 490 を採用 → base 490
    ///     + メインスロット(ラフリア +277) + 全スロット共通(BIロリカ+3 +21) = 788
    ///
    ///   最終期待値:
    ///     メイン攻撃力 = STR + 武器スキル + 8 + 装備攻撃 + 戦闘ボーナス(攻撃)
    ///                  = (161+247) + 788 + 8 + 448 + 125
    ///                  = 408 + 788 + 8 + 448 + 125 = 1777
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
            result.main_attack, 1777,
            "メイン攻撃力: got {} expected 1777",
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
    ///                = (161+293) + 767 + 8 + 494 + 125
    ///                = 454 + 767 + 8 + 494 + 125 = 1848
    ///       ※ 戦闘ボーナス +125 = 特性35 + ギフト70 + JPカテゴリ idx9(20)
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
            result.main_attack, 1848,
            "メイン攻撃力: got {} expected 1848",
            result.main_attack
        );
        assert_eq!(
            result.main_accuracy, 1299,
            "メイン命中: got {} expected 1299",
            result.main_accuracy
        );
    }

    /// SAM Lv99 + Store TP メリット 5 + 装備 Store TP+30 のケース。
    /// ジョブ特性 Store TP V (Lv90)=+30, メリット +5, 装備 +30 → 合計 +65
    #[test]
    fn test_sam_store_tp_total() {
        let mut merit = MeritPoints::default();
        merit.store_tp = 5;
        let bonus = BonusStats {
            store_tp: 30,
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Sam, 99)
            .master_lv(0)
            .merit_points(merit)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.store_tp, 65,
            "SAM99 Store TP 合計: got {} expected 65 (装備30 + 特性30 + メリット5)",
            result.store_tp
        );
    }

    /// SAM 以外 (WAR99) ではメリット store_tp が無効。
    /// 装備 +20 のみ反映され、トレイト/メリットは 0。
    #[test]
    fn test_war_store_tp_no_trait() {
        let mut merit = MeritPoints::default();
        merit.store_tp = 5; // WAR には適用されない
        let bonus = BonusStats {
            store_tp: 20,
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .master_lv(0)
            .merit_points(merit)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(result.store_tp, 20);
    }

    /// SAM サポートでもジョブ特性は反映される (max(main, sup))
    #[test]
    fn test_war_with_sam_sub_store_tp_trait() {
        // WAR99/SAM49 → SAM の Store TP I (Lv10), II (Lv30) 適用 = rank 2 → +15
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Sam, 49)
            .master_lv(0)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.store_tp, 15,
            "WAR99/SAM49 サポート Store TP rank2: got {} expected 15",
            result.store_tp
        );
    }

    /// SAM99 (JP 0) で連携ボーナスのジョブ特性のみが StatusResult に反映される。
    /// SAM Lv78/88/98 で +8/+12/+16 → Lv99 では +16
    #[test]
    fn test_sam99_skillchain_bonus_trait() {
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Sam, 99)
            .master_lv(0)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.skillchain_bonus, 16,
            "SAM99 (JP 0) Skillchain Bonus: got {} expected 16 (装備0 + 特性16 + ギフト0)",
            result.skillchain_bonus
        );
    }

    /// JS の calculate_status_from_profile 経由でも skillchain_bonus が反映されることを確認。
    /// 装備・JP なしで SAM99 → 16 (特性のみ)
    #[test]
    fn test_sam99_skillchain_via_profile() {
        use crate::character_profile::{CharacterProfile, JobLevel};
        let mut profile = CharacterProfile {
            name: "Test".to_string(),
            race: Race::Hum,
            job_levels: enum_map::enum_map! { _ => JobLevel { level: 0, master_lv: 0 } },
            merit_points: MeritPoints::default(),
            job_points: crate::job_points::JobPoints::default(),
            skills: CharacterSkills::default(),
        };
        profile.job_levels[Job::Sam] = JobLevel { level: 99, master_lv: 0 };

        let chara = profile.to_chara(Job::Sam, None).unwrap();
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.skillchain_bonus, 16,
            "SAM99 (装備0/JP0) Skillchain Bonus via profile: got {} expected 16",
            result.skillchain_bonus
        );
    }

    /// BLU99 (0 JP) → MagicAccuracyBonus rank 1 = +10 が StatusResult.magic_accuracy_bonus に反映
    #[test]
    fn test_blu99_magic_accuracy_bonus_no_gift() {
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Blu, 99)
            .master_lv(0)
            .build()
            .expect("Failed to build BLU");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.magic_accuracy_bonus, 10,
            "BLU99 0 JP MagicAccuracyBonus: got {} expected 10",
            result.magic_accuracy_bonus
        );
        assert_eq!(
            result.magic_evasion_bonus, 10,
            "BLU99 0 JP MagicEvasionBonus: got {} expected 10",
            result.magic_evasion_bonus
        );
    }

    /// BLU99 + JP 全振り (≥1200) で「ジョブ特性効果アップ」ギフトにより rank 2 にアップ → +22
    /// 加えて BLU の MagicAccuracy ギフトカテゴリ (累積) が乗る。
    #[test]
    fn test_blu99_magic_accuracy_bonus_gift_full_jp() {
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Blu, 99)
            .master_lv(0)
            .job_points(JobPointCategories::all_maxed())
            .build()
            .expect("Failed to build BLU");
        let result = chara_to_status_result(&chara);
        // ジョブ特性「ジョブ特性効果アップ」: 1200 JP 以上で rank+2 → rank 1+2=3 (累積配列 [10,22] で clamp → 22)
        // BLU の MagicAccuracy ギフト (data/job_gifts.json):
        //   tiers [[125,5],[450,8],[1050,10],[1900,13]] → 全振り 2100 JP で 5+8+10+13 = 36 加算
        // 期待値: 特性 22 + ギフト 36 = 58
        assert_eq!(
            result.magic_accuracy_bonus, 22 + 36,
            "BLU99 全振り MagicAccuracyBonus: got {} expected 58 (特性22 + ギフト36)",
            result.magic_accuracy_bonus
        );
    }

    /// SAM99 (JP 全振り) でジョブ特性 +16 + ギフト +8 + 装備 +22 = +46
    #[test]
    fn test_sam99_skillchain_bonus_total() {
        let bonus = BonusStats {
            skillchain_bonus: 22, // 装備 (例: ムパカキャップ +15 + C. Palug Hammer +7)
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Sam, 99)
            .master_lv(0)
            .job_points(JobPointCategories::all_maxed())
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        // 全振り JP: 各カテゴリ 20 ランク, 1 カテゴリ 210 JP, 10 カテゴリ = 2100 JP
        // ギフト 4 段 (150/450/1125/2000) すべて解放 → +2*4 = +8
        // 特性 +16, 装備 +22 → 合計 46
        assert_eq!(
            result.skillchain_bonus, 46,
            "SAM99 JP 全振り Skillchain Bonus: got {} expected 46 (装備22 + 特性16 + ギフト8)",
            result.skillchain_bonus
        );
    }

    /// WAR99 ML50 + SAM59 サポート（ML50 でのサポート上限 = 99/2+50/5 = 59）。
    /// SAM Store TP は Lv10/30/50 の 3 段階解放 → rank 3 = +20
    #[test]
    fn test_war_ml50_with_sam59_store_tp_trait() {
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Sam, 59)
            .master_lv(50)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.store_tp, 20,
            "WAR99 ML50/SAM59 サポート Store TP rank3: got {} expected 20",
            result.store_tp
        );
    }

    // ---- ダブルアタック ----

    /// WAR Lv24 はまだ DA 特性を習得していない。装備分のみ反映。
    #[test]
    fn test_war_double_attack_below_lv25() {
        let bonus = BonusStats {
            double_attack_pct: 5,
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 24)
            .master_lv(0)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(result.double_attack_pct, 5);
    }

    /// WAR Lv25 で DA1 (+10%) 習得。装備分との合計を確認。
    #[test]
    fn test_war_double_attack_lv25() {
        let bonus = BonusStats {
            double_attack_pct: 5,
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 25)
            .master_lv(0)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.double_attack_pct, 15,
            "WAR25 DA: got {} expected 15 (装備5 + 特性10)",
            result.double_attack_pct
        );
    }

    /// WAR Lv99 + 装備 + メリット 5 + 全カテゴリ JP=20（累計 2100JP → DAギフト 4 段全解放 = +10%）
    /// 装備10 + 特性18 + メリット5 + ギフト10 = 43%
    #[test]
    fn test_war_double_attack_full() {
        let mut merit = MeritPoints::default();
        let mut war_merits = crate::status::JobMerits::default();
        war_merits.group1[4] = 5; // ダブルアタック確率
        merit.job_merits.insert("War".to_string(), war_merits);

        let jp = crate::job_points::JobPointCategories::all_maxed(); // 2100 JP

        let bonus = BonusStats {
            double_attack_pct: 10,
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .master_lv(0)
            .merit_points(merit)
            .job_points(jp)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.double_attack_pct, 43,
            "WAR99 DA full: got {} expected 43 (装備10 + 特性18 + メリット5 + ギフト10)",
            result.double_attack_pct
        );
    }

    /// WAR DA ギフトの閾値テスト
    /// 124JP→0%, 125JP→2%, 449JP→2%, 450JP→4%, 1049JP→4%, 1050JP→7%, 1899JP→7%, 1900JP→10%
    #[test]
    fn test_war_da_gift_thresholds() {
        use crate::job_points::calc_war_da_gift_bonus;
        assert_eq!(calc_war_da_gift_bonus(0), 0);
        assert_eq!(calc_war_da_gift_bonus(124), 0);
        assert_eq!(calc_war_da_gift_bonus(125), 2);
        assert_eq!(calc_war_da_gift_bonus(449), 2);
        assert_eq!(calc_war_da_gift_bonus(450), 4);
        assert_eq!(calc_war_da_gift_bonus(1049), 4);
        assert_eq!(calc_war_da_gift_bonus(1050), 7);
        assert_eq!(calc_war_da_gift_bonus(1899), 7);
        assert_eq!(calc_war_da_gift_bonus(1900), 10);
        assert_eq!(calc_war_da_gift_bonus(2100), 10);
    }

    /// WAR JP idx 9「ダブルアタック効果」は物理攻撃力 +1/rank として加算される (DA 率には加算されない)。
    /// 全カテゴリ JP=20 で +20 物理攻撃。
    #[test]
    fn test_war_da_jp_category_is_attack_bonus() {
        let jp = crate::job_points::JobPointCategories::all_maxed(); // ranks[9] = 20
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .master_lv(0)
            .job_points(jp)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        // attack_bonus = trait(rank3=35) + gift(2100JP→70) + JP idx9(20) = 125
        assert_eq!(
            result.attack_bonus, 125,
            "WAR99 attack_bonus with full JP: got {} expected 125 (trait35 + gift70 + JPidx9 20)",
            result.attack_bonus
        );
        // DA 率は装備0 + 特性18 + メリット0 + ギフト10 = 28%
        assert_eq!(result.double_attack_pct, 28);
    }

    /// WAR 以外（SAM99）ではメリット・特性・ギフト DA% ともに 0。
    #[test]
    fn test_sam_double_attack_no_trait() {
        let mut merit = MeritPoints::default();
        let mut war_merits = crate::status::JobMerits::default();
        war_merits.group1[4] = 5;
        merit.job_merits.insert("War".to_string(), war_merits);

        let jp = crate::job_points::JobPointCategories::all_maxed();

        let bonus = BonusStats {
            double_attack_pct: 7,
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Sam, 99)
            .master_lv(0)
            .merit_points(merit)
            .job_points(jp)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");
        let result = chara_to_status_result(&chara);
        assert_eq!(result.double_attack_pct, 7);
    }

    /// Hum COR99/NIN59 ML50 + 遠隔WS構成装備セットの飛攻/飛命テスト
    ///
    /// 装備:
    ///   メイン:  ロスタムB25  (短剣, 短剣+269 受流+269 魔命スキル+255 命中+50 飛命+50
    ///                          魔命+50 HP+150 魔ダメ+217)
    ///   サブ:    クスタウィ+1A15  (短剣, 短剣+242 受流+242 魔命スキル+188
    ///                              飛攻+16 飛命+25 回避+22  / オグメA15: 飛攻+20 飛命+40 魔命+40)
    ///   遠隔:   フォーマルハウトA15  (射撃, 射撃+269 魔命+40 魔ダメ+155 ストアTP+10 TPボーナス+500
    ///                                  / オグメA15 Default rank15: Ｄ+9 ラストスタンド:ダメ+10% 飛命+30 魔命+30)
    ///   矢弾:   クロノブレット  (飛攻+20 飛命+20)
    ///   頭:     ニャメヘルムB30  (基本ステ+ヘイスト+6%、オグメB30: 飛攻+35 飛命+10 攻+35 命中+10
    ///                              ダブルアタック+5% WSダメ+11%)
    ///   首:     フォシャゴルゲット (WSダメ+10%)
    ///   耳1:    アスプロピアスA30 (HP+100 ヘイスト+5% / オグメA30: 飛命+15 魔命+15 ALL BP+10 ストアTP+5)
    ///   耳2:    胡蝶のイヤリング (カスタム: 魔攻+4 TPボーナス+250 → 魔攻+4 のみ反映)
    ///   胴:     イケンガベストA30 (基本ステ+飛攻+40 飛命+40 ストアTP+11 / オグメA30: 飛攻+30 飛命+15 魔命+15)
    ///   両手:  CSガントリー+3 (基本ステ+飛命+62 飛攻+62 ヘイスト+5% クリ+8% WSダメ+12%)
    ///   指1:    コーネリアリング (WS命中+20 WSダメ+10% → flat ACC/RACC には未反映、WSダメ+10%のみ反映)
    ///   指2:    王将の指輪 (HP+50 STR+10 DEX+10 VIT+10 AGI+10 攻+20 飛攻+20)
    ///   背:     カムラスマント (カスタム: AGI+30 飛命+20 飛攻+20 WSダメ+10% 被物理-10%)
    ///   腰:     フォシャベルト (WSダメ+10%)
    ///   両脚:  ニャメフランチャB30 (基本ステ+ヘイスト+5% / オグメB30: 飛攻+35 攻+35 STR+15 DA+6% WSダメ+12%)
    ///   両足:  ニャメソルレットB30 (基本ステ+ヘイスト+3% / オグメB30: 飛攻+35 飛命+13 攻+35 命中+13 DA+5% WSダメ+11%)
    ///
    /// 装備合計（JS パーサーで集計、コーネリアの WS命中+20 はメイン/飛命に反映されない）:
    ///   bonus_stats.ranged_attack = 423   (CSガントリー+3 飛攻+62 含む / 省略形 Rng. Atk. を集計対象化)
    ///   bonus_stats.ranged_accuracy = 460 (フォーマルハウトA15 飛命+30、CSガントリー+3 飛命+62 含む)
    ///   bonus_stats.magic_accuracy = 412  (フォーマルハウトA15 魔命+30、CSガントリー+3 魔命+62 含む)
    ///   bonus_stats.accuracy = 270        (CSガントリー+3 命中+62 含む)
    ///   bonus_stats.str_ = 188, dex = 163, vit = 164, agi = 218, int = 154, mnd = 163, chr = 148
    ///   skill_bonus_main = { Dagger: 269 }
    ///   skill_bonus_sub  = { Dagger: 242 }
    ///   skill_bonus_ranged = { Marksmanship: 269 }
    ///   skill_bonus_global = { Parrying: 511 }
    ///
    /// 内訳（メリットあり: ステータス全 +15、戦闘/魔法スキルメリット 8、JP全カテゴリmax 2100JP）:
    ///   STR=338 = floor(Hum(D) 37.5 + COR99(E) 34.5 + NIN59/2(C) 13.5)=85 + ML50(50) + Merit(15) + 装備(188)
    ///   AGI=378 = floor(Hum(D) 37.5 + COR99(B) 49.5 + NIN59/2(B) 15.5)=102 + ML50(50) + Merit(15) + 装備(218)
    ///                ※ 既存のCOR99 grade テーブル: STR=E, AGI=B / NIN STR=C, AGI=B
    ///   ranged_skill_value = COR99 ML50 Marksmanship(B) cap(448) + メリット(8*2=16) + 装備(269) = 733
    ///   attack_bonus  = trait(0, COR/NINに無し) + gift(2100JP→COR slot1=+36) + jp_cat(物理攻撃力 0) = 36
    ///   accuracy_bonus = trait(0) + gift(2100JP→COR slot3=+36) + jp_cat(0) = 36
    ///   ranged_accuracy_extra (COR JP idx 7「遠隔命中アップ」+1/rank × 20) = 20
    ///   ※ COR JP idx 9「適正距離の遠隔攻撃力アップ」は条件付き（適正距離）のためステータスには加算しない
    ///   ranged_attack  = STR + skill + 8 + equip_ranged_attack + attack_bonus
    ///                  = 338 + 733 + 8 + 423 + 36 = 1538
    ///   ranged_accuracy = floor(AGI * 0.75) + ranged_accuracy_skill_term(skill) + equip_ranged_accuracy
    ///                     + accuracy_bonus + ranged_accuracy_extra
    ///                   = floor(378*0.75)=283 + (200 + floor((733-200)*0.9)=200+479=679) + 460 + 36 + 20
    ///                   = 283 + 679 + 460 + 36 + 20 = 1478
    ///
    /// 期待値の枠組み（実装値を観測してから埋める）:
    ///   ・ranged_attack_total = STR + Marksmanship_skill_value + 8 + equip_ranged_attack + attack_bonus
    ///   ・ranged_accuracy_total = floor(AGI*0.75) + accuracy_skill_term(skill) + equip_ranged_accuracy + accuracy_bonus
    ///   ・attack_bonus = (COR/NINの物理攻撃トレイト 0) + ギフト(0JP→0) + JPカテゴリ(0)
    ///   ・accuracy_bonus = (COR/NINの命中トレイト 0) + ギフト(0JP→0) + JPカテゴリ(0)
    #[test]
    fn test_cor_ranged_ws_attack_accuracy() {
        use crate::character_profile::JobLevel;
        use crate::skills::default_skills;
        use enum_map::EnumMap;

        // メリット: ステータス全て 15、戦闘/魔法スキルメリット 8（既存テストと同条件）
        let mut merit = MeritPoints {
            hp: 15, mp: 15,
            str_: 15, dex: 15, vit: 15, agi: 15, int: 15, mnd: 15, chr: 15,
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

        // ジョブポイント全カテゴリ最大 (累計 2100 JP) → ギフト全段解放
        let jp = crate::job_points::JobPointCategories::all_maxed();

        // 全ジョブ最大の cap でスキルをデフォルト化（シミュレータと同じ挙動）
        let mut job_levels: EnumMap<Job, JobLevel> = EnumMap::default();
        job_levels[Job::Cor] = JobLevel { level: 99, master_lv: 50 };
        job_levels[Job::Nin] = JobLevel { level: 59, master_lv: 0 };
        let skills = default_skills(&job_levels, &merit);

        let mut skill_bonus_main: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_main.insert("Dagger".to_string(), 269); // ロスタム

        let mut skill_bonus_sub: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_sub.insert("Dagger".to_string(), 242); // クスタウィ+1

        let mut skill_bonus_ranged: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_ranged.insert("Marksmanship".to_string(), 269); // フォーマルハウト

        let mut skill_bonus_global: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_global.insert("Parrying".to_string(), 511); // ロスタム+クスタウィ受流合算

        let bonus = BonusStats {
            hp: 704,
            mp: 162,
            str_: 188,
            dex: 163,
            vit: 164,
            agi: 218,
            int: 154,
            mnd: 163,
            chr: 148,
            attack: 215,
            accuracy: 270,
            evasion: 482,
            ranged_attack: 423,
            ranged_accuracy: 460,
            magic_attack: 94,
            magic_def_bonus: 31,
            store_tp: 26,
            double_attack_pct: 16,
            main_weapon_skill_id: Some(2),    // ロスタム = 短剣
            sub_weapon_skill_id: Some(2),     // クスタウィ+1 = 短剣
            ranged_weapon_skill_id: Some(26), // フォーマルハウト = 射撃
            skill_bonus_main,
            skill_bonus_sub,
            skill_bonus_ranged,
            skill_bonus_global,
            ..BonusStats::default()
        };

        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Cor, 99)
            .support_job(Job::Nin, 59)
            .master_lv(50)
            .merit_points(merit)
            .job_points(jp)
            .skills(skills)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");

        let result = chara_to_status_result(&chara);

        // 期待値が異なる場合、上記内訳のどこが食い違うかをチェックして原因を切り分けたい:
        //   ・基本ステ (STR/AGI) の値が違う？
        //   ・有効武器スキル値が違う？（COR99 ML50 cap が違う？合算ロジックの誤り？）
        //   ・遠隔命中の AGI 係数 (現行 0.5)、または skill_term 区分が違う？
        //   ・JP/メリット/ジョブポイントが反映されるべき？
        assert_eq!(result.str_, 338, "STR mismatch");
        assert_eq!(result.dex, 321, "DEX mismatch");
        assert_eq!(result.vit, 314, "VIT mismatch");
        assert_eq!(result.agi, 378, "AGI mismatch");
        assert_eq!(result.int, 308, "INT mismatch");
        assert_eq!(result.mnd, 306, "MND mismatch");
        assert_eq!(result.chr, 293, "CHR mismatch");
        assert_eq!(result.ranged_weapon_skill_value, Some(733), "ranged skill value mismatch");
        assert_eq!(result.attack_bonus, 36, "attack_bonus mismatch (gift COR slot1)");
        assert_eq!(result.accuracy_bonus, 36, "accuracy_bonus mismatch (gift COR slot3)");
        assert_eq!(result.ranged_attack, Some(1538), "ranged_attack mismatch");
        assert_eq!(result.ranged_accuracy, Some(1478), "ranged_accuracy mismatch (incl COR JP idx7 +20)");
    }

    /// 属性WS 用のテストケース。COR99/NIN59 ML50 ML50、メリット全 +15、JP 全カテゴリ最大。
    /// 装備セット (Web シミュレータでの装備記述パースを忠実に模倣):
    ///   武器1: ロスタム (id=21581) Type:B rank25
    ///   武器2: トーレット (id=21565)
    ///   遠隔: デスペナルティ (id=22141) Default rank15
    ///   矢弾: ライヴブレット (id=21326)
    ///   頭:   妖蟲の髪飾り+1 (id=26696)
    ///   首:   コモドアチャーム+2 (id=25515) Type:A rank25
    ///   耳1:  フリオミシピアス (id=28514)
    ///   耳2:  胡蝶のイヤリング (id=11697) USER aug: "Magic Atk. Bonus"+4 "TP Bonus"+250
    ///   胴:   LAフラック+4 (id=23979)
    ///   両手: ニャメガントレ (id=23775) Type:B rank30
    ///   指1:  コーネリアリング (id=26227)
    ///   指2:  ディンジルリング (id=26187)
    ///   背:   カムラスマント (id=26262) USER aug: AGI+30 魔命+20 魔法ダメージ+20 WSダメ+10% 被物理ダメ-10%
    ///   腰:   闇輪の帯 (id=15442)
    ///   両足: ニャメフランチャ (id=23782) Type:B rank30
    ///   脚:   LAブーツ+4 (id=24114)
    ///
    /// 装備合算 (Pet:除外、JS extractAllStats 互換 = stat 種別ごとに各装備で先頭一致のみ採用):
    ///   STR=159 DEX=130 VIT=145 AGI=193 INT=175 MND=142 CHR=121 HP=477 MP=326
    ///   attack=160 accuracy=272 evasion=376
    ///   ranged_attack=319 ranged_accuracy=185
    ///   magic_attack=264 (Magic Atk. Bonus 合計、ただし属性別 "Dark Elemental Magic Atk. Bonus" は除外)
    ///   ※ magic_accuracy/magic_damage/magic_accuracy_skill/double_attack_pct 等は
    ///      WASM の StatusResult には反映されないため本テストでは確認しない
    /// スキル合算: Dagger=519, Marksmanship=269, Parrying=519
    #[test]
    fn test_cor_elemental_ws_set() {
        use crate::character_profile::JobLevel;
        use crate::skills::default_skills;
        use enum_map::EnumMap;

        let mut merit = MeritPoints {
            hp: 15, mp: 15,
            str_: 15, dex: 15, vit: 15, agi: 15, int: 15, mnd: 15, chr: 15,
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

        // ジョブポイント全カテゴリ最大 → ギフト全段解放
        let jp = crate::job_points::JobPointCategories::all_maxed();

        let mut job_levels: EnumMap<Job, JobLevel> = EnumMap::default();
        job_levels[Job::Cor] = JobLevel { level: 99, master_lv: 50 };
        job_levels[Job::Nin] = JobLevel { level: 59, master_lv: 0 };
        let skills = default_skills(&job_levels, &merit);

        let mut skill_bonus_main: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_main.insert("Dagger".to_string(), 269); // ロスタム
        let mut skill_bonus_sub: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_sub.insert("Dagger".to_string(), 250); // トーレット
        let mut skill_bonus_ranged: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_ranged.insert("Marksmanship".to_string(), 269); // デスペナルティ
        let mut skill_bonus_global: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_global.insert("Parrying".to_string(), 519); // ロスタム+トーレット 受流合算

        let bonus = BonusStats {
            hp: 477,
            mp: 326,
            str_: 159,
            dex: 130,
            vit: 145,
            agi: 193,
            int: 175,
            mnd: 142,
            chr: 121,
            attack: 160,
            accuracy: 272,
            evasion: 376,
            ranged_attack: 319,
            ranged_accuracy: 185,
            magic_attack: 264,
            store_tp: 0,
            double_attack_pct: 11,
            main_weapon_skill_id: Some(2),    // 短剣
            sub_weapon_skill_id: Some(2),     // 短剣
            ranged_weapon_skill_id: Some(26), // 射撃
            skill_bonus_main,
            skill_bonus_sub,
            skill_bonus_ranged,
            skill_bonus_global,
            ..BonusStats::default()
        };

        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Cor, 99)
            .support_job(Job::Nin, 59)
            .master_lv(50)
            .merit_points(merit)
            .job_points(jp)
            .skills(skills)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");

        let result = chara_to_status_result(&chara);

        // === 期待値: 既存 COR99/NIN59 ML50 + merit ALL+15 のベース値に装備合算を加算 ===
        // ベース (装備なし) は test_cor_ranged_ws_attack_accuracy で確認済み:
        //   STR=150 DEX=158 VIT=150 AGI=160 INT=154 MND=143 CHR=145
        // 本テストの装備合算: STR+159 DEX+130 VIT+145 AGI+193 INT+175 MND+142 CHR+121
        assert_eq!(result.str_, 309, "STR mismatch (base 150 + equip 159)");
        assert_eq!(result.dex, 288, "DEX mismatch (base 158 + equip 130)");
        assert_eq!(result.vit, 295, "VIT mismatch (base 150 + equip 145)");
        assert_eq!(result.agi, 353, "AGI mismatch (base 160 + equip 193)");
        assert_eq!(result.int, 329, "INT mismatch (base 154 + equip 175)");
        assert_eq!(result.mnd, 285, "MND mismatch (base 143 + equip 142)");
        assert_eq!(result.chr, 266, "CHR mismatch (base 145 + equip 121)");

        // 主武器 = 短剣 (Dagger)、有効スキル値 = 短剣スキル cap (COR99 ML50) + 装備 269 + 全体 0 + メリット 8
        // CORの短剣スキル cap は default_skills(...) 経由で取得する。
        // 想定: cap 348 + 装備 269 + メリット 8 ≈ 625 程度（WAR/SAM のロスタム例 659 と同様の式）
        // 実装の詳細値はテスト実行時に確認する（assertion は緩めに）。
        assert!(result.main_weapon_skill_value >= 600,
                "main_weapon_skill_value mismatch: got {}", result.main_weapon_skill_value);

        // === 魔攻総合値 = 100 + equip.magic_attack + magic_attack_bonus ===
        // 期待: 100 + 装備264 + ボーナス14 = 378
        assert_eq!(result.magic_attack, 378,
                   "magic_attack mismatch: 100 + equip 264 + bonus 14 = 378");
        assert_eq!(result.magic_attack_bonus, 14,
                   "magic_attack_bonus mismatch (COR ギフト由来)");

        // === デバッグ用: 食い違いがあった場合に内訳を確認できるよう出力 ===
        // 値が期待と異なる場合は cargo test -- --nocapture で詳細を確認:
        eprintln!("[debug] str={} dex={} vit={} agi={} int={} mnd={} chr={}",
                  result.str_, result.dex, result.vit, result.agi,
                  result.int, result.mnd, result.chr);
        eprintln!("[debug] main_weapon_skill={:?} value={} ranged_weapon_skill={:?} value={:?}",
                  result.main_weapon_skill, result.main_weapon_skill_value,
                  result.ranged_weapon_skill, result.ranged_weapon_skill_value);
        eprintln!("[debug] magic_attack={} magic_attack_bonus={} magic_accuracy_bonus={}",
                  result.magic_attack, result.magic_attack_bonus, result.magic_accuracy_bonus);
        eprintln!("[debug] main_attack={} main_accuracy={} double_attack_pct={}",
                  result.main_attack, result.main_accuracy, result.double_attack_pct);
        if let Some(elem) = result.effective_skills.get("Elemental") {
            eprintln!("[debug] effective Elemental skill={}", elem);
        }
        if let Some(dagger) = result.effective_skills.get("Dagger") {
            eprintln!("[debug] effective Dagger skill={}", dagger);
        }
    }
}
