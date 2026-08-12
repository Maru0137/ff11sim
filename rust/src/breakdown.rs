//! ステータス / プロパティ値のソース別内訳 (docs/adr/0016)。
//!
//! `chara_to_status_result` (wasm.rs) と同じソース分解を「行」として公開する。
//! 集計ロジック本体は `Chara::status_parts` / `Chara::job_trait_breakdown` /
//! `combat_skills` に委譲し、二重実装によるドリフトを防ぐ。「基礎」行
//! (レベル・ステータス・スキル由来の項) は剰余ではなく解析的に計算し、
//! 「Σ行 + 装備 == 総合値」の恒等式はテストで担保する。
//!
//! 装備スロット別の寄与は JS 側 (equip-bonuses の per_slot_stats) が持つため、
//! ここにはキャラクター由来の行のみを含む。

use std::collections::BTreeMap;

use serde::Serialize;

use crate::chara::Chara;
use crate::gift::Gift;
use crate::job::{Job, JobTrait};
use crate::job_points::{calc_gift_bonuses, calc_jp_category_bonuses, calc_war_da_gift_bonus};
use crate::status::{
    StatusKind, calc_accuracy, calc_defense, calc_evasion, calc_magic_attack, calc_magic_defense,
    calc_main_attack, calc_ranged_accuracy, calc_ranged_attack,
};
use crate::wasm::combat_skills;

pub const SRC_BASE: &str = "base";
pub const SRC_RACE: &str = "race";
pub const SRC_MAIN_JOB: &str = "main_job";
pub const SRC_SUPPORT_JOB: &str = "support_job";
pub const SRC_MERIT: &str = "merit";
pub const SRC_JOB_POINTS: &str = "job_points";
pub const SRC_GIFT: &str = "gift";
pub const SRC_MASTER_LEVEL: &str = "master_level";

/// キャラクター由来のソース別内訳。
/// rows: ソース → 列キー → 値。値 0 のセルは含まない (疎)。
/// 列キーは UI の列と 1:1 (例: gift の physical_attack は "attack" に写像済み)。
/// race / main_job / support_job の基本ステは f32 由来の小数値を含む
/// (ゲーム仕様では 3 ソース合算後に切り捨て)。
#[derive(Debug, Default, Serialize)]
pub struct StatusBreakdown {
    pub rows: BTreeMap<String, BTreeMap<String, f64>>,
}

#[derive(Default)]
struct Builder {
    rows: BTreeMap<String, BTreeMap<String, f64>>,
}

impl Builder {
    fn add(&mut self, src: &str, col: &str, v: f64) {
        if v != 0.0 {
            *self
                .rows
                .entry(src.to_string())
                .or_default()
                .entry(col.to_string())
                .or_default() += v;
        }
    }

    /// ジョブ特性を採用側 (メイン/サポート) の行に振り分けて加算する
    fn add_trait(&mut self, chara: &Chara, col: &str, t: JobTrait) {
        let tb = chara.job_trait_breakdown(t);
        self.add(SRC_MAIN_JOB, col, tb.adopted_main() as f64);
        self.add(SRC_SUPPORT_JOB, col, tb.adopted_support() as f64);
    }
}

/// キャラクター由来 (装備以外) のソース別内訳を計算する。
/// 各列の合算式は `chara_to_status_result` と対をなす — 新しいソースを
/// どちらかに足すときは必ず両方に反映し、恒等式テストで検証すること。
pub fn chara_breakdown(chara: &Chara) -> StatusBreakdown {
    let mut b = Builder::default();

    // --- 基本 9 ステ (種族/メイン/サポ/メリポ/ML。HP/MP 特性は採用側ジョブに合算) ---
    const STAT_COLS: [(StatusKind, &str); 9] = [
        (StatusKind::Hp, "hp"),
        (StatusKind::Mp, "mp"),
        (StatusKind::Str, "str"),
        (StatusKind::Dex, "dex"),
        (StatusKind::Vit, "vit"),
        (StatusKind::Agi, "agi"),
        (StatusKind::Int, "int"),
        (StatusKind::Mnd, "mnd"),
        (StatusKind::Chr, "chr"),
    ];
    for (kind, col) in STAT_COLS {
        let p = chara.status_parts(kind);
        b.add(SRC_RACE, col, p.race as f64);
        b.add(SRC_MAIN_JOB, col, p.main_job as f64 + p.trait_main as f64);
        b.add(
            SRC_SUPPORT_JOB,
            col,
            p.support_job as f64 + p.trait_support as f64,
        );
        b.add(SRC_MERIT, col, p.merit as f64);
        b.add(SRC_MASTER_LEVEL, col, p.mlv as f64);
    }

    let total_jp = chara.job_points.total_jp_spent();
    let gift = calc_gift_bonuses(chara.main_job, total_jp);
    let jp_cat = calc_jp_category_bonuses(chara.main_job, &chara.job_points);
    let cs = combat_skills(chara);

    let vit = chara.status(StatusKind::Vit);
    let agi = chara.status(StatusKind::Agi);
    let str_val = chara.status(StatusKind::Str);
    let dex = chara.status(StatusKind::Dex);

    // --- 防御系 ---
    b.add(SRC_BASE, "def", calc_defense(vit, chara.main_lv, 0) as f64);
    b.add_trait(chara, "def", JobTrait::DefenseBonus);
    b.add(SRC_GIFT, "def", gift.physical_defense as f64);
    b.add(SRC_JOB_POINTS, "def", jp_cat.physical_defense as f64);

    b.add(
        SRC_BASE,
        "evasion",
        calc_evasion(agi, cs.eff_evasion_skill, 0) as f64,
    );
    b.add_trait(chara, "evasion", JobTrait::EvasionBonus);
    b.add(SRC_GIFT, "evasion", gift.physical_evasion as f64);
    b.add(SRC_JOB_POINTS, "evasion", jp_cat.physical_evasion as f64);

    b.add(SRC_BASE, "mdef", calc_magic_defense(0) as f64);
    b.add_trait(chara, "mdef", JobTrait::MagicDefenseBonus);
    b.add(SRC_GIFT, "mdef", gift.magic_defense as f64);
    b.add(SRC_JOB_POINTS, "mdef", jp_cat.magic_defense as f64);

    // 魔回避に基礎値はない (総合値 = 装備 + 下記ボーナス)
    b.add_trait(chara, "magic_evasion", JobTrait::MagicEvasionBonus);
    b.add(SRC_GIFT, "magic_evasion", gift.magic_evasion as f64);
    b.add(SRC_JOB_POINTS, "magic_evasion", jp_cat.magic_evasion as f64);

    // --- 近接 攻撃/命中 ---
    let (main_skill, is_h2h) = cs.main_skill();
    b.add(
        SRC_BASE,
        "attack",
        calc_main_attack(str_val, main_skill, is_h2h, 0) as f64,
    );
    b.add_trait(chara, "attack", JobTrait::AttackBonus);
    b.add(SRC_GIFT, "attack", gift.physical_attack as f64);
    b.add(SRC_JOB_POINTS, "attack", jp_cat.physical_attack as f64);

    b.add(
        SRC_BASE,
        "accuracy",
        calc_accuracy(dex, main_skill, 0) as f64,
    );
    b.add_trait(chara, "accuracy", JobTrait::AccuracyBonus);
    b.add(SRC_GIFT, "accuracy", gift.physical_accuracy as f64);
    b.add(SRC_JOB_POINTS, "accuracy", jp_cat.physical_accuracy as f64);

    // --- 遠隔 攻撃/命中 (レンジ武器装備時のみ) ---
    if let Some((_, skill_v)) = cs.ranged_weapon {
        b.add(
            SRC_BASE,
            "ranged_attack",
            calc_ranged_attack(str_val, skill_v, 0) as f64,
        );
        b.add_trait(chara, "ranged_attack", JobTrait::AttackBonus);
        b.add(
            SRC_GIFT,
            "ranged_attack",
            (gift.physical_attack + gift.ranged_attack) as f64,
        );
        b.add(
            SRC_JOB_POINTS,
            "ranged_attack",
            (jp_cat.physical_attack + jp_cat.ranged_attack) as f64,
        );

        b.add(
            SRC_BASE,
            "ranged_accuracy",
            calc_ranged_accuracy(agi, skill_v, 0) as f64,
        );
        b.add_trait(chara, "ranged_accuracy", JobTrait::AccuracyBonus);
        b.add(
            SRC_GIFT,
            "ranged_accuracy",
            (gift.physical_accuracy + gift.ranged_accuracy) as f64,
        );
        b.add(
            SRC_JOB_POINTS,
            "ranged_accuracy",
            (jp_cat.physical_accuracy + jp_cat.ranged_accuracy) as f64,
        );
    }

    // --- 魔法系 ---
    b.add(SRC_BASE, "magic_attack", calc_magic_attack(0) as f64);
    b.add_trait(chara, "magic_attack", JobTrait::MagicAttackBonus);
    b.add(SRC_GIFT, "magic_attack", gift.magic_attack as f64);
    b.add(SRC_JOB_POINTS, "magic_attack", jp_cat.magic_attack as f64);

    b.add_trait(chara, "magic_accuracy", JobTrait::MagicAccuracyBonus);
    b.add(SRC_GIFT, "magic_accuracy", gift.magic_accuracy as f64);
    b.add(
        SRC_JOB_POINTS,
        "magic_accuracy",
        jp_cat.magic_accuracy as f64,
    );

    b.add_trait(chara, "fast_cast", JobTrait::FastCast);
    b.add(
        SRC_GIFT,
        "fast_cast",
        chara.main_job.gift_value(Gift::FastCastEffect, total_jp) as f64,
    );

    // --- 近接プロパティ ---
    b.add_trait(chara, "store_tp", JobTrait::StoreTp);
    if chara.main_job == Job::Sam {
        b.add(SRC_MERIT, "store_tp", chara.merit_points.store_tp as f64);
    }
    b.add(SRC_GIFT, "store_tp", gift.store_tp as f64);
    b.add(SRC_JOB_POINTS, "store_tp", jp_cat.store_tp as f64);

    b.add_trait(chara, "double_attack", JobTrait::DoubleAttack);
    if chara.main_job == Job::War {
        let merit = chara
            .merit_points
            .job_merits
            .get("War")
            .map(|m| m.group1[4])
            .unwrap_or(0);
        b.add(SRC_MERIT, "double_attack", merit as f64);
        b.add(
            SRC_GIFT,
            "double_attack",
            calc_war_da_gift_bonus(total_jp) as f64,
        );
    }

    b.add_trait(chara, "triple_attack", JobTrait::TripleAttack);
    b.add(
        SRC_GIFT,
        "triple_attack",
        chara.main_job.gift_value(Gift::TripleAttackRate, total_jp) as f64,
    );

    b.add_trait(chara, "subtle_blow", JobTrait::SubtleBlow);
    b.add(
        SRC_GIFT,
        "subtle_blow",
        chara.main_job.gift_value(Gift::SubtleBlow, total_jp) as f64,
    );

    b.add_trait(chara, "skillchain_bonus", JobTrait::SkillchainBonus);
    b.add(SRC_GIFT, "skillchain_bonus", gift.skillchain_bonus as f64);

    // --- 遠隔プロパティ / 待機・回復系 ---
    b.add_trait(chara, "rapid_shot", JobTrait::RapidShot);
    b.add_trait(chara, "regen", JobTrait::AutoRegen);
    b.add_trait(chara, "refresh", JobTrait::AutoRefresh);

    // --- 状態異常レジスト補正 (テナシティ) ---
    b.add_trait(chara, "tenacity", JobTrait::Tenacity);

    StatusBreakdown { rows: b.rows }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::Race;
    use crate::status::{BonusStats, MeritPoints};
    use crate::wasm::chara_to_status_result;

    fn cell(b: &StatusBreakdown, src: &str, col: &str) -> f64 {
        b.rows
            .get(src)
            .and_then(|r| r.get(col))
            .copied()
            .unwrap_or(0.0)
    }

    /// 基本ステの恒等式: floor(Σキャラ行) + 装備 == StatusResult。
    /// (小数を持つのは race/main/sub のみで、整数項は floor を素通りするため
    ///  ゲーム仕様の「3 ソース合算後に切り捨て」と一致する)
    fn assert_stat_identity(chara: &Chara, b: &StatusBreakdown, col: &str, kind: StatusKind) {
        let char_sum = [
            SRC_RACE,
            SRC_MAIN_JOB,
            SRC_SUPPORT_JOB,
            SRC_MERIT,
            SRC_MASTER_LEVEL,
        ]
        .iter()
        .map(|src| cell(b, src, col))
        .sum::<f64>();
        let total = chara_to_status_result(chara);
        let expected = match kind {
            StatusKind::Hp => total.hp,
            StatusKind::Mp => total.mp,
            StatusKind::Str => total.str_,
            StatusKind::Vit => total.vit,
            StatusKind::Agi => total.agi,
            _ => chara.status(kind),
        };
        assert_eq!(
            char_sum.floor() as i32 + chara.bonus_stats.get(kind),
            expected,
            "identity failed for {col}"
        );
    }

    #[test]
    fn basic_stats_identity_and_race_fraction() {
        // Hum/War99/Drg59/ML50 (chara.rs の既存テストと同じ構成)
        // STR: race(D:37.5) + main(A:45) + sub(B@59:15) + mlv(50) = 147
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Drg, 59)
            .master_lv(50)
            .bonus_stats(BonusStats {
                str_: 20,
                hp: 100,
                ..Default::default()
            })
            .build()
            .unwrap();
        let b = chara_breakdown(&chara);

        assert_eq!(cell(&b, SRC_RACE, "str"), 37.5);
        assert_eq!(cell(&b, SRC_MASTER_LEVEL, "str"), 50.0);
        // War の MaxHpBoost (Lv95 rank9: +180) はメイン行の HP に合算される
        let hp_parts = chara.status_parts(StatusKind::Hp);
        assert_eq!(hp_parts.trait_main, 180);
        assert_eq!(
            cell(&b, SRC_MAIN_JOB, "hp"),
            hp_parts.main_job as f64 + 180.0
        );

        for (kind, col) in [
            (StatusKind::Hp, "hp"),
            (StatusKind::Mp, "mp"),
            (StatusKind::Str, "str"),
            (StatusKind::Dex, "dex"),
            (StatusKind::Vit, "vit"),
            (StatusKind::Agi, "agi"),
            (StatusKind::Int, "int"),
            (StatusKind::Mnd, "mnd"),
            (StatusKind::Chr, "chr"),
        ] {
            assert_stat_identity(&chara, &b, col, kind);
        }
    }

    #[test]
    fn mp_all_zero_when_main_job_has_no_mp() {
        // War はメインジョブに MP グレードなし → 全ソースの MP 行が空
        let chara = Chara::builder()
            .race(Race::Tar)
            .main_job(Job::War, 99)
            .support_job(Job::Blm, 49)
            .master_lv(0)
            .build()
            .unwrap();
        let b = chara_breakdown(&chara);
        for src in b.rows.keys() {
            assert_eq!(cell(&b, src, "mp"), 0.0, "mp must be 0 for source {src}");
        }
    }

    #[test]
    fn defense_evasion_mdef_identity() {
        let chara = Chara::builder()
            .race(Race::Elv)
            .main_job(Job::Pld, 99)
            .support_job(Job::Rdm, 49)
            .master_lv(20)
            .job_points(crate::job_points::JobPointCategories::all_maxed())
            .bonus_stats(BonusStats {
                def: 150,
                evasion: 40,
                magic_def_bonus: 12,
                ..Default::default()
            })
            .build()
            .unwrap();
        let b = chara_breakdown(&chara);
        let total = chara_to_status_result(&chara);

        let sum = |col: &str| -> i32 {
            b.rows
                .values()
                .map(|r| r.get(col).copied().unwrap_or(0.0))
                .sum::<f64>() as i32
        };
        assert_eq!(sum("def") + chara.bonus_stats.def, total.def);
        assert_eq!(sum("evasion") + chara.bonus_stats.evasion, total.evasion);
        assert_eq!(sum("mdef") + chara.bonus_stats.magic_def_bonus, total.mdef);
        // 魔回避は基礎なし: Σ行 == magic_evasion_bonus (装備は JS 側で加算)
        assert_eq!(sum("magic_evasion"), total.magic_evasion_bonus);
        // 基礎行が解析値であること (剰余ではない)
        assert_eq!(
            cell(&b, SRC_BASE, "def") as i32,
            calc_defense(chara.status(StatusKind::Vit), 99, 0)
        );
        assert_eq!(cell(&b, SRC_BASE, "mdef") as i32, 100);
    }

    #[test]
    fn attack_accuracy_identity_without_weapon() {
        // 武器なし = H2H 扱いの基礎値で恒等式が成り立つ
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Mnk, 99)
            .master_lv(0)
            .bonus_stats(BonusStats {
                attack: 30,
                accuracy: 25,
                ..Default::default()
            })
            .build()
            .unwrap();
        let b = chara_breakdown(&chara);
        let total = chara_to_status_result(&chara);
        let sum = |col: &str| -> i32 {
            b.rows
                .values()
                .map(|r| r.get(col).copied().unwrap_or(0.0))
                .sum::<f64>() as i32
        };
        assert_eq!(sum("attack") + chara.bonus_stats.attack, total.main_attack);
        assert_eq!(
            sum("accuracy") + chara.bonus_stats.accuracy,
            total.main_accuracy
        );
        // レンジ武器なし → 遠隔列は生成されない
        assert_eq!(sum("ranged_attack"), 0);
    }

    #[test]
    fn store_tp_trait_adopted_by_support() {
        // WAR99/SAM49: Store TP はサポ侍 rank2 (+15) → support 行
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Sam, 49)
            .master_lv(0)
            .build()
            .unwrap();
        let b = chara_breakdown(&chara);
        assert_eq!(cell(&b, SRC_MAIN_JOB, "store_tp"), 0.0);
        assert_eq!(cell(&b, SRC_SUPPORT_JOB, "store_tp"), 15.0);
        // DoubleAttack は WAR メイン採用 → main 行に job_trait_total と同値
        assert_eq!(
            cell(&b, SRC_MAIN_JOB, "double_attack"),
            chara.job_trait_total(JobTrait::DoubleAttack) as f64
        );
        assert!(cell(&b, SRC_MAIN_JOB, "double_attack") > 0.0);
        assert_eq!(cell(&b, SRC_SUPPORT_JOB, "double_attack"), 0.0);
    }

    #[test]
    fn sam_store_tp_merit_and_war_da_merit_gift() {
        // SAM メイン: Store TP メリット 5 → merit 行
        let merit = MeritPoints {
            store_tp: 5,
            ..Default::default()
        };
        let sam = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Sam, 99)
            .master_lv(0)
            .merit_points(merit)
            .build()
            .unwrap();
        let b = chara_breakdown(&sam);
        assert_eq!(cell(&b, SRC_MERIT, "store_tp"), 5.0);

        // WAR メイン: DA メリット group1[4]=5 → merit 行、JP 全振りギフト +10 → gift 行
        let mut merit = MeritPoints::default();
        merit.job_merits.insert(
            "War".to_string(),
            crate::status::JobMerits {
                group1: [0, 0, 0, 0, 5, 0, 0, 0],
                group2: [0; 8],
            },
        );
        let war = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .master_lv(0)
            .merit_points(merit)
            .job_points(crate::job_points::JobPointCategories::all_maxed())
            .build()
            .unwrap();
        let b = chara_breakdown(&war);
        assert_eq!(cell(&b, SRC_MERIT, "double_attack"), 5.0);
        assert_eq!(cell(&b, SRC_GIFT, "double_attack"), 10.0);
        // 恒等式: Σ行 == StatusResult.double_attack_pct (装備 0)
        let total = chara_to_status_result(&war);
        let sum: f64 = b
            .rows
            .values()
            .map(|r| r.get("double_attack").copied().unwrap_or(0.0))
            .sum();
        assert_eq!(sum as i32, total.double_attack_pct);
    }

    #[test]
    fn fast_cast_trait_adopted_by_support_rdm() {
        // PLD99/RDM49: FastCast はサポ赤 (Lv15 rank1: +10) → support 行
        let chara = Chara::builder()
            .race(Race::Elv)
            .main_job(Job::Pld, 99)
            .support_job(Job::Rdm, 49)
            .master_lv(0)
            .build()
            .unwrap();
        let b = chara_breakdown(&chara);
        assert_eq!(cell(&b, SRC_MAIN_JOB, "fast_cast"), 0.0);
        let total = chara_to_status_result(&chara);
        assert_eq!(
            cell(&b, SRC_SUPPORT_JOB, "fast_cast") as i32,
            total.fast_cast_pct
        );
        assert!(total.fast_cast_pct > 0);
    }

    /// 主要プロパティ列の恒等式を一括検証 (装備なし構成)
    #[test]
    fn property_columns_identity() {
        for (main, sub) in [
            (Job::War, Some(Job::Sam)),
            (Job::Sam, Some(Job::War)),
            (Job::Rdm, Some(Job::Whm)),
            (Job::Pld, Some(Job::Blu)),
            (Job::Thf, None),
        ] {
            let mut builder = Chara::builder()
                .race(Race::Hum)
                .main_job(main, 99)
                .master_lv(50)
                .job_points(crate::job_points::JobPointCategories::all_maxed());
            if let Some(sub) = sub {
                builder = builder.support_job(sub, 49);
            }
            let chara = builder.build().unwrap();
            let b = chara_breakdown(&chara);
            let total = chara_to_status_result(&chara);
            let sum = |col: &str| -> i32 {
                b.rows
                    .values()
                    .map(|r| r.get(col).copied().unwrap_or(0.0))
                    .sum::<f64>() as i32
            };
            let cases: [(&str, i32); 9] = [
                ("store_tp", total.store_tp),
                ("double_attack", total.double_attack_pct),
                ("triple_attack", total.triple_attack_pct),
                ("subtle_blow", total.subtle_blow),
                ("skillchain_bonus", total.skillchain_bonus),
                ("fast_cast", total.fast_cast_pct),
                ("rapid_shot", total.rapid_shot_pct),
                ("regen", total.regen),
                ("refresh", total.refresh),
            ];
            for (col, expected) in cases {
                assert_eq!(
                    sum(col),
                    expected,
                    "identity failed for {col} ({main:?}/{sub:?})"
                );
            }
            assert_eq!(sum("magic_accuracy"), total.magic_accuracy_bonus);
            assert_eq!(sum("magic_attack"), total.magic_attack, "magic_attack");
            assert_eq!(sum("tenacity"), total.tenacity, "tenacity");
        }
    }
}
