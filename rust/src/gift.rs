//! ギフト (Gift) — ジョブポイントの累計量に応じて獲得する成長要素。
//!
//! データソース: https://wiki.ffo.jp/html/32343.html
//! 各ジョブのギフト詳細: https://wiki.ffo.jp/html/37176.html〜37197.html
//!
//! 設計:
//! - JP 閾値 (threshold) は同一ギフト種でジョブ間共通 (`Gift::thresholds`)
//! - 累積効果値 (cumulative) はジョブ毎に異なる (`Job::gift_cumulative`)
//!   - 例: War PhysicalDefense は +10/+15/+20/+25 (累積 70)、Mnk は +5/+8/+10/+13 (累積 36)
//! - 集計は `Chara::gift_total(gift)` で取得 (メインジョブのみ)。
//!
//! スコープ:
//! - 基本ステータス系 (Physical/Magic Attack/Defense/Accuracy/Evasion 等)
//! - ジョブ特性効果アップ系 (DA/Fencer/CritIncrease 等)
//! - クリ率/WS ボーナス系 (CritRate/WeaponSkillDamage)
//!
//! スコープ外:
//! - キャパシティポイントアップ
//! - スペリア 1-5 (装備 tier 制限)
//! - ★ジョブマスター系 (再使用時間短縮、独自効果)

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
    /// 飛攻 (遠隔攻撃力) — 戦士などは PhysicalAttack に統合される表記もあるが、
    /// Cor/Rng の専用枠を区別する場合に使用
    RangedAttack,
    /// 飛命 (遠隔命中)
    RangedAccuracy,
    StoreTp,
    SkillchainBonus,

    // ============ ジョブ特性効果アップ系 ============
    /// ダブルアタック確率アップ (% を直接加算)
    DoubleAttackRate,
    /// ダブルアタック効果アップ (DA 時のダメージ%)
    DoubleAttackEffect,
    /// トリプルアタック確率アップ
    TripleAttackRate,
    /// フェンサー効果アップ (TP ボーナス増)
    FencerEffect,
    /// C.インクリース効果アップ (与クリダメ%)
    CritIncreaseEffect,
    /// スマイト効果アップ
    SmiteEffect,
    /// BLU 専用: ジョブ特性効果アップ (rank +N)
    JobTraitEffectUp,

    // ============ クリ率/WS 系 ============
    /// クリティカルヒット確率アップ (% を直接加算)
    CriticalHitRate,
    /// ウェポンスキルダメージアップ (%)
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
    Gift::RangedAttack,
    Gift::RangedAccuracy,
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

impl Gift {
    /// JP 閾値リスト (このギフトを獲得可能なジョブで共通)。
    /// ティア 1 → thresholds[0], ティア 2 → thresholds[1], ...
    /// 空スライスはこのギフト種が未定義 (今後の追加待ち) を表す。
    pub fn thresholds(&self) -> &'static [i32] {
        match self {
            // 基本ステータス系: wiki によりジョブ共通の閾値
            Gift::PhysicalDefense => &[5, 180, 605, 1280],
            Gift::PhysicalAttack => &[10, 210, 660, 1360],
            Gift::PhysicalEvasion => &[20, 245, 720, 1445],
            Gift::PhysicalAccuracy => &[30, 280, 780, 1530],
            Gift::MagicEvasion => &[45, 320, 845, 1620],
            Gift::MagicAccuracy => &[60, 360, 910, 1710],
            Gift::MagicDefense => &[5, 180, 605, 1280],
            Gift::MagicAttack => &[10, 210, 660, 1360],
            Gift::RangedAttack => &[10, 210, 660, 1360],
            Gift::RangedAccuracy => &[30, 280, 780, 1530],

            // 連携ボーナス (Sam/Dnc): 暫定 (要 wiki 確認)
            Gift::SkillchainBonus => &[80, 405, 980, 1805],
            // ストアTP (Sam): 暫定
            Gift::StoreTp => &[],

            // 特性効果アップ系
            Gift::DoubleAttackRate => &[125, 450, 1050, 1900],
            Gift::DoubleAttackEffect => &[],
            Gift::TripleAttackRate => &[],
            Gift::FencerEffect => &[80, 405, 980, 1805],
            Gift::CritIncreaseEffect => &[150, 500, 1125, 2000],
            Gift::SmiteEffect => &[],
            Gift::JobTraitEffectUp => &[100, 1200],

            // クリ率/WS 系
            Gift::CriticalHitRate => &[100, 1200],
            Gift::WeaponSkillDamage => &[550],
        }
    }
}

impl Job {
    /// このジョブの (gift) 累積効果値テーブル。
    /// length は `gift.thresholds()` と一致する想定。空スライスは未獲得。
    pub fn gift_cumulative(&self, gift: Gift) -> &'static [i32] {
        gift_cumulative_table(*self, gift)
    }

    /// 指定 (gift, total_jp) で獲得済みのティア数を返す (0 = 未獲得)。
    pub fn gift_tier_at_jp(&self, gift: Gift, total_jp: i32) -> usize {
        if self.gift_cumulative(gift).is_empty() {
            return 0;
        }
        gift.thresholds()
            .iter()
            .filter(|&&req_jp| total_jp >= req_jp)
            .count()
    }

    /// このジョブ・指定 total_jp 時点でのギフト効果値 (累積)。
    pub fn gift_value(&self, gift: Gift, total_jp: i32) -> i32 {
        let tier = self.gift_tier_at_jp(gift, total_jp);
        if tier == 0 {
            return 0;
        }
        let cumulative = self.gift_cumulative(gift);
        let idx = std::cmp::min(tier, cumulative.len()) - 1;
        cumulative[idx]
    }
}

// ---------------------------------------------------------------------------
// (job, gift) → 累積効果値テーブル
// データソース: data/job_gifts.json (元) + 各ジョブの wiki ページ
//   War: https://wiki.ffo.jp/html/37176.html
//   Mnk: https://wiki.ffo.jp/html/37177.html
//   Whm: https://wiki.ffo.jp/html/37178.html
//   ... (~37197.html for Run)
// ---------------------------------------------------------------------------

fn gift_cumulative_table(job: Job, gift: Gift) -> &'static [i32] {
    use Gift::*;
    use Job::*;

    match (gift, job) {
        // ============ PhysicalDefense (累計 [+inc1, +inc1+inc2, ...]) ============
        (PhysicalDefense, War) => &[10, 25, 45, 70], // +10/+15/+20/+25
        (PhysicalDefense, Mnk) => &[5, 13, 23, 36],  // +5/+8/+10/+13
        (PhysicalDefense, Thf) => &[4, 10, 18, 28],  // +4/+6/+8/+10
        (PhysicalDefense, Pld) => &[15, 38, 68, 106], // +15/+23/+30/+38
        (PhysicalDefense, Drk) => &[4, 10, 18, 28],  // +4/+6/+8/+10
        (PhysicalDefense, Bst) => &[10, 25, 45, 70],
        (PhysicalDefense, Brd) => &[3, 8, 14, 22],   // +3/+5/+6/+8
        (PhysicalDefense, Rng) => &[3, 8, 14, 22],
        (PhysicalDefense, Sam) => &[6, 15, 27, 42],  // +6/+9/+12/+15
        (PhysicalDefense, Nin) => &[3, 8, 14, 22],
        (PhysicalDefense, Drg) => &[6, 15, 27, 42],
        (PhysicalDefense, Smn) => &[3, 8, 14, 22],
        (PhysicalDefense, Blu) => &[6, 15, 27, 42],
        (PhysicalDefense, Cor) => &[3, 8, 14, 22],
        (PhysicalDefense, Dnc) => &[6, 15, 27, 42],
        (PhysicalDefense, Run) => &[10, 25, 45, 70],

        // ============ PhysicalAttack ============
        (PhysicalAttack, War) => &[10, 25, 45, 70],
        (PhysicalAttack, Mnk) => &[8, 20, 36, 56],   // +8/+12/+16/+20
        (PhysicalAttack, Thf) => &[7, 18, 32, 50],   // +7/+11/+14/+18
        (PhysicalAttack, Pld) => &[4, 10, 18, 28],
        (PhysicalAttack, Drk) => &[15, 38, 68, 106],
        (PhysicalAttack, Bst) => &[10, 25, 45, 70],
        (PhysicalAttack, Rng) => &[10, 25, 45, 70],
        (PhysicalAttack, Sam) => &[10, 25, 45, 70],
        (PhysicalAttack, Nin) => &[8, 20, 36, 56],
        (PhysicalAttack, Drg) => &[10, 25, 45, 70],
        (PhysicalAttack, Blu) => &[8, 20, 36, 56],
        (PhysicalAttack, Cor) => &[8, 20, 36, 56],
        (PhysicalAttack, Pup) => &[8, 20, 36, 56],
        (PhysicalAttack, Dnc) => &[8, 20, 36, 56],
        (PhysicalAttack, Run) => &[8, 20, 36, 56],

        // ============ PhysicalEvasion ============
        (PhysicalEvasion, War) => &[5, 13, 23, 36],
        (PhysicalEvasion, Mnk) => &[6, 15, 27, 42],
        (PhysicalEvasion, Thf) => &[10, 25, 45, 70],
        (PhysicalEvasion, Pld) => &[3, 8, 14, 22],
        (PhysicalEvasion, Drk) => &[3, 8, 14, 22],
        (PhysicalEvasion, Bst) => &[5, 13, 23, 36],
        (PhysicalEvasion, Brd) => &[5, 13, 23, 36],
        (PhysicalEvasion, Rng) => &[5, 13, 23, 36],
        (PhysicalEvasion, Sam) => &[3, 8, 14, 22],
        (PhysicalEvasion, Nin) => &[8, 20, 36, 56],
        (PhysicalEvasion, Drg) => &[3, 8, 14, 22],
        (PhysicalEvasion, Smn) => &[3, 8, 14, 22],
        (PhysicalEvasion, Blu) => &[5, 13, 23, 36],
        (PhysicalEvasion, Cor) => &[5, 13, 23, 36],
        (PhysicalEvasion, Pup) => &[5, 13, 23, 36],
        (PhysicalEvasion, Dnc) => &[8, 20, 36, 56],
        (PhysicalEvasion, Run) => &[5, 13, 23, 36],

        // ============ PhysicalAccuracy ============
        (PhysicalAccuracy, War) => &[5, 13, 23, 36],
        (PhysicalAccuracy, Mnk) => &[6, 15, 26, 41],  // +6/+9/+11/+15
        (PhysicalAccuracy, Whm) => &[2, 5, 9, 14],    // +2/+3/+4/+5
        (PhysicalAccuracy, Rdm) => &[3, 8, 14, 22],
        (PhysicalAccuracy, Thf) => &[5, 13, 23, 36],
        (PhysicalAccuracy, Pld) => &[4, 10, 18, 28],
        (PhysicalAccuracy, Drk) => &[3, 8, 14, 22],
        (PhysicalAccuracy, Bst) => &[5, 13, 23, 36],
        (PhysicalAccuracy, Brd) => &[3, 8, 14, 22],
        (PhysicalAccuracy, Rng) => &[10, 25, 45, 70],
        (PhysicalAccuracy, Sam) => &[6, 15, 27, 42],
        (PhysicalAccuracy, Nin) => &[5, 13, 23, 36],
        (PhysicalAccuracy, Drg) => &[5, 13, 23, 36],
        (PhysicalAccuracy, Blu) => &[5, 13, 23, 36],
        (PhysicalAccuracy, Cor) => &[6, 15, 27, 42],
        (PhysicalAccuracy, Pup) => &[5, 13, 23, 36],
        (PhysicalAccuracy, Dnc) => &[6, 15, 27, 42],
        (PhysicalAccuracy, Run) => &[5, 13, 23, 36],

        // ============ MagicEvasion ============
        (MagicEvasion, War) => &[5, 13, 23, 36],
        (MagicEvasion, Mnk) => &[5, 13, 23, 36],
        (MagicEvasion, Whm) => &[7, 18, 32, 50],
        (MagicEvasion, Blm) => &[6, 15, 27, 42],
        (MagicEvasion, Rdm) => &[8, 20, 36, 56],
        (MagicEvasion, Thf) => &[5, 13, 23, 36],
        (MagicEvasion, Pld) => &[6, 15, 27, 42],
        (MagicEvasion, Drk) => &[6, 15, 27, 42],
        (MagicEvasion, Bst) => &[5, 13, 23, 36],
        (MagicEvasion, Brd) => &[5, 13, 23, 36],
        (MagicEvasion, Rng) => &[5, 13, 23, 36],
        (MagicEvasion, Sam) => &[5, 13, 23, 36],
        (MagicEvasion, Drg) => &[5, 13, 23, 36],
        (MagicEvasion, Smn) => &[6, 15, 27, 42],
        (MagicEvasion, Cor) => &[5, 13, 23, 36],
        (MagicEvasion, Pup) => &[5, 13, 23, 36],
        (MagicEvasion, Dnc) => &[5, 13, 23, 36],
        (MagicEvasion, Sch) => &[6, 15, 27, 42],
        (MagicEvasion, Geo) => &[6, 15, 27, 42],
        (MagicEvasion, Run) => &[6, 15, 27, 42],

        // ============ MagicAccuracy ============
        (MagicAccuracy, War) => &[5, 13, 23, 36],
        (MagicAccuracy, Mnk) => &[5, 13, 23, 36],
        (MagicAccuracy, Whm) => &[7, 18, 32, 50],
        (MagicAccuracy, Blm) => &[6, 15, 27, 42],
        (MagicAccuracy, Rdm) => &[10, 25, 45, 70],
        (MagicAccuracy, Thf) => &[5, 13, 23, 36],
        (MagicAccuracy, Pld) => &[6, 15, 27, 42],
        (MagicAccuracy, Drk) => &[6, 15, 27, 42],
        (MagicAccuracy, Bst) => &[5, 13, 23, 36],
        (MagicAccuracy, Brd) => &[5, 13, 23, 36],
        (MagicAccuracy, Rng) => &[5, 13, 23, 36],
        (MagicAccuracy, Sam) => &[5, 13, 23, 36],
        (MagicAccuracy, Nin) => &[5, 13, 23, 36],
        (MagicAccuracy, Drg) => &[5, 13, 23, 36],
        (MagicAccuracy, Blu) => &[5, 13, 23, 36],
        (MagicAccuracy, Cor) => &[5, 13, 23, 36],
        (MagicAccuracy, Pup) => &[5, 13, 23, 36],
        (MagicAccuracy, Dnc) => &[5, 13, 23, 36],
        (MagicAccuracy, Sch) => &[6, 15, 27, 42],
        (MagicAccuracy, Geo) => &[6, 15, 27, 42],
        (MagicAccuracy, Run) => &[5, 13, 23, 36],

        // ============ MagicDefense ============
        (MagicDefense, Whm) => &[7, 18, 32, 50],
        (MagicDefense, Blm) => &[2, 5, 9, 14],
        (MagicDefense, Rdm) => &[4, 10, 18, 28],
        (MagicDefense, Brd) => &[5, 13, 23, 36],
        (MagicDefense, Smn) => &[5, 13, 23, 36],
        (MagicDefense, Sch) => &[5, 13, 23, 36],
        (MagicDefense, Geo) => &[5, 13, 23, 36],
        (MagicDefense, Run) => &[5, 13, 23, 36],

        // ============ MagicAttack ============
        (MagicAttack, Whm) => &[3, 8, 14, 22],
        (MagicAttack, Blm) => &[7, 18, 32, 50],
        (MagicAttack, Rdm) => &[4, 10, 18, 28],
        (MagicAttack, Nin) => &[3, 8, 14, 22],   // 暫定
        (MagicAttack, Cor) => &[3, 8, 14, 22],   // 暫定
        (MagicAttack, Sch) => &[7, 18, 32, 50],
        (MagicAttack, Geo) => &[7, 18, 32, 50],

        // ============ SkillchainBonus (Sam/Dnc) ============
        // 80/405/980/1805 で +2/+2/+2/+2 (累積 +2/+4/+6/+8)
        (SkillchainBonus, Sam) => &[2, 4, 6, 8],
        (SkillchainBonus, Dnc) => &[2, 4, 6, 8],

        // ============ DoubleAttackRate (戦士) ============
        // 125/450/1050/1900 で +2/+2/+3/+3
        (DoubleAttackRate, War) => &[2, 4, 7, 10],

        // ============ FencerEffect (戦士のみ) ============
        // 80/405/980/1805 で +50/+50/+60/+70
        (FencerEffect, War) => &[50, 100, 160, 230],

        // ============ CritIncreaseEffect (戦士) ============
        // 150/500/1125/2000 で +2/+2/+3/+3
        (CritIncreaseEffect, War) => &[2, 4, 7, 10],

        // ============ CriticalHitRate (戦士) ============
        // 100/1200 で +5/+5
        (CriticalHitRate, War) => &[5, 10],

        // ============ WeaponSkillDamage (戦士) ============
        // 550 で +3
        (WeaponSkillDamage, War) => &[3],

        // ============ JobTraitEffectUp (BLU) ============
        // 100/1200 で +1/+2 rank
        (JobTraitEffectUp, Blu) => &[1, 2],

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
                let _ = job.gift_cumulative(g);
                let _ = g.thresholds();
                for jp in [0, 50, 100, 500, 1200, 2100] {
                    let _ = job.gift_value(g, jp);
                }
            }
        }
    }

    /// 構造テスト: gift_cumulative の長さは thresholds の長さ以下であること
    /// (空はOK = 未獲得)
    #[test]
    fn test_gift_cumulative_length_matches_thresholds() {
        for job in Job::iter() {
            for &g in ALL_GIFTS {
                let cumulative = job.gift_cumulative(g);
                let thresholds = g.thresholds();
                if !cumulative.is_empty() {
                    assert_eq!(
                        cumulative.len(),
                        thresholds.len(),
                        "{:?} {:?}: cumulative={} but thresholds={}",
                        job,
                        g,
                        cumulative.len(),
                        thresholds.len()
                    );
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
        // 物理防御力 +25 (累計 70)
        assert_eq!(Job::War.gift_value(Gift::PhysicalDefense, total_jp), 70);
        assert_eq!(Job::War.gift_value(Gift::PhysicalAttack, total_jp), 70);
        assert_eq!(Job::War.gift_value(Gift::PhysicalEvasion, total_jp), 36);
        assert_eq!(Job::War.gift_value(Gift::PhysicalAccuracy, total_jp), 36);
        assert_eq!(Job::War.gift_value(Gift::MagicEvasion, total_jp), 36);
        assert_eq!(Job::War.gift_value(Gift::MagicAccuracy, total_jp), 36);
        // ダブルアタック確率: +2+2+3+3 = +10
        assert_eq!(Job::War.gift_value(Gift::DoubleAttackRate, total_jp), 10);
        // フェンサー効果アップ: 累計 230
        assert_eq!(Job::War.gift_value(Gift::FencerEffect, total_jp), 230);
        // C.インクリース効果アップ: +2+2+3+3 = +10
        assert_eq!(Job::War.gift_value(Gift::CritIncreaseEffect, total_jp), 10);
        // クリティカルヒット確率: +5+5 = +10
        assert_eq!(Job::War.gift_value(Gift::CriticalHitRate, total_jp), 10);
        // WS ダメージ: +3
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

    /// パラディン 2100 JP (PDEF が高い)
    #[test]
    fn test_pld_full_jp() {
        let total_jp = 2100;
        assert_eq!(Job::Pld.gift_value(Gift::PhysicalDefense, total_jp), 106);
        assert_eq!(Job::Pld.gift_value(Gift::PhysicalAttack, total_jp), 28);
    }

    /// 暗黒騎士 2100 JP (PATK が高い)
    #[test]
    fn test_drk_full_jp() {
        let total_jp = 2100;
        assert_eq!(Job::Drk.gift_value(Gift::PhysicalAttack, total_jp), 106);
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

    /// BLU の ジョブ特性効果アップ (旧 blu_trait_effect_up_bonus_ranks 相当)
    #[test]
    fn test_blu_job_trait_effect_up() {
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 0), 0);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 99), 0);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 100), 1);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 1199), 1);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 1200), 2);
        assert_eq!(Job::Blu.gift_value(Gift::JobTraitEffectUp, 2100), 2);
        // BLU 以外は 0
        assert_eq!(Job::War.gift_value(Gift::JobTraitEffectUp, 2100), 0);
    }
}
