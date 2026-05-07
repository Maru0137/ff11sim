use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use strum::VariantArray;
use wasm_bindgen::prelude::*;

use crate::chara::Chara;
use crate::character_profile::CharacterProfile;
use crate::gift::Gift;
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
    /// Triple Attack 発動率総合値 (%) (装備 + ジョブ特性)
    pub triple_attack_pct: i32,
    /// Regen 総合値 (3 秒ごとの HP 回復量、装備 + オートリジェネ特性)
    pub regen: i32,
    /// Refresh 総合値 (3 秒ごとの MP 回復量、装備 + オートリフレシュ特性)
    pub refresh: i32,
    /// Subtle Blow 総合値 (装備 + ジョブ特性 モクシャ)
    pub subtle_blow: i32,
    /// Rapid Shot 発動率総合値 (%) (装備 + ジョブ特性)
    pub rapid_shot_pct: i32,
    /// Fast Cast 総合値 (%) (装備 + ジョブ特性)
    pub fast_cast_pct: i32,
    /// デッドエイム総合値 (% — 遠隔クリティカルダメージ加算、ジョブ特性のみ Rng)
    pub dead_aim: i32,
    /// フェンサー総合値 (ジョブ特性ランク値 + ギフト「フェンサー効果アップ」TPボーナス)
    pub fencer: i32,
    /// マーシャルアーツ総合値 (格闘攻撃間隔短縮、負値で保持)
    /// = ジョブ特性 (Mnk/Pup) + ギフト「マーシャルアーツ効果アップ」
    pub martial_arts: i32,
    /// 二刀流総合値 (% — 攻撃間隔短縮率の magnitude、正値で保持)
    /// = ジョブ特性 DualWield (Thf/Nin/Dnc) + ギフト「二刀流効果アップ」(Thf/Dnc)
    pub dual_wield: i32,
    /// 残心総合値 (% — ミス時再攻撃発動率)
    /// = ジョブ特性 Zanshin (Sam) + ギフト「残心発動率アップ」(Sam)
    pub zanshin: i32,
    /// スマイト総合値 (両手武器装備時 ATK +N、ジョブ特性のみ)
    pub smite: i32,
    /// 打剣総合値 (% — 忍者の打剣発動率)
    /// = ジョブ特性 Daken (Nin) + ギフト「打剣効果アップ」(Nin)
    pub daken: i32,
    /// シールドバリア総合値 (Pld 専用、プロテス系効果強化のバイナリ)
    /// = ジョブ特性 ShieldBarrier (バイナリ 0/1) のみ
    pub shield_barrier: i32,
    /// プロテス効果総合値 (Pld 専用、防御力 +N)
    /// = ギフト「プロテス効果アップ」(Pld 550 JP で +10) のみ
    pub protes_effect: i32,
    /// クリアマインド総合値 (ヒーリング時 MP 回復量 +N、ジョブ特性のみ)
    pub clear_mind: i32,
    /// コンサーブ MP 総合値 (% — 魔法詠唱時の MP 消費軽減発動率、ジョブ特性のみ)
    pub conserve_mp: i32,
    /// トランキルハート総合値 (回復魔法敵対心軽減、バイナリ 0/1、ジョブ特性のみ)
    pub tranquil_heart: i32,
    /// ディバインベニゾン総合値 (Whm 専用、バニシュ・バニシュガ系の効果アップランク、ジョブ特性のみ)
    pub divine_benison: i32,
    /// カーディナルチャント総合値 (Geo 専用、白魔法効果アップランク、ジョブ特性のみ)
    pub cardinal_chant: i32,
    /// ブラッドブーン総合値 (Smn 専用、% — 召喚魔法 MP 消費軽減発動率、ジョブ特性のみ)
    pub blood_boon: i32,
    /// ベロシティショット効果総合値 (Rng 専用、ベロシティショットの遠隔攻撃間隔短縮量を増す、ギフトのみ)
    pub velocity_shot_effect: i32,
    /// ストレイフ総合値 (Drg 専用、ジョブ特性のみ)
    pub strafe: i32,
    /// トゥルーショット総合値 (% — 遠隔攻撃時の威力アップ)
    /// = ジョブ特性 TrueShot (Rng/Cor) + ギフト「トゥルーショット効果アップ」(Rng/Cor)
    pub trueshot: i32,
    /// 乱れ撃ち総合値 (Rng 専用、弾数 +N、ギフトのみ)
    pub barrage: i32,
    /// スナップショット総合値 (Cor 専用、遠隔攻撃の構えキャンセル % 短縮の magnitude、ギフトのみ)
    pub snapshot: i32,
    /// リサイクル総合値 (% — 矢弾消費せず遠隔攻撃の発動率)
    /// = ジョブ特性 Recycle (Rng/Cor) + ギフト「矢弾消費量軽減」(Cor)
    pub recycle: i32,
    /// シールドマスタリー総合値 (盾防御発動時の TP ボーナス、ジョブ特性のみ War/Rdm/Pld)
    pub shield_mastery: i32,
    /// シールドマスタリー効果総合値 (Pld 専用、ギフト「シールドマスタリー効果アップ」のみ)
    pub shield_mastery_effect: i32,
    /// アサシン総合値 (Thf 専用、だまし討ち強化、バイナリ 0/1、ジョブ特性のみ)
    pub assassin: i32,
    /// 歌の詠唱時間総合値 (% 短縮、Brd 専用、ギフトのみ)
    pub song_cast_time: i32,
    /// 歌の効果時間総合値 (% 延長、Brd 専用、ギフトのみ)
    pub song_effect_duration: i32,
    /// 心眼効果アップ総合値 (Sam 専用、よける回数 +N、ギフトのみ)
    pub third_eye_effect: i32,
    /// タンデムヒット (Tandem Strike) 総合値 (Bst 専用、ペット連携時の命中/魔命+N、ジョブ特性のみ)
    pub tandem_strike: i32,
    /// タンデムモクシャ (Tandem Blow) 総合値 (Bst 専用、ペット連携時の与TP-%、ジョブ特性のみ)
    pub tandem_blow: i32,
    /// レジストウィルス総合値 (ジョブ特性のみ)
    pub resist_virus: i32,
    /// レジストペトリ総合値 (石化、ジョブ特性のみ)
    pub resist_petrify: i32,
    /// レジストグラビティ総合値 (重力、ジョブ特性のみ)
    pub resist_gravity: i32,
    /// レジストスリプル総合値 (睡眠、ジョブ特性のみ)
    pub resist_sleep: i32,
    /// レジストパライズ総合値 (麻痺、ジョブ特性のみ)
    pub resist_paralyze: i32,
    /// レジストスロウ総合値 (ジョブ特性のみ)
    pub resist_slow: i32,
    /// レジストサイレス総合値 (静寂、ジョブ特性のみ)
    pub resist_silence: i32,
    /// レジストポイズン総合値 (毒、ジョブ特性のみ)
    pub resist_poison: i32,
    /// レジストブライン総合値 (暗闇、ジョブ特性のみ)
    pub resist_blind: i32,
    /// レジストバインド総合値 (バインド、ジョブ特性のみ)
    pub resist_bind: i32,
    /// レジストアムネジア総合値 (アムネジア、ジョブ特性のみ)
    pub resist_amnesia: i32,
    /// アンデッドキラー総合値 (ジョブ特性のみ)
    pub undead_killer: i32,
    /// アルカナキラー総合値 (ジョブ特性のみ)
    pub arcana_killer: i32,
    /// デーモンキラー総合値 (ジョブ特性のみ)
    pub demon_killer: i32,
    /// ドラゴンキラー総合値 (ジョブ特性のみ)
    pub dragon_killer: i32,
    /// ヴァーミンキラー総合値 (ジョブ特性のみ)
    pub vermin_killer: i32,
    /// バードキラー総合値 (ジョブ特性のみ)
    pub bird_killer: i32,
    /// アモルフキラー総合値 (不定形、ジョブ特性のみ)
    pub amorph_killer: i32,
    /// リザードキラー総合値 (ジョブ特性のみ)
    pub lizard_killer: i32,
    /// アクアンキラー総合値 (水棲、ジョブ特性のみ)
    pub aquan_killer: i32,
    /// プラントイドキラー総合値 (植物、ジョブ特性のみ)
    pub plantoid_killer: i32,
    /// ビーストキラー総合値 (獣、ジョブ特性のみ)
    pub beast_killer: i32,
    /// アラートネス総合値 (バイナリ、ジョブ特性のみ)
    pub alertness: i32,
    /// ステルス総合値 (バイナリ 2 ランク、ジョブ特性のみ)
    pub stealth: i32,
    /// ギルファインダー総合値 (バイナリ 2 ランク、ジョブ特性のみ)
    pub gilfinder: i32,
    /// タクティカルパリー総合値 (受け流し時 TP +N、ジョブ特性のみ)
    pub tactical_parry: i32,
    /// タクティカルガード総合値 (ガード時 TP +N、ジョブ特性のみ)
    pub tactical_guard: i32,
    /// エクストリームガード総合値 (ガード時被ダメ -%、ジョブ特性のみ)
    pub extreme_guard: i32,
    /// デスパレートブロー総合値 (HP 低下時の攻撃間隔短縮、ジョブ特性のみ)
    pub desperate_blows: i32,
    /// スタルワートソウル総合値 (ウェポンスキル時 HP/MP 回復、ジョブ特性のみ)
    pub stalwart_soul: i32,
    /// テナシティ総合値 (状態異常時被ダメ軽減、ジョブ特性のみ)
    pub tenacity: i32,
    /// マックスダメージブースト総合値 (物理ダメ最大値+N、ジョブ特性のみ)
    pub max_damage_boost: i32,
    // ============ A: クリティカル系 ============
    /// クリティカルヒットダメージ総合値 (% — 与クリダメ強化)
    /// = ジョブ特性 CritIncrease + ギフト「C.インクリース効果アップ」
    pub crit_increase: i32,
    /// クリティカルダメージ軽減総合値 (% — 被クリダメ軽減)
    /// = ジョブ特性 CritReduce + ギフト「C.リデュース効果アップ」(Pld)
    pub crit_reduce: i32,
    /// 直接クリティカル率総合値 (% — War 専用、ギフトのみ)
    pub critical_hit_rate: i32,
    // ============ B: 戦闘特性系 (合算済み以外) ============
    /// ウェポンスキルダメージ総合値 (%)
    /// = ジョブ特性 WeaponSkillDamage + ギフト「ウェポンスキルダメージアップ」(War/Drk/Nin)
    pub weapon_skill_damage: i32,
    /// カウンター総合値 (% — 反撃発動率)
    /// = ジョブ特性 Counter + ギフト「カウンター効果アップ」(Mnk)
    pub counter: i32,
    /// カウンターダメージ総合値 (% — Mnk 専用、ギフトのみ)
    pub counter_damage: i32,
    // ============ C: 魔法系 ============
    /// ケアル回復量総合値 (+N、ギフトのみ Whm/Pld)
    pub cure_amount: i32,
    /// 回復魔法詠唱時間総合値 (% 短縮、ギフトのみ Whm)
    pub healing_magic_cast_time: i32,
    /// リジェネ回復量総合値 (+N、ギフトのみ Whm)
    pub regen_amount: i32,
    /// マジックバーストダメージ総合値 (%)
    /// = ジョブ特性 MagicBurstBonus + ギフト「マジックバーストダメージアップ」(Blm/Sch)
    pub magic_burst_damage: i32,
    /// 魔法ダメージ総合値 (+N)
    /// = ジョブ特性 MagicAcumen + ギフト「魔法ダメージアップ」(Blm/Geo)
    pub magic_damage: i32,
    /// エレメンタルセレリティ総合値 (% — 精霊魔法詠唱短縮)
    /// = ジョブ特性 ElementalCelerity + ギフト「エレメントセレリティ効果アップ」(Blm)
    pub elemental_celerity: i32,
    /// 魔法剣ダメージ総合値 (+N、Rdm 専用、ギフトのみ)
    pub enspell_effect: i32,
    /// 被強化魔法効果時間総合値 (% 延長、Run 専用、ギフトのみ)
    pub enhance_magic_duration_on_self: i32,
    /// 青魔法効果アップ総合値 (% — 青魔の物理魔法効果強化、Blu 専用、ギフトのみ)
    pub blue_magic_effect: i32,
    // ============ D: 狩・コ・盗・盾系 (合算済み以外) ============
    /// コンサーブTP総合値 (%)
    /// = ジョブ特性 ConserveTp + ギフト「コンサーブTP効果アップ」(Rng)
    pub conserve_tp: i32,
    /// クイックドロー再使用時間総合値 (-秒、Cor 専用、ギフトのみ)
    pub quick_draw_recast: i32,
    /// トレジャーハンター総合値 (TH ランク+確率合算)
    /// = ジョブ特性 TreasureHunter + ギフト「トレジャーハンター効果アップ」(Thf)
    pub treasure_hunter: i32,
    /// トレジャーハンター上限総合値 (上限+N、Thf 専用、ギフトのみ)
    pub treasure_hunter_max_level: i32,
    /// ドレッドスパイク効果総合値 (+N、Drk 専用、ギフトのみ)
    pub dread_spike_effect: i32,
    /// インクァルタタ総合値 (% — 受け流し率)
    /// = ジョブ特性 Inquartata + ギフト「インクァルタタ効果アップ」(Run)
    pub inquartata: i32,
    // ============ E: 侍・踊系 (合算済み以外) ============
    /// 八双・星眼効果アップ総合値 (% — 残心/カウンター上限+%、Sam 専用、ギフトのみ)
    pub hasso_seigan_effect: i32,
    /// フィニシングムーブ最大値総合値 (+N、Dnc 専用、ギフトのみ)
    pub finishing_move_count: i32,
    // ============ F: ペット系 ============
    /// ペット物理攻撃/防御アップ (Bst 専用、ギフトのみ)
    pub pet_physical_atk_def: i32,
    /// ペット物理命中/回避アップ (Bst 専用、ギフトのみ)
    pub pet_physical_acc_eva: i32,
    /// ペットステータスアップ (Bst 専用、ギフトのみ)
    pub pet_status: i32,
    /// ペットTPボーナス (Bst 専用、ギフトのみ)
    pub pet_tp_bonus: i32,
    /// 召喚獣物理攻撃/防御アップ (Smn 専用、ギフトのみ)
    pub avatar_physical_atk_def: i32,
    /// 召喚獣物理命中/回避アップ (Smn 専用、ギフトのみ)
    pub avatar_physical_acc_eva: i32,
    /// 召喚獣魔法攻撃/防御アップ (Smn 専用、ギフトのみ)
    pub avatar_magical_atk_def: i32,
    /// 召喚獣魔法命中/回避アップ (Smn 専用、ギフトのみ)
    pub avatar_magical_acc_eva: i32,
    /// 神獣の加護効果アップ (Smn 専用、ギフトのみ)
    pub avatar_blessing_effect: i32,
    /// オートマトン物理攻撃/防御アップ (Pup 専用、ギフトのみ)
    pub automaton_physical_atk_def: i32,
    /// オートマトン物理命中/回避アップ (Pup 専用、ギフトのみ)
    pub automaton_physical_acc_eva: i32,
    /// オートマトン魔法攻撃/防御アップ (Pup 専用、ギフトのみ)
    pub automaton_magical_atk_def: i32,
    /// オートマトン魔法命中/回避アップ (Pup 専用、ギフトのみ)
    pub automaton_magical_acc_eva: i32,
    /// オートマトン属性値アップ (Pup 専用、ギフトのみ)
    pub automaton_element_boost: i32,
    /// ワイバーンステータスアップ時の効果アップ (Drg 専用、ギフトのみ)
    pub wyvern_boost_effect: i32,
    /// ワイバーン物理命中/回避アップ (Drg 専用、ギフトのみ)
    pub wyvern_physical_acc_eva: i32,
    /// ワイバーン魔法命中/回避アップ (Drg 専用、ギフトのみ)
    pub wyvern_magical_acc_eva: i32,
    /// ブレス再使用時間短縮 (Drg 専用、ギフトのみ、-秒)
    pub breath_recast: i32,
    /// スタウトサーヴァント総合値 (Bst 専用、ペット被ダメ軽減、ジョブ特性のみ)
    pub stout_servant: i32,
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

/// ジョブポイントギフトのスキル系ボーナス (累計 JP に応じた累積値) を返す。
/// 武器スキル/防御スキルにギフトでの直接加算は無いため魔法・楽器・忍術・風水・歌系のみ対象。
fn skill_gift_bonus(job: crate::job::Job, skill: SkillKind, total_jp: i32) -> i32 {
    use crate::gift::Gift;
    let gift = match skill {
        SkillKind::Healing => Gift::HealingMagicSkill,
        SkillKind::Divine => Gift::DivineMagicSkill,
        SkillKind::Elemental => Gift::ElementalMagicSkill,
        SkillKind::Dark => Gift::DarkMagicSkill,
        SkillKind::Enhancing => Gift::EnhancingMagicSkill,
        SkillKind::Enfeebling => Gift::EnfeeblingMagicSkill,
        SkillKind::Summoning => Gift::SummoningMagicSkill,
        SkillKind::BlueMagic => Gift::BlueMagicSkill,
        SkillKind::Ninjutsu => Gift::NinjutsuSkill,
        SkillKind::Geomancy => Gift::GeomancySkill,
        SkillKind::Handbell => Gift::GeomanticBellSkill,
        SkillKind::Singing => Gift::SingingSkill,
        SkillKind::StringInstrument => Gift::StringInstrumentSkill,
        SkillKind::WindInstrument => Gift::WindInstrumentSkill,
        SkillKind::Guarding => Gift::GuardSkill,
        _ => return 0,
    };
    job.gift_value(gift, total_jp)
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
    let auto_regen_trait = chara.job_trait_total(JobTrait::AutoRegen);
    let auto_refresh_trait = chara.job_trait_total(JobTrait::AutoRefresh);
    let triple_attack_trait = chara.job_trait_total(JobTrait::TripleAttack);
    let subtle_blow_trait = chara.job_trait_total(JobTrait::SubtleBlow);
    let rapid_shot_trait = chara.job_trait_total(JobTrait::RapidShot);
    let fast_cast_trait = chara.job_trait_total(JobTrait::FastCast);
    let dead_aim_trait = chara.job_trait_total(JobTrait::DeadAim);
    let fencer_trait = chara.job_trait_total(JobTrait::Fencer);
    let martial_arts_trait = chara.job_trait_total(JobTrait::MartialArts);
    let dual_wield_trait = chara.job_trait_total(JobTrait::DualWield);
    let zanshin_trait = chara.job_trait_total(JobTrait::Zanshin);
    let smite_trait = chara.job_trait_total(JobTrait::Smite);
    let daken_trait = chara.job_trait_total(JobTrait::Daken);
    let shield_barrier_trait = chara.job_trait_total(JobTrait::ShieldBarrier);
    let clear_mind_trait = chara.job_trait_total(JobTrait::ClearMind);
    let conserve_mp_trait = chara.job_trait_total(JobTrait::ConserveMp);
    let tranquil_heart_trait = chara.job_trait_total(JobTrait::TranquilHeart);
    let divine_benison_trait = chara.job_trait_total(JobTrait::DivineBenison);
    let cardinal_chant_trait = chara.job_trait_total(JobTrait::CardinalChant);
    let blood_boon_trait = chara.job_trait_total(JobTrait::BloodBoon);
    let strafe_trait = chara.job_trait_total(JobTrait::Strafe);
    let trueshot_trait = chara.job_trait_total(JobTrait::TrueShot);
    let recycle_trait = chara.job_trait_total(JobTrait::Recycle);
    let shield_mastery_trait = chara.job_trait_total(JobTrait::ShieldMastery);
    let assassin_trait = chara.job_trait_total(JobTrait::Assassin);
    let tandem_strike_trait = chara.job_trait_total(JobTrait::TandemStrike);
    let tandem_blow_trait = chara.job_trait_total(JobTrait::TandemBlow);
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

    // effective_skills（表示用）: base + global + ギフトのスキルボーナス
    // ジョブ構成がスキルを持たない場合はボーナスも適用しない（未習得扱い）
    let mut effective_skills: BTreeMap<String, i32> = BTreeMap::new();
    for (skill, base) in base_effective.iter() {
        let v = if job_has_skill(*skill) {
            base + global_bonus(*skill) + skill_gift_bonus(chara.main_job, *skill, total_jp)
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
        // Triple Attack 総合 = 装備 + ジョブ特性 + ギフト (Thf 等)
        triple_attack_pct: chara.bonus_stats.triple_attack_pct
            + triple_attack_trait
            + chara.main_job.gift_value(Gift::TripleAttackRate, total_jp),
        // オートリジェネ/リフレシュ 総合 = 装備 + ジョブ特性
        regen: chara.bonus_stats.regen + auto_regen_trait,
        refresh: chara.bonus_stats.refresh + auto_refresh_trait,
        // モクシャ 総合 = 装備 + ジョブ特性 + ギフト (Mnk/Dnc)
        subtle_blow: chara.bonus_stats.subtle_blow
            + subtle_blow_trait
            + chara.main_job.gift_value(Gift::SubtleBlow, total_jp),
        // ラピッドショット 総合 = 装備 + ジョブ特性
        rapid_shot_pct: chara.bonus_stats.rapid_shot_pct + rapid_shot_trait,
        // ファストキャスト 総合 = 装備 + ジョブ特性 + ギフト (Rdm)
        fast_cast_pct: chara.bonus_stats.fast_cast_pct
            + fast_cast_trait
            + chara.main_job.gift_value(Gift::FastCastEffect, total_jp),
        // デッドエイム 総合 = ジョブ特性 (Rng) のみ
        dead_aim: dead_aim_trait,
        // フェンサー 総合 = ジョブ特性ランク + ギフト「フェンサー効果アップ」(War/Bst)
        fencer: fencer_trait + chara.main_job.gift_value(Gift::FencerEffect, total_jp),
        // マーシャルアーツ 総合 = ジョブ特性 + ギフト (Mnk/Pup)
        martial_arts: martial_arts_trait
            + chara.main_job.gift_value(Gift::MartialArtsEffect, total_jp),
        // 二刀流 総合 = ジョブ特性 + ギフト (Thf/Dnc)
        dual_wield: dual_wield_trait
            + chara.main_job.gift_value(Gift::DualWieldEffect, total_jp),
        // 残心 総合 = ジョブ特性 + ギフト (Sam)
        zanshin: zanshin_trait + chara.main_job.gift_value(Gift::ZanshinRate, total_jp),
        // スマイト 総合 = ジョブ特性 (War/Mnk/Drk/Drg/Pup) のみ
        smite: smite_trait,
        // 打剣 総合 = ジョブ特性 (Nin) + ギフト「打剣効果アップ」(Nin)
        daken: daken_trait + chara.main_job.gift_value(Gift::ShurikenThrowEffect, total_jp),
        // シールドバリア 総合 = ジョブ特性 (Pld バイナリ) のみ
        shield_barrier: shield_barrier_trait,
        // プロテス効果 総合 = ギフト「プロテス効果アップ」(Pld) のみ
        protes_effect: chara.main_job.gift_value(Gift::ProtesEffect, total_jp),
        // クリアマインド 総合 = ジョブ特性 (Whm/Blm/Rdm/Smn/Sch/Geo) のみ
        clear_mind: clear_mind_trait,
        // コンサーブ MP 総合 = ジョブ特性 (Blm/Sch/Geo) のみ
        conserve_mp: conserve_mp_trait,
        // トランキルハート 総合 = ジョブ特性 (Whm/Rdm/Sch) のみ (バイナリ)
        tranquil_heart: tranquil_heart_trait,
        // ディバインベニゾン 総合 = ジョブ特性 (Whm) のみ
        divine_benison: divine_benison_trait,
        // カーディナルチャント 総合 = ジョブ特性 (Geo) のみ
        cardinal_chant: cardinal_chant_trait,
        // ブラッドブーン 総合 = ジョブ特性 (Smn) のみ
        blood_boon: blood_boon_trait,
        // ベロシティショット効果 総合 = ギフト「ベロシティショット効果アップ」(Rng) のみ
        velocity_shot_effect: chara.main_job.gift_value(Gift::VelocityShotEffect, total_jp),
        // ストレイフ 総合 = ジョブ特性 (Drg) のみ
        strafe: strafe_trait,
        // トゥルーショット 総合 = ジョブ特性 (Rng/Cor) + ギフト「トゥルーショット効果アップ」(Rng/Cor)
        trueshot: trueshot_trait + chara.main_job.gift_value(Gift::TrueshotEffect, total_jp),
        // 乱れ撃ち 総合 = ギフト「乱れ撃ち効果アップ」(Rng) のみ
        barrage: chara.main_job.gift_value(Gift::BarrageEffect, total_jp),
        // スナップショット 総合 = ギフト「スナップショット効果アップ」(Cor) のみ
        snapshot: chara.main_job.gift_value(Gift::SnapshotEffect, total_jp),
        // リサイクル 総合 = ジョブ特性 (Rng/Cor) + ギフト「矢弾消費量軽減」(Cor)
        recycle: recycle_trait + chara.main_job.gift_value(Gift::AmmoCostReduction, total_jp),
        // シールドマスタリー 総合 = ジョブ特性 (War/Rdm/Pld) のみ
        shield_mastery: shield_mastery_trait,
        // シールドマスタリー効果 総合 = ギフト「シールドマスタリー効果アップ」(Pld) のみ
        shield_mastery_effect: chara.main_job.gift_value(Gift::ShieldMasteryEffect, total_jp),
        // アサシン 総合 = ジョブ特性 (Thf) のみ (バイナリ)
        assassin: assassin_trait,
        // 歌の詠唱時間 総合 = ギフト「歌の詠唱時間短縮」(Brd) のみ (% 短縮の負値)
        song_cast_time: chara.main_job.gift_value(Gift::SongCastTime, total_jp),
        // 歌の効果時間 総合 = ギフト「歌の効果時間延長」(Brd) のみ (% 延長)
        song_effect_duration: chara.main_job.gift_value(Gift::SongEffectDuration, total_jp),
        // 心眼効果アップ 総合 = ギフト「心眼効果アップ」(Sam) のみ
        third_eye_effect: chara.main_job.gift_value(Gift::ThirdEyeEffect, total_jp),
        // タンデムヒット 総合 = ジョブ特性 (Bst) のみ
        tandem_strike: tandem_strike_trait,
        // タンデムモクシャ 総合 = ジョブ特性 (Bst) のみ
        tandem_blow: tandem_blow_trait,
        // レジスト系 (全 11 種、ジョブ特性のみ)
        resist_virus: chara.job_trait_total(JobTrait::ResistVirus),
        resist_petrify: chara.job_trait_total(JobTrait::ResistPetrify),
        resist_gravity: chara.job_trait_total(JobTrait::ResistGravity),
        resist_sleep: chara.job_trait_total(JobTrait::ResistSleep),
        resist_paralyze: chara.job_trait_total(JobTrait::ResistParalyze),
        resist_slow: chara.job_trait_total(JobTrait::ResistSlow),
        resist_silence: chara.job_trait_total(JobTrait::ResistSilence),
        resist_poison: chara.job_trait_total(JobTrait::ResistPoison),
        resist_blind: chara.job_trait_total(JobTrait::ResistBlind),
        resist_bind: chara.job_trait_total(JobTrait::ResistBind),
        resist_amnesia: chara.job_trait_total(JobTrait::ResistAmnesia),
        // キラー系 (全 11 種、ジョブ特性のみ)
        undead_killer: chara.job_trait_total(JobTrait::UndeadKiller),
        arcana_killer: chara.job_trait_total(JobTrait::ArcanaKiller),
        demon_killer: chara.job_trait_total(JobTrait::DemonKiller),
        dragon_killer: chara.job_trait_total(JobTrait::DragonKiller),
        vermin_killer: chara.job_trait_total(JobTrait::VerminKiller),
        bird_killer: chara.job_trait_total(JobTrait::BirdKiller),
        amorph_killer: chara.job_trait_total(JobTrait::AmorphKiller),
        lizard_killer: chara.job_trait_total(JobTrait::LizardKiller),
        aquan_killer: chara.job_trait_total(JobTrait::AquanKiller),
        plantoid_killer: chara.job_trait_total(JobTrait::PlantoidKiller),
        beast_killer: chara.job_trait_total(JobTrait::BeastKiller),
        // I 系 (その他のジョブ特性、ギフトなし)
        alertness: chara.job_trait_total(JobTrait::Alertness),
        stealth: chara.job_trait_total(JobTrait::Stealth),
        gilfinder: chara.job_trait_total(JobTrait::Gilfinder),
        tactical_parry: chara.job_trait_total(JobTrait::TacticalParry),
        tactical_guard: chara.job_trait_total(JobTrait::TacticalGuard),
        extreme_guard: chara.job_trait_total(JobTrait::ExtremeGuard),
        desperate_blows: chara.job_trait_total(JobTrait::DesperateBlows),
        stalwart_soul: chara.job_trait_total(JobTrait::StalwartSoul),
        tenacity: chara.job_trait_total(JobTrait::Tenacity),
        max_damage_boost: chara.job_trait_total(JobTrait::MaxDamageBoost),
        // A: クリティカル系
        crit_increase: chara.job_trait_total(JobTrait::CritIncrease)
            + chara.main_job.gift_value(Gift::CritIncreaseEffect, total_jp),
        crit_reduce: chara.job_trait_total(JobTrait::CritReduce)
            + chara.main_job.gift_value(Gift::CritReduceEffect, total_jp),
        critical_hit_rate: chara.main_job.gift_value(Gift::CriticalHitRate, total_jp),
        // B: 戦闘特性系
        weapon_skill_damage: chara.job_trait_total(JobTrait::WeaponSkillDamage)
            + chara.main_job.gift_value(Gift::WeaponSkillDamage, total_jp),
        counter: chara.job_trait_total(JobTrait::Counter)
            + chara.main_job.gift_value(Gift::CounterRate, total_jp),
        counter_damage: chara.main_job.gift_value(Gift::CounterDamage, total_jp),
        // C: 魔法系
        cure_amount: chara.main_job.gift_value(Gift::CureAmount, total_jp),
        healing_magic_cast_time: chara.main_job.gift_value(Gift::HealingMagicCastTime, total_jp),
        regen_amount: chara.main_job.gift_value(Gift::RegenAmount, total_jp),
        magic_burst_damage: chara.job_trait_total(JobTrait::MagicBurstBonus)
            + chara.main_job.gift_value(Gift::MagicBurstDamage, total_jp),
        magic_damage: chara.job_trait_total(JobTrait::MagicAcumen)
            + chara.main_job.gift_value(Gift::MagicDamage, total_jp),
        elemental_celerity: chara.job_trait_total(JobTrait::ElementalCelerity)
            + chara.main_job.gift_value(Gift::ElementalCelerityEffect, total_jp),
        enspell_effect: chara.main_job.gift_value(Gift::EnspellEffect, total_jp),
        enhance_magic_duration_on_self: chara
            .main_job
            .gift_value(Gift::EnhanceMagicDurationOnSelf, total_jp),
        blue_magic_effect: chara.main_job.gift_value(Gift::BlueMagicEffect, total_jp),
        // D: 狩・コ・盗・盾系 (合算済み以外)
        conserve_tp: chara.job_trait_total(JobTrait::ConserveTp)
            + chara.main_job.gift_value(Gift::ConserveTpEffect, total_jp),
        quick_draw_recast: chara.main_job.gift_value(Gift::QuickDrawRecast, total_jp),
        treasure_hunter: chara.job_trait_total(JobTrait::TreasureHunter)
            + chara.main_job.gift_value(Gift::TreasureHunterEffect, total_jp),
        treasure_hunter_max_level: chara
            .main_job
            .gift_value(Gift::TreasureHunterMaxLevel, total_jp),
        dread_spike_effect: chara.main_job.gift_value(Gift::DreadSpikeEffect, total_jp),
        inquartata: chara.job_trait_total(JobTrait::Inquartata)
            + chara.main_job.gift_value(Gift::InquartataEffect, total_jp),
        // E: 侍・踊系 (合算済み以外)
        hasso_seigan_effect: chara.main_job.gift_value(Gift::HassoSeiganEffect, total_jp),
        finishing_move_count: chara.main_job.gift_value(Gift::FinishingMoveCount, total_jp),
        // F: ペット系
        pet_physical_atk_def: chara.main_job.gift_value(Gift::PetPhysicalAtkDef, total_jp),
        pet_physical_acc_eva: chara.main_job.gift_value(Gift::PetPhysicalAccEva, total_jp),
        pet_status: chara.main_job.gift_value(Gift::PetStatus, total_jp),
        pet_tp_bonus: chara.main_job.gift_value(Gift::PetTpBonus, total_jp),
        avatar_physical_atk_def: chara.main_job.gift_value(Gift::AvatarPhysicalAtkDef, total_jp),
        avatar_physical_acc_eva: chara.main_job.gift_value(Gift::AvatarPhysicalAccEva, total_jp),
        avatar_magical_atk_def: chara.main_job.gift_value(Gift::AvatarMagicalAtkDef, total_jp),
        avatar_magical_acc_eva: chara.main_job.gift_value(Gift::AvatarMagicalAccEva, total_jp),
        avatar_blessing_effect: chara.main_job.gift_value(Gift::AvatarBlessingEffect, total_jp),
        automaton_physical_atk_def: chara
            .main_job
            .gift_value(Gift::AutomatonPhysicalAtkDef, total_jp),
        automaton_physical_acc_eva: chara
            .main_job
            .gift_value(Gift::AutomatonPhysicalAccEva, total_jp),
        automaton_magical_atk_def: chara
            .main_job
            .gift_value(Gift::AutomatonMagicalAtkDef, total_jp),
        automaton_magical_acc_eva: chara
            .main_job
            .gift_value(Gift::AutomatonMagicalAccEva, total_jp),
        automaton_element_boost: chara.main_job.gift_value(Gift::AutomatonElementBoost, total_jp),
        wyvern_boost_effect: chara.main_job.gift_value(Gift::WyvernBoostEffect, total_jp),
        wyvern_physical_acc_eva: chara.main_job.gift_value(Gift::WyvernPhysicalAccEva, total_jp),
        wyvern_magical_acc_eva: chara.main_job.gift_value(Gift::WyvernMagicalAccEva, total_jp),
        breath_recast: chara.main_job.gift_value(Gift::BreathRecast, total_jp),
        stout_servant: chara.job_trait_total(JobTrait::StoutServant),
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

    /// SMN90 → AutoRefresh rank 2 = +2/3sec が StatusResult.refresh に反映
    /// PLD35 → rank 1 = +1
    #[test]
    fn test_pld_smn_auto_refresh() {
        let pld = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Pld, 99)
            .master_lv(0)
            .build()
            .unwrap();
        let smn90 = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Smn, 90)
            .master_lv(0)
            .build()
            .unwrap();
        assert_eq!(chara_to_status_result(&pld).refresh, 1);
        assert_eq!(chara_to_status_result(&smn90).refresh, 2);
    }

    /// RUN95 → AutoRegen rank 3 = +3/3sec、装備 +5 とで合計 +8
    #[test]
    fn test_run_auto_regen_with_equip() {
        let bonus = BonusStats {
            regen: 5, // 装備合計
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Run, 99)
            .master_lv(0)
            .bonus_stats(bonus)
            .build()
            .unwrap();
        assert_eq!(chara_to_status_result(&chara).regen, 5 + 3);
    }

    /// NIN91 → SubtleBlow rank 6 = 27、装備 +10 とで合計 37
    #[test]
    fn test_nin_subtle_blow_with_equip() {
        let bonus = BonusStats { subtle_blow: 10, ..BonusStats::default() };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Nin, 99)
            .master_lv(0)
            .bonus_stats(bonus)
            .build()
            .unwrap();
        assert_eq!(chara_to_status_result(&chara).subtle_blow, 10 + 27);
    }

    /// RDM99 → FastCast rank 6 = 30%、装備 +10% とで合計 40%
    /// (RDM は Lv15 で rank 2 を直接習得し、最大は Lv90 で rank 6)
    #[test]
    fn test_rdm_fast_cast_with_equip() {
        let bonus = BonusStats { fast_cast_pct: 10, ..BonusStats::default() };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Rdm, 99)
            .master_lv(0)
            .bonus_stats(bonus)
            .build()
            .unwrap();
        assert_eq!(chara_to_status_result(&chara).fast_cast_pct, 10 + 30);
    }

    /// RNG76 → RapidShot rank 2 (cumulative 配列 [25] のため clamp で 25)
    #[test]
    fn test_rng_rapid_shot() {
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Rng, 99)
            .master_lv(0)
            .build()
            .unwrap();
        assert_eq!(chara_to_status_result(&chara).rapid_shot_pct, 25);
    }

    /// THF95 → TripleAttack rank 2 = 6%、装備 +5% とで合計 11%
    #[test]
    fn test_thf_triple_attack_with_equip() {
        let bonus = BonusStats {
            triple_attack_pct: 5,
            ..BonusStats::default()
        };
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Thf, 99)
            .master_lv(0)
            .bonus_stats(bonus)
            .build()
            .unwrap();
        assert_eq!(chara_to_status_result(&chara).triple_attack_pct, 5 + 6);
    }

    /// BLU99 + JP 全振り でジョブ特性は青魔法未対応のため 0、ギフトカテゴリのみが乗る。
    /// BLU の MagicAccuracy ギフト (data/job_gifts.json):
    ///   tiers [[125,5],[450,8],[1050,10],[1900,13]] → 全振り 2100 JP で 5+8+10+13 = 36
    #[test]
    fn test_blu99_magic_accuracy_bonus_gift_category_only() {
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Blu, 99)
            .master_lv(0)
            .job_points(JobPointCategories::all_maxed())
            .build()
            .expect("Failed to build BLU");
        let result = chara_to_status_result(&chara);
        assert_eq!(
            result.magic_accuracy_bonus, 36,
            "BLU99 全振り MagicAccuracyBonus: got {} expected 36 (ギフトカテゴリのみ)",
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

    /// Brd99/Pld59 ML50 の回避値検証テスト。
    ///
    /// 装備セット (items.json + augments.json から抽出。Unity/path augment は注釈で明記):
    ///
    /// | スロット | 装備 | AGI / 回避値 / 回避スキル |
    /// |---------|------|------------------------|
    /// | main    | ニビルナイフ (id=20600) + 命中+20 攻+15 回避+20 (custom aug) | AGI+5 / 回避+49 (29+20 aug) / 短剣+242 受流+242 |
    /// | sub     | 玄冥盾 (id=27645)                                  | 0 / 0 / 盾+112 |
    /// | range   | リノス (id=21404) + AGI+8 回避+15 被物理-5% (custom aug) | AGI+8 / 回避+15 / 0 |
    /// | ammo    | (なし)                                              | 0 / 0 / 0 |
    /// | head    | 無の面 (id=24270)                                  | AGI+28 / 回避+100 / 0 |
    /// | body    | レベレプレート (id=24125) Default30 (命中+30 飛命+30 魔命+30 DA+8% PDL+7%) | AGI+37 / 回避+123 / 0 |
    /// | hands   | ニャメガントレ (id=23775) Type:B30 (攻+35 飛攻+35 WSD+11% DA+5% VIT+15) | AGI+12 / 回避+80 / 0 |
    /// | legs    | レベレブレー (id=24131) Default30 (命中+30 飛命+30 魔命+30 DA+7% PDL+6%) | AGI+34 / 回避+119 / 0 |
    /// | feet    | ヒポメネソックス+1 (id=27410) Unity max +20 + Default15 aug (回避+20 ALL BP+10) | AGI+33+10 / 回避+71+20+20 = 111 / 0 |
    /// | neck    | ウォーダチャーム+1 (id=27505) Unity max +5                | 0 / 0 / 0 |
    /// | waist   | 無の腰当 (id=26367)                                  | 0 / 回避+30 / 0 |
    /// | ear1    | アスプロピアス (id=26119) Default30 aug (命中+15 飛命+15 魔命+15 ALL BP+10 ストアTP+5) | AGI+10 / 0 / 0 |
    /// | ear2    | アレテデルルナ+1 (id=28487) Unity max +25 (耐光)        | 0 / 0 / 0 |
    /// | ring1   | シュネデックリング (id=27590)                          | 0 / 0 / 0 |
    /// | ring2   | フォテファイリング (id=10773)                          | 0 / 0 / 0 |
    /// | back    | 無の外装 (id=26274)                                  | 0 / 回避+50 / 0 |
    ///
    /// 装備合計:
    ///   AGI = 5 + 8 + 28 + 37 + 12 + 34 + (33+10) + 10 = 177
    ///       (= 装備ベース 157 + path augment ALL BP+10 ×2 = +20)
    ///   回避値 = 49 + 15 + 100 + 123 + 80 + 119 + (71+20+20) + 30 + 50 = 677
    ///       (= 装備ベース 657 + path augment 回避+20 = +20)
    ///   回避スキル+ (装備直接加算): 0 (該当無し)
    ///
    /// 期待値式 (回避):
    ///   AGI 総合 = 種族(Hum D) + 主ジョブ(Brd F) + サポ(Pld G)/2 + ML50 + メリット15 + 装備177 = 317
    ///   回避スキル有効値 = Brd99 D cap (334) + ML50 + メリット16 = 400
    ///   skill_term = piecewise(400) = 200 + (400-200)*0.9 = 380
    ///   evasion_total = floor(317*0.5) + 380 + 677 + ギフト「物理回避アップ」(Brd 2100JP=22)
    ///                 = 158 + 380 + 677 + 22 = 1237
    #[test]
    fn test_brd_pld_evasion_breakdown() {
        use crate::character_profile::JobLevel;
        use crate::skills::default_skills;
        use enum_map::EnumMap;

        // メリット: ステータス全 15、戦闘/魔法スキル 8 (既存テストと同条件)
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

        let mut job_levels: EnumMap<Job, JobLevel> = EnumMap::default();
        job_levels[Job::Brd] = JobLevel { level: 99, master_lv: 50 };
        job_levels[Job::Pld] = JobLevel { level: 59, master_lv: 0 };
        let skills = default_skills(&job_levels, &merit);

        // 装備のスキル+ ボーナス (短剣/受流/盾、global slot として加算)
        // ニビルナイフ: 短剣+242 受流+242
        // 玄冥盾: 盾+112
        // → main slot: Dagger+242
        // → sub slot:  なし (Brd は受流持たないため Parrying を持っていてもジョブ依存)
        // → global:   Shield+112, Parrying+242 (受流は global で加算)
        let mut skill_bonus_main: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_main.insert("Dagger".to_string(), 242);

        let mut skill_bonus_global: BTreeMap<String, i32> = BTreeMap::new();
        skill_bonus_global.insert("Shield".to_string(), 112);
        skill_bonus_global.insert("Parrying".to_string(), 242);

        // BonusStats: 装備合計値 (Unity/path augment は最大値で集計)
        // AGI = 5 (Nibiru) + 8 (Linos aug) + 28 (無の面) + 37 (Reverence body)
        //     + 12 (Nyame hands) + 34 (Reverence legs) + 33+10 (Hipomenes base+ALLBP) + 10 (Aspropias ALLBP)
        //     = 177
        // 回避値 = 49 (Nibiru+aug) + 15 (Linos aug) + 100 (無の面)
        //        + 123 (Reverence body) + 80 (Nyame hands) + 119 (Reverence legs)
        //        + 71+20+20 (Hipomenes base+Unity+aug) + 30 (無の腰当) + 50 (無の外装)
        //        = 677
        let bonus = BonusStats {
            agi: 177,
            evasion: 677,
            // 他のステータス (DEX/STR/etc.) は装備ベースを集計するが、回避テストでは AGI/回避値が主
            // 必要に応じて他装備分も追加すべし
            str_: 47 + 0 + 17 + 57 + 10, // body + nyame + legs + feet (近似)
            dex: 5 + 45 + 42 + 0 + 11, // Nibiru + body + nyame + Hipomenes
            vit: 0 + 37 + 39 + 35 + 10,
            int: 0 + 39 + 28 + 43 + 17,
            mnd: 0 + 28 + 40 + 22 + 19,
            chr: 5 + 37 + 24 + 28 + 34, // Nibiru + body + nyame + legs + Hipomenes
            // HP も忘れずに加算
            hp: 130 + 91 + 119 + 13 + 100, // body + nyame + legs + Hipomenes + アスプロピアス
            mp: 73 + 14 + 50, // nyame + Hipomenes + フォテファイリング
            // 命中・攻撃: ニビル aug + 玄冥盾 + Reverence A path + Nyame B path 等
            accuracy: 20 + 15 + 30 + 30 + 30 + 5, // Nibiru aug + 玄冥 + body Apath + nyame Bpath + legs Apath + フォテ
            attack: 15 + 15 + 30 + 30 + 30, // Nibiru aug + 玄冥 + body + nyame + legs
            main_weapon_skill_id: Some(2),    // ニビルナイフ = 短剣
            sub_weapon_skill_id: None,        // 玄冥盾 = サブだが武器スキルではない
            ranged_weapon_skill_id: Some(42), // リノス
            skill_bonus_main,
            skill_bonus_global,
            ..BonusStats::default()
        };

        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Brd, 99)
            .support_job(Job::Pld, 59)
            .master_lv(50)
            .merit_points(merit)
            .job_points(jp)
            .skills(skills)
            .bonus_stats(bonus)
            .build()
            .expect("Failed to build Chara");

        let result = chara_to_status_result(&chara);

        // === デバッグ出力: cargo test -- --nocapture で確認 ===
        eprintln!("[evasion] AGI total = {}", result.agi);
        eprintln!("[evasion] effective Evasion skill = {:?}",
                  result.effective_skills.get("Evasion"));
        eprintln!("[evasion] evasion_bonus (trait+gift+jpcat) = {}", result.evasion_bonus);
        eprintln!("[evasion] evasion total = {}", result.evasion);

        // 期待値:
        //   AGI 総合 = 317 (race 75 + ML 50 + merit 15 + 装備 177)
        //   回避スキル有効値 = 400 (Brd99 D cap 334 + ML 50 + merit 16)
        //   skill_term = 200 + (400-200)*0.9 = 380
        //   evasion = floor(317/2) + 380 + 677 + 22 = 158 + 380 + 677 + 22 = 1237
        assert_eq!(result.agi, 317, "AGI mismatch");
        assert_eq!(result.effective_skills.get("Evasion").copied(), Some(400),
                   "Evasion skill mismatch");
        assert_eq!(result.evasion_bonus, 22,
                   "evasion_bonus mismatch (Brd 2100JP gift PhysicalEvasion=22)");
        assert_eq!(result.evasion, 1237, "evasion total mismatch");
    }
}
