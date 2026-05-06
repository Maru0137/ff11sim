//! ギフト (Gift) — ジョブポイントの累計量に応じて獲得する成長要素。
//!
//! データソース: https://wiki.ffo.jp/html/32343.html
//! 各ジョブのギフト詳細: https://wiki.ffo.jp/html/37176.html〜37197.html
//!
//! 設計:
//! - 各 (job, gift) ペアに `&[(threshold_jp, cumulative_value)]` の配列が対応する。
//!   - 例: `(War, PhysicalAttack) => &[(10, 10), (210, 25), (660, 45), (1360, 70)]`
//!     → 10 JP で +10、210 JP で +25 (累積)、…、1360 JP で +70。
//! - 閾値も累積値もジョブ毎に異なる (スロット位置やジョブの方向性で変動)。
//! - 空スライス = 未獲得。
//!
//! スコープ: 基本ステータス系 + ジョブ特性効果アップ系 + クリ率/WS 系
//! スコープ外: キャパシティポイントアップ / スペリア 1-5 / ★ジョブマスター系

use crate::job::Job;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gift {
    // ============ 基本ステータス系 ============
    PhysicalAttack,
    PhysicalDefense,
    PhysicalAccuracy,
    PhysicalEvasion,
    MagicAttack,
    MagicDefense,
    MagicAccuracy,
    MagicEvasion,
    StoreTp,
    SkillchainBonus,
    // 注: 遠隔系 (RangedAttack/RangedAccuracy) はギフトとしてのティア定義が
    //     存在しないため Gift enum には含めない。COR の「遠隔命中アップ」等は
    //     ジョブポイント項目 (JP category) 側で扱う。

    // ============ ジョブ特性効果アップ系 ============
    DoubleAttackRate,
    DoubleAttackEffect,
    TripleAttackRate,
    FencerEffect,
    CritIncreaseEffect,
    SmiteEffect,
    /// BLU 専用: ジョブ特性効果アップ (rank +N)
    JobTraitEffectUp,

    // ============ クリ率/WS 系 ============
    CriticalHitRate,
    WeaponSkillDamage,
}

pub const ALL_GIFTS: &[Gift] = &[
    Gift::PhysicalAttack,
    Gift::PhysicalDefense,
    Gift::PhysicalAccuracy,
    Gift::PhysicalEvasion,
    Gift::MagicAttack,
    Gift::MagicDefense,
    Gift::MagicAccuracy,
    Gift::MagicEvasion,
    Gift::StoreTp,
    Gift::SkillchainBonus,
    Gift::DoubleAttackRate,
    Gift::DoubleAttackEffect,
    Gift::TripleAttackRate,
    Gift::FencerEffect,
    Gift::CritIncreaseEffect,
    Gift::SmiteEffect,
    Gift::JobTraitEffectUp,
    Gift::CriticalHitRate,
    Gift::WeaponSkillDamage,
];

impl Job {
    /// このジョブの (gift) ティア定義: `&[(threshold_jp, cumulative_value)]`。
    /// 各ティアは累計 JP が threshold 以上になると cumulative_value で上書きされる。
    /// 空スライス = 未獲得。
    pub fn gift_tiers(&self, gift: Gift) -> &'static [(i32, i32)] {
        gift_tiers_table(*self, gift)
    }

    /// 累計 JP からティア数を返す (0 = 未獲得)。
    pub fn gift_tier_at_jp(&self, gift: Gift, total_jp: i32) -> usize {
        self.gift_tiers(gift)
            .iter()
            .filter(|&&(req_jp, _)| total_jp >= req_jp)
            .count()
    }

    /// 累計 JP からギフト効果値 (累積) を返す。
    pub fn gift_value(&self, gift: Gift, total_jp: i32) -> i32 {
        let tiers = self.gift_tiers(gift);
        if tiers.is_empty() {
            return 0;
        }
        let count = tiers.iter().filter(|&&(req_jp, _)| total_jp >= req_jp).count();
        if count == 0 {
            return 0;
        }
        let idx = std::cmp::min(count, tiers.len()) - 1;
        tiers[idx].1
    }
}

// ---------------------------------------------------------------------------
// (job, gift) → ティア定義テーブル
// データソース: data/job_gifts.json (元データ) + 各ジョブの wiki ページ
// 各タプル: (累計 JP の閾値, そのティアでの累積効果値)
// ---------------------------------------------------------------------------

fn gift_tiers_table(job: Job, gift: Gift) -> &'static [(i32, i32)] {
    use Gift::*;
    use Job::*;

    match (gift, job) {
        // ============ PhysicalDefense ============
        // 閾値はスロット 0 共通: 5/180/605/1280
        (PhysicalDefense, War) => &[(5, 10), (180, 25), (605, 45), (1280, 70)],
        (PhysicalDefense, Mnk) => &[(5, 5), (180, 13), (605, 23), (1280, 36)],
        (PhysicalDefense, Thf) => &[(5, 4), (180, 10), (605, 18), (1280, 28)],
        (PhysicalDefense, Pld) => &[(5, 15), (180, 38), (605, 68), (1280, 106)],
        (PhysicalDefense, Drk) => &[(5, 4), (180, 10), (605, 18), (1280, 28)],
        (PhysicalDefense, Bst) => &[(5, 10), (180, 25), (605, 45), (1280, 70)],
        (PhysicalDefense, Brd) => &[(5, 3), (180, 8), (605, 14), (1280, 22)],
        (PhysicalDefense, Rng) => &[(5, 3), (180, 8), (605, 14), (1280, 22)],
        (PhysicalDefense, Sam) => &[(5, 6), (180, 15), (605, 27), (1280, 42)],
        (PhysicalDefense, Nin) => &[(5, 3), (180, 8), (605, 14), (1280, 22)],
        (PhysicalDefense, Drg) => &[(5, 6), (180, 15), (605, 27), (1280, 42)],
        (PhysicalDefense, Smn) => &[(5, 3), (180, 8), (605, 14), (1280, 22)],
        (PhysicalDefense, Blu) => &[(5, 6), (180, 15), (605, 27), (1280, 42)],
        (PhysicalDefense, Cor) => &[(5, 3), (180, 8), (605, 14), (1280, 22)],
        (PhysicalDefense, Dnc) => &[(5, 6), (180, 15), (605, 27), (1280, 42)],
        (PhysicalDefense, Run) => &[(5, 10), (180, 25), (605, 45), (1280, 70)],

        // ============ PhysicalAttack ============
        // 閾値はスロット 1 共通: 10/210/660/1360
        (PhysicalAttack, War) => &[(10, 10), (210, 25), (660, 45), (1360, 70)],
        (PhysicalAttack, Mnk) => &[(10, 8), (210, 20), (660, 36), (1360, 56)],
        (PhysicalAttack, Thf) => &[(10, 7), (210, 18), (660, 32), (1360, 50)],
        (PhysicalAttack, Pld) => &[(10, 4), (210, 10), (660, 18), (1360, 28)],
        (PhysicalAttack, Drk) => &[(10, 15), (210, 38), (660, 68), (1360, 106)],
        (PhysicalAttack, Bst) => &[(10, 10), (210, 25), (660, 45), (1360, 70)],
        (PhysicalAttack, Rng) => &[(10, 10), (210, 25), (660, 45), (1360, 70)],
        (PhysicalAttack, Sam) => &[(10, 10), (210, 25), (660, 45), (1360, 70)],
        (PhysicalAttack, Nin) => &[(10, 8), (210, 20), (660, 36), (1360, 56)],
        (PhysicalAttack, Drg) => &[(10, 10), (210, 25), (660, 45), (1360, 70)],
        (PhysicalAttack, Blu) => &[(10, 8), (210, 20), (660, 36), (1360, 56)],
        (PhysicalAttack, Cor) => &[(10, 5), (210, 13), (660, 23), (1360, 36)],
        (PhysicalAttack, Pup) => &[(10, 8), (210, 20), (660, 36), (1360, 56)],
        (PhysicalAttack, Dnc) => &[(10, 8), (210, 20), (660, 36), (1360, 56)],
        (PhysicalAttack, Run) => &[(10, 8), (210, 20), (660, 36), (1360, 56)],

        // ============ PhysicalEvasion ============
        // 閾値はスロット 2 共通: 20/245/720/1445
        (PhysicalEvasion, War) => &[(20, 5), (245, 13), (720, 23), (1445, 36)],
        (PhysicalEvasion, Mnk) => &[(20, 6), (245, 15), (720, 27), (1445, 42)],
        (PhysicalEvasion, Thf) => &[(20, 10), (245, 25), (720, 45), (1445, 70)],
        (PhysicalEvasion, Pld) => &[(20, 3), (245, 8), (720, 14), (1445, 22)],
        (PhysicalEvasion, Drk) => &[(20, 3), (245, 8), (720, 14), (1445, 22)],
        (PhysicalEvasion, Bst) => &[(20, 5), (245, 13), (720, 23), (1445, 36)],
        (PhysicalEvasion, Brd) => &[(20, 5), (245, 13), (720, 23), (1445, 36)],
        (PhysicalEvasion, Rng) => &[(20, 5), (245, 13), (720, 23), (1445, 36)],
        (PhysicalEvasion, Sam) => &[(20, 3), (245, 8), (720, 14), (1445, 22)],
        (PhysicalEvasion, Nin) => &[(20, 8), (245, 20), (720, 36), (1445, 56)],
        (PhysicalEvasion, Drg) => &[(20, 3), (245, 8), (720, 14), (1445, 22)],
        (PhysicalEvasion, Smn) => &[(20, 3), (245, 8), (720, 14), (1445, 22)],
        (PhysicalEvasion, Blu) => &[(20, 5), (245, 13), (720, 23), (1445, 36)],
        (PhysicalEvasion, Cor) => &[(20, 3), (245, 8), (720, 14), (1445, 22)],
        (PhysicalEvasion, Pup) => &[(20, 5), (245, 13), (720, 23), (1445, 36)],
        (PhysicalEvasion, Dnc) => &[(20, 8), (245, 20), (720, 36), (1445, 56)],
        (PhysicalEvasion, Run) => &[(20, 5), (245, 13), (720, 23), (1445, 36)],

        // ============ PhysicalAccuracy ============
        // 閾値はスロット 3 共通: 30/280/780/1530
        (PhysicalAccuracy, War) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Mnk) => &[(30, 6), (280, 15), (780, 26), (1530, 41)],
        (PhysicalAccuracy, Whm) => &[(30, 2), (280, 5), (780, 9), (1530, 14)],
        (PhysicalAccuracy, Rdm) => &[(30, 3), (280, 8), (780, 14), (1530, 22)],
        (PhysicalAccuracy, Thf) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Pld) => &[(30, 4), (280, 10), (780, 18), (1530, 28)],
        (PhysicalAccuracy, Drk) => &[(30, 3), (280, 8), (780, 14), (1530, 22)],
        (PhysicalAccuracy, Bst) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Brd) => &[(30, 3), (280, 8), (780, 14), (1530, 22)],
        (PhysicalAccuracy, Rng) => &[(30, 10), (280, 25), (780, 45), (1530, 70)],
        (PhysicalAccuracy, Sam) => &[(30, 6), (280, 15), (780, 27), (1530, 42)],
        (PhysicalAccuracy, Nin) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Drg) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Blu) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Cor) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Pup) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],
        (PhysicalAccuracy, Dnc) => &[(30, 6), (280, 15), (780, 27), (1530, 42)],
        (PhysicalAccuracy, Run) => &[(30, 5), (280, 13), (780, 23), (1530, 36)],

        // ============ MagicAttack (主に魔法系ジョブ) ============
        // 閾値はスロット 1 共通 (戦闘ジョブの PATK と同じ): 10/210/660/1360
        (MagicAttack, Whm) => &[(10, 3), (210, 8), (660, 14), (1360, 22)],
        (MagicAttack, Blm) => &[(10, 7), (210, 18), (660, 32), (1360, 50)],
        (MagicAttack, Rdm) => &[(10, 4), (210, 10), (660, 18), (1360, 28)],
        (MagicAttack, Sch) => &[(10, 7), (210, 18), (660, 32), (1360, 50)],
        (MagicAttack, Geo) => &[(10, 7), (210, 18), (660, 32), (1360, 50)],
        // Nin の MATK はスロット 5 (60/360/910/1710) と推測
        (MagicAttack, Nin) => &[(60, 3), (360, 8), (910, 14), (1710, 22)],
        // Cor の MATK はスロット 4 (45/320/845/1620)
        (MagicAttack, Cor) => &[(45, 2), (320, 5), (845, 9), (1620, 14)],

        // ============ MagicDefense ============
        // 閾値はスロット 0 共通: 5/180/605/1280
        (MagicDefense, Whm) => &[(5, 7), (180, 18), (605, 32), (1280, 50)],
        (MagicDefense, Blm) => &[(5, 2), (180, 5), (605, 9), (1280, 14)],
        (MagicDefense, Rdm) => &[(5, 4), (180, 10), (605, 18), (1280, 28)],
        (MagicDefense, Brd) => &[(5, 5), (180, 13), (605, 23), (1280, 36)],
        (MagicDefense, Smn) => &[(5, 5), (180, 13), (605, 23), (1280, 36)],
        (MagicDefense, Sch) => &[(5, 5), (180, 13), (605, 23), (1280, 36)],
        (MagicDefense, Geo) => &[(5, 5), (180, 13), (605, 23), (1280, 36)],
        (MagicDefense, Run) => &[(5, 5), (180, 13), (605, 23), (1280, 36)],

        // ============ MagicEvasion ============
        // 閾値はジョブによって異なる:
        //   戦闘系 (PDEF/PATK/PEVA/PACC で 4 スロット使用) はスロット 4: 45/320/845/1620
        //   魔法系 (MDEF が slot 0) はスロット 2: 20/245/720/1445
        //   Cor は MEVA がスロット 5: 60/360/910/1710 (slot 4 が MATK)
        (MagicEvasion, War) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Mnk) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Whm) => &[(20, 7), (245, 18), (720, 32), (1445, 50)],
        (MagicEvasion, Blm) => &[(20, 6), (245, 15), (720, 27), (1445, 42)],
        (MagicEvasion, Rdm) => &[(20, 8), (245, 20), (720, 36), (1445, 56)],
        (MagicEvasion, Thf) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Pld) => &[(45, 6), (320, 15), (845, 27), (1620, 42)],
        (MagicEvasion, Drk) => &[(45, 6), (320, 15), (845, 27), (1620, 42)],
        (MagicEvasion, Bst) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Brd) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Rng) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Sam) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Drg) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Smn) => &[(45, 6), (320, 15), (845, 27), (1620, 42)],
        (MagicEvasion, Cor) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicEvasion, Pup) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Dnc) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicEvasion, Sch) => &[(45, 6), (320, 15), (845, 27), (1620, 42)],
        (MagicEvasion, Geo) => &[(45, 6), (320, 15), (845, 27), (1620, 42)],
        (MagicEvasion, Run) => &[(45, 6), (320, 15), (845, 27), (1620, 42)],

        // ============ MagicAccuracy ============
        // 戦闘系: スロット 5 (60/360/910/1710)
        // 魔法系: スロット 3 (30/280/780/1530)
        // Cor は MACC が slot 6 (80/405/980/1805)
        (MagicAccuracy, War) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Mnk) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Whm) => &[(30, 7), (280, 18), (780, 32), (1530, 50)],
        (MagicAccuracy, Blm) => &[(30, 6), (280, 15), (780, 27), (1530, 42)],
        (MagicAccuracy, Rdm) => &[(30, 10), (280, 25), (780, 45), (1530, 70)],
        (MagicAccuracy, Thf) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Pld) => &[(60, 6), (360, 15), (910, 27), (1710, 42)],
        (MagicAccuracy, Drk) => &[(60, 6), (360, 15), (910, 27), (1710, 42)],
        (MagicAccuracy, Bst) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Brd) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Rng) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Sam) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Nin) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Drg) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Blu) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Cor) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],
        (MagicAccuracy, Pup) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Dnc) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicAccuracy, Sch) => &[(30, 6), (280, 15), (780, 27), (1530, 42)],
        (MagicAccuracy, Geo) => &[(30, 6), (280, 15), (780, 27), (1530, 42)],
        (MagicAccuracy, Run) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],

        // ============ SkillchainBonus (Sam/Dnc) ============
        // 80/405/980/1805 で +2/+2/+2/+2 (累積 +2/+4/+6/+8) ※暫定要 wiki 確認
        (SkillchainBonus, Sam) => &[(80, 2), (405, 4), (980, 6), (1805, 8)],
        (SkillchainBonus, Dnc) => &[(80, 2), (405, 4), (980, 6), (1805, 8)],

        // ============ DoubleAttackRate (戦士) ============
        // 125/450/1050/1900 で +2/+2/+3/+3 (累積 +2/+4/+7/+10)
        (DoubleAttackRate, War) => &[(125, 2), (450, 4), (1050, 7), (1900, 10)],

        // ============ FencerEffect (戦士のみ) ============
        // 80/405/980/1805 で +50/+50/+60/+70 (累積)
        (FencerEffect, War) => &[(80, 50), (405, 100), (980, 160), (1805, 230)],

        // ============ CritIncreaseEffect (戦士) ============
        // 150/500/1125/2000 で +2/+2/+3/+3
        (CritIncreaseEffect, War) => &[(150, 2), (500, 4), (1125, 7), (2000, 10)],

        // ============ CriticalHitRate (戦士) ============
        // 100/1200 で +5/+5
        (CriticalHitRate, War) => &[(100, 5), (1200, 10)],

        // ============ WeaponSkillDamage (戦士) ============
        // 550 で +3
        (WeaponSkillDamage, War) => &[(550, 3)],

        // ============ JobTraitEffectUp (BLU) ============
        // 100/1200 で +1/+2 rank
        (JobTraitEffectUp, Blu) => &[(100, 1), (1200, 2)],

        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::Job;
    use strum::IntoEnumIterator;

    /// 構造テスト: 全 (job, gift) ペアでテーブル参照がパニックしない
    #[test]
    fn test_gift_definitions_for_all_pairs() {
        for job in Job::iter() {
            for &g in ALL_GIFTS {
                let _ = job.gift_tiers(g);
                for jp in [0, 50, 100, 500, 1200, 2100] {
                    let _ = job.gift_value(g, jp);
                }
            }
        }
    }

    /// 全ジョブ 0 JP → ギフト効果は 0
    #[test]
    fn test_no_gift_at_zero_jp() {
        for job in Job::iter() {
            for &g in ALL_GIFTS {
                assert_eq!(
                    job.gift_value(g, 0),
                    0,
                    "{:?} {:?} should be 0 at 0 JP",
                    job,
                    g
                );
            }
        }
    }

    /// 戦士 2100 JP の主要ギフト最大値検証
    #[test]
    fn test_war_full_jp() {
        let total_jp = 2100;
        assert_eq!(Job::War.gift_value(Gift::PhysicalDefense, total_jp), 70);
        assert_eq!(Job::War.gift_value(Gift::PhysicalAttack, total_jp), 70);
        assert_eq!(Job::War.gift_value(Gift::PhysicalEvasion, total_jp), 36);
        assert_eq!(Job::War.gift_value(Gift::PhysicalAccuracy, total_jp), 36);
        assert_eq!(Job::War.gift_value(Gift::MagicEvasion, total_jp), 36);
        assert_eq!(Job::War.gift_value(Gift::MagicAccuracy, total_jp), 36);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, total_jp), 10);
        assert_eq!(Job::War.gift_value(Gift::FencerEffect, total_jp), 230);
        assert_eq!(Job::War.gift_value(Gift::CritIncreaseEffect, total_jp), 10);
        assert_eq!(Job::War.gift_value(Gift::CriticalHitRate, total_jp), 10);
        assert_eq!(Job::War.gift_value(Gift::WeaponSkillDamage, total_jp), 3);
    }

    /// 白魔道士 2100 JP
    #[test]
    fn test_whm_full_jp() {
        let total_jp = 2100;
        assert_eq!(Job::Whm.gift_value(Gift::MagicDefense, total_jp), 50);
        assert_eq!(Job::Whm.gift_value(Gift::MagicAttack, total_jp), 22);
        assert_eq!(Job::Whm.gift_value(Gift::MagicEvasion, total_jp), 50);
        assert_eq!(Job::Whm.gift_value(Gift::MagicAccuracy, total_jp), 50);
        assert_eq!(Job::Whm.gift_value(Gift::PhysicalAccuracy, total_jp), 14);
    }

    /// パラディン 2100 JP
    #[test]
    fn test_pld_full_jp() {
        let total_jp = 2100;
        assert_eq!(Job::Pld.gift_value(Gift::PhysicalDefense, total_jp), 106);
        assert_eq!(Job::Pld.gift_value(Gift::PhysicalAttack, total_jp), 28);
    }

    /// 暗黒騎士 2100 JP
    #[test]
    fn test_drk_full_jp() {
        let total_jp = 2100;
        assert_eq!(Job::Drk.gift_value(Gift::PhysicalAttack, total_jp), 106);
    }

    /// COR は MEVA/MACC のスロットがずれるため特殊
    #[test]
    fn test_cor_full_jp() {
        let total_jp = 2100;
        assert_eq!(Job::Cor.gift_value(Gift::PhysicalAttack, total_jp), 36);
        assert_eq!(Job::Cor.gift_value(Gift::MagicAttack, total_jp), 14);
        assert_eq!(Job::Cor.gift_value(Gift::MagicEvasion, total_jp), 36);
        assert_eq!(Job::Cor.gift_value(Gift::MagicAccuracy, total_jp), 36);
    }

    /// 戦士 ダブルアタック確率の閾値ごとの値検証
    #[test]
    fn test_war_double_attack_rate_thresholds() {
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 124), 0);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 125), 2);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 449), 2);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 450), 4);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 1049), 4);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 1050), 7);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 1899), 7);
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, 1900), 10);
    }

    /// 全 22 ジョブ × 全ギフトを 2100 JP で評価し、最後のティア値と一致することを確認する。
    /// (gift_tiers_table の構造整合性: gift_value(2100) == tiers.last().value)
    #[test]
    fn test_all_jobs_full_jp_consistency() {
        for job in Job::iter() {
            for &g in ALL_GIFTS {
                let tiers = job.gift_tiers(g);
                let expected = tiers.last().map_or(0, |(_, v)| *v);
                let actual = job.gift_value(g, 2100);
                assert_eq!(
                    actual, expected,
                    "{:?} {:?}: gift_value(2100) = {}, last tier value = {}",
                    job, g, actual, expected
                );
            }
        }
    }

    /// 各 (job, gift) ペアの 2100 JP 時の期待値 (gift_tiers_table とは独立した
    /// ハードコード値)。書き起こし時のタイポ検知に使う。
    /// 一覧に無いペアは 0 (このジョブが当該ギフトを獲得しない)。
    fn expected_gift_at_full_jp(job: Job, gift: Gift) -> i32 {
        use Gift::*;
        use Job::*;
        match (gift, job) {
            // ========= PhysicalDefense (slot0: 5/180/605/1280) =========
            (PhysicalDefense, War) => 70,
            (PhysicalDefense, Mnk) => 36,
            (PhysicalDefense, Thf) => 28,
            (PhysicalDefense, Pld) => 106,
            (PhysicalDefense, Drk) => 28,
            (PhysicalDefense, Bst) => 70,
            (PhysicalDefense, Brd) => 22,
            (PhysicalDefense, Rng) => 22,
            (PhysicalDefense, Sam) => 42,
            (PhysicalDefense, Nin) => 22,
            (PhysicalDefense, Drg) => 42,
            (PhysicalDefense, Smn) => 22,
            (PhysicalDefense, Blu) => 42,
            (PhysicalDefense, Cor) => 22,
            (PhysicalDefense, Dnc) => 42,
            (PhysicalDefense, Run) => 70,

            // ========= PhysicalAttack (slot1: 10/210/660/1360) =========
            (PhysicalAttack, War) => 70,
            (PhysicalAttack, Mnk) => 56,
            (PhysicalAttack, Thf) => 50,
            (PhysicalAttack, Pld) => 28,
            (PhysicalAttack, Drk) => 106,
            (PhysicalAttack, Bst) => 70,
            (PhysicalAttack, Rng) => 70,
            (PhysicalAttack, Sam) => 70,
            (PhysicalAttack, Nin) => 56,
            (PhysicalAttack, Drg) => 70,
            (PhysicalAttack, Blu) => 56,
            (PhysicalAttack, Cor) => 36,
            (PhysicalAttack, Pup) => 56,
            (PhysicalAttack, Dnc) => 56,
            (PhysicalAttack, Run) => 56,

            // ========= PhysicalEvasion (slot2: 20/245/720/1445) =========
            (PhysicalEvasion, War) => 36,
            (PhysicalEvasion, Mnk) => 42,
            (PhysicalEvasion, Thf) => 70,
            (PhysicalEvasion, Pld) => 22,
            (PhysicalEvasion, Drk) => 22,
            (PhysicalEvasion, Bst) => 36,
            (PhysicalEvasion, Brd) => 36,
            (PhysicalEvasion, Rng) => 36,
            (PhysicalEvasion, Sam) => 22,
            (PhysicalEvasion, Nin) => 56,
            (PhysicalEvasion, Drg) => 22,
            (PhysicalEvasion, Smn) => 22,
            (PhysicalEvasion, Blu) => 36,
            (PhysicalEvasion, Cor) => 22,
            (PhysicalEvasion, Pup) => 36,
            (PhysicalEvasion, Dnc) => 56,
            (PhysicalEvasion, Run) => 36,

            // ========= PhysicalAccuracy (slot3: 30/280/780/1530) =========
            (PhysicalAccuracy, War) => 36,
            (PhysicalAccuracy, Mnk) => 41,
            (PhysicalAccuracy, Whm) => 14,
            (PhysicalAccuracy, Rdm) => 22,
            (PhysicalAccuracy, Thf) => 36,
            (PhysicalAccuracy, Pld) => 28,
            (PhysicalAccuracy, Drk) => 22,
            (PhysicalAccuracy, Bst) => 36,
            (PhysicalAccuracy, Brd) => 22,
            (PhysicalAccuracy, Rng) => 70,
            (PhysicalAccuracy, Sam) => 42,
            (PhysicalAccuracy, Nin) => 36,
            (PhysicalAccuracy, Drg) => 36,
            (PhysicalAccuracy, Blu) => 36,
            (PhysicalAccuracy, Cor) => 36,
            (PhysicalAccuracy, Pup) => 36,
            (PhysicalAccuracy, Dnc) => 42,
            (PhysicalAccuracy, Run) => 36,

            // ========= MagicDefense (slot0 for casters: 5/180/605/1280) =========
            (MagicDefense, Whm) => 50,
            (MagicDefense, Blm) => 14,
            (MagicDefense, Rdm) => 28,
            (MagicDefense, Brd) => 36,
            (MagicDefense, Smn) => 36,
            (MagicDefense, Sch) => 36,
            (MagicDefense, Geo) => 36,
            (MagicDefense, Run) => 36,

            // ========= MagicAttack =========
            (MagicAttack, Whm) => 22,
            (MagicAttack, Blm) => 50,
            (MagicAttack, Rdm) => 28,
            (MagicAttack, Sch) => 50,
            (MagicAttack, Geo) => 50,
            (MagicAttack, Nin) => 22,
            (MagicAttack, Cor) => 14,

            // ========= MagicEvasion =========
            (MagicEvasion, War) => 36,
            (MagicEvasion, Mnk) => 36,
            (MagicEvasion, Whm) => 50,
            (MagicEvasion, Blm) => 42,
            (MagicEvasion, Rdm) => 56,
            (MagicEvasion, Thf) => 36,
            (MagicEvasion, Pld) => 42,
            (MagicEvasion, Drk) => 42,
            (MagicEvasion, Bst) => 36,
            (MagicEvasion, Brd) => 36,
            (MagicEvasion, Rng) => 36,
            (MagicEvasion, Sam) => 36,
            (MagicEvasion, Drg) => 36,
            (MagicEvasion, Smn) => 42,
            (MagicEvasion, Cor) => 36,
            (MagicEvasion, Pup) => 36,
            (MagicEvasion, Dnc) => 36,
            (MagicEvasion, Sch) => 42,
            (MagicEvasion, Geo) => 42,
            (MagicEvasion, Run) => 42,

            // ========= MagicAccuracy =========
            (MagicAccuracy, War) => 36,
            (MagicAccuracy, Mnk) => 36,
            (MagicAccuracy, Whm) => 50,
            (MagicAccuracy, Blm) => 42,
            (MagicAccuracy, Rdm) => 70,
            (MagicAccuracy, Thf) => 36,
            (MagicAccuracy, Pld) => 42,
            (MagicAccuracy, Drk) => 42,
            (MagicAccuracy, Bst) => 36,
            (MagicAccuracy, Brd) => 36,
            (MagicAccuracy, Rng) => 36,
            (MagicAccuracy, Sam) => 36,
            (MagicAccuracy, Nin) => 36,
            (MagicAccuracy, Drg) => 36,
            (MagicAccuracy, Blu) => 36,
            (MagicAccuracy, Cor) => 36,
            (MagicAccuracy, Pup) => 36,
            (MagicAccuracy, Dnc) => 36,
            (MagicAccuracy, Sch) => 42,
            (MagicAccuracy, Geo) => 42,
            (MagicAccuracy, Run) => 36,

            // ========= SkillchainBonus =========
            (SkillchainBonus, Sam) => 8,
            (SkillchainBonus, Dnc) => 8,

            // ========= 戦士の特性効果アップ系・クリ率系 =========
            (DoubleAttackRate, War) => 10,
            (FencerEffect, War) => 230,
            (CritIncreaseEffect, War) => 10,
            (CriticalHitRate, War) => 10,
            (WeaponSkillDamage, War) => 3,

            // ========= BLU JobTraitEffectUp =========
            (JobTraitEffectUp, Blu) => 2,

            _ => 0,
        }
    }

    /// 全 22 ジョブ × 全ギフトを 2100 JP で評価し、ハードコードされた期待値と一致するか検証する。
    /// `gift_tiers_table` と `expected_gift_at_full_jp` の両方が二重チェックされ、
    /// 書き起こし時のタイポを検知できる (JobTrait の `test_all_jobs_lv99_traits` と対称)。
    #[test]
    fn test_all_jobs_full_jp_gifts() {
        for job in Job::iter() {
            for &g in ALL_GIFTS {
                let actual = job.gift_value(g, 2100);
                let expected = expected_gift_at_full_jp(job, g);
                assert_eq!(
                    actual, expected,
                    "{:?} 2100JP / {:?}: expected {}, got {}",
                    job, g, expected, actual
                );
            }
        }
    }

    /// BLU の ジョブ特性効果アップ
    #[test]
    fn test_blu_job_trait_effect_up() {
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 0), 0);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 99), 0);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 100), 1);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 1199), 1);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 1200), 2);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 2100), 2);
        assert_eq!(Job::War.gift_value(Gift::JobTraitEffectUp, 2100), 0);
    }
}
