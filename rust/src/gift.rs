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
    /// モクシャ効果アップ (相手に与える TP 減少 % 増)
    SubtleBlow,
    /// マーシャルアーツ効果アップ (格闘の攻撃間隔減算)
    MartialArtsEffect,
    /// カウンター効果アップ (反撃発動確率 %)
    CounterRate,
    /// カウンターダメージアップ (カウンター時のダメージ %)
    CounterDamage,
    /// ケアル回復量アップ (ケアル系の絶対値加算)
    CureAmount,
    /// 回復魔法詠唱時間短縮 (% 減算、回復魔法のみ)
    HealingMagicCastTime,
    /// リジェネ回復量アップ
    RegenAmount,
    /// マジックバーストダメージアップ (MB ダメージ %)
    MagicBurstDamage,
    /// 魔法ダメージアップ (魔法ダメージ絶対値加算)
    MagicDamage,
    /// エレメントセレリティ効果アップ (精霊魔法詠唱時間短縮 % 追加)
    ElementalCelerityEffect,
    /// BLU 専用: ジョブ特性効果アップ (rank +N)
    JobTraitEffectUp,

    // ============ クリ率/WS 系 ============
    CriticalHitRate,
    WeaponSkillDamage,

    // ============ スキル系 ============
    /// ガードスキルアップ (絶対値の skill 加算)
    GuardSkill,
    /// 回復魔法スキルアップ
    HealingMagicSkill,
    /// 神聖魔法スキルアップ
    DivineMagicSkill,
    /// 精霊魔法スキルアップ
    ElementalMagicSkill,
    /// 暗黒魔法スキルアップ
    DarkMagicSkill,
    /// 強化魔法スキルアップ (Rdm/Sch/Run)
    EnhancingMagicSkill,
    /// 弱体魔法スキルアップ (Rdm)
    EnfeeblingMagicSkill,
    /// 召喚魔法スキルアップ (Smn)
    SummoningMagicSkill,
    /// 青魔法スキルアップ (Blu)
    BlueMagicSkill,
    /// 忍術スキルアップ (Nin)
    NinjutsuSkill,
    /// 風水魔法スキルアップ (Geo)
    GeomancySkill,
    /// 風水鈴スキルアップ (Geo)
    GeomanticBellSkill,
    /// 歌唱スキルアップ (Brd)
    SingingSkill,
    /// 弦楽器スキルアップ (Brd)
    StringInstrumentSkill,
    /// 管楽器スキルアップ (Brd)
    WindInstrumentSkill,

    // ============ ジョブ固有の特殊効果アップ系 ============
    /// 青魔法効果アップ (Blu)
    BlueMagicEffect,
    /// 魔法剣ダメージアップ (Rdm)
    EnspellEffect,
    /// ファストキャスト効果アップ (Rdm, % 短縮)
    FastCastEffect,
    /// インクァルタタ効果アップ (Run, 受け流し率 %)
    InquartataEffect,
    /// 被強化魔法効果時間延長 (Run, 自身が受けた強化魔法の効果時間 %)
    EnhanceMagicDurationOnSelf,
    /// 残心発動率アップ (Sam, %)
    ZanshinRate,
    /// 心眼効果アップ (Sam, よける回数 +N)
    SeiganEffect,
    /// 八双・星眼効果アップ (Sam, 残心/カウンター確率の上限 +N%)
    HassoSeiganEffect,
    /// トレジャーハンター効果アップ (Thf, ランク +N)
    TreasureHunterEffect,
    /// シールドマスタリー効果アップ (Pld, TP ボーナス %)
    ShieldMasteryEffect,
    /// C.リデュース効果アップ (Pld, 被クリダメージ % 軽減)
    CritReduceEffect,
    /// プロテス効果アップ (Pld, 防御力 +N)
    ProtesEffect,
    /// ドレッドスパイク効果アップ (Drk, 吸収量 +N)
    DreadSpikeEffect,
    /// コンサーブTP効果アップ (Rng, %)
    ConserveTpEffect,
    /// ベロシティショット効果アップ (Rng, % 短縮)
    VelocityShotEffect,
    /// トゥルーショット効果アップ (Rng/Cor, %)
    TrueshotEffect,
    /// 乱れ撃ち効果アップ (Rng, 弾数 +N)
    BarrageEffect,
    /// 打剣効果アップ (Nin, %)
    ShurikenThrowEffect,
    /// スナップショット効果アップ (Cor, % 短縮)
    SnapshotEffect,
    /// 矢弾消費量軽減 (Cor, 不消費確率 %)
    AmmoCostReduction,
    /// クイックドロー再使用時間短縮 (Cor, -秒)
    QuickDrawRecast,
    /// 二刀流効果アップ (Thf/Dnc, 攻撃間隔 % 短縮)
    DualWieldEffect,
    /// フィニシングムーブ最大値アップ (Dnc, +N)
    FinishingMoveCount,
    /// 歌の詠唱時間短縮 (Brd, % 短縮)
    SongCastTime,
    /// 歌の効果時間延長 (Brd, %)
    SongEffectDuration,

    // ============ ペット系 (Bst/Smn/Pup/Drg) ============
    /// ペットの物理攻撃力・物理防御力アップ (Bst)
    PetPhysicalAtkDef,
    /// ペットの物理命中・物理回避アップ (Bst)
    PetPhysicalAccEva,
    /// ペットステータスアップ (Bst, 全ステ +N)
    PetStatus,
    /// ペットのTPボーナス (Bst, +N)
    PetTpBonus,
    /// 召喚獣の物理攻撃力・物理防御力アップ (Smn)
    AvatarPhysicalAtkDef,
    /// 召喚獣の物理命中・物理回避アップ (Smn)
    AvatarPhysicalAccEva,
    /// 召喚獣の魔法攻撃力・魔法防御力アップ (Smn)
    AvatarMagicalAtkDef,
    /// 召喚獣・神獣の魔法命中・魔法回避アップ (Smn)
    AvatarMagicalAccEva,
    /// 神獣の加護効果アップ (Smn)
    AvatarBlessingEffect,
    /// オートマトンの物理攻撃力・物理防御力アップ (Pup)
    AutomatonPhysicalAtkDef,
    /// オートマトンの物理命中・物理回避アップ (Pup)
    AutomatonPhysicalAccEva,
    /// オートマトンの魔法攻撃力・魔法防御力アップ (Pup)
    AutomatonMagicalAtkDef,
    /// オートマトンの魔法命中・魔法回避アップ (Pup)
    AutomatonMagicalAccEva,
    /// オートマトンの属性値アップ (Pup)
    AutomatonElementBoost,
    /// ワイバーンのステータスアップ時の効果アップ (Drg)
    WyvernBoostEffect,
    /// ワイバーンの物理命中・物理回避アップ (Drg)
    WyvernPhysicalAccEva,
    /// ワイバーンの魔法命中・魔法回避アップ (Drg)
    WyvernMagicalAccEva,
    /// ブレス再使用時間短縮 (Drg, -秒)
    BreathRecast,
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
    Gift::SubtleBlow,
    Gift::MartialArtsEffect,
    Gift::CounterRate,
    Gift::CounterDamage,
    Gift::CureAmount,
    Gift::HealingMagicCastTime,
    Gift::RegenAmount,
    Gift::MagicBurstDamage,
    Gift::MagicDamage,
    Gift::ElementalCelerityEffect,
    Gift::JobTraitEffectUp,
    Gift::CriticalHitRate,
    Gift::WeaponSkillDamage,
    Gift::GuardSkill,
    Gift::HealingMagicSkill,
    Gift::DivineMagicSkill,
    Gift::ElementalMagicSkill,
    Gift::DarkMagicSkill,
    Gift::EnhancingMagicSkill,
    Gift::EnfeeblingMagicSkill,
    Gift::SummoningMagicSkill,
    Gift::BlueMagicSkill,
    Gift::NinjutsuSkill,
    Gift::GeomancySkill,
    Gift::GeomanticBellSkill,
    Gift::SingingSkill,
    Gift::StringInstrumentSkill,
    Gift::WindInstrumentSkill,
    Gift::BlueMagicEffect,
    Gift::EnspellEffect,
    Gift::FastCastEffect,
    Gift::InquartataEffect,
    Gift::EnhanceMagicDurationOnSelf,
    Gift::ZanshinRate,
    Gift::SeiganEffect,
    Gift::HassoSeiganEffect,
    Gift::TreasureHunterEffect,
    Gift::ShieldMasteryEffect,
    Gift::CritReduceEffect,
    Gift::ProtesEffect,
    Gift::DreadSpikeEffect,
    Gift::ConserveTpEffect,
    Gift::VelocityShotEffect,
    Gift::TrueshotEffect,
    Gift::BarrageEffect,
    Gift::ShurikenThrowEffect,
    Gift::SnapshotEffect,
    Gift::AmmoCostReduction,
    Gift::QuickDrawRecast,
    Gift::DualWieldEffect,
    Gift::FinishingMoveCount,
    Gift::SongCastTime,
    Gift::SongEffectDuration,
    Gift::PetPhysicalAtkDef,
    Gift::PetPhysicalAccEva,
    Gift::PetStatus,
    Gift::PetTpBonus,
    Gift::AvatarPhysicalAtkDef,
    Gift::AvatarPhysicalAccEva,
    Gift::AvatarMagicalAtkDef,
    Gift::AvatarMagicalAccEva,
    Gift::AvatarBlessingEffect,
    Gift::AutomatonPhysicalAtkDef,
    Gift::AutomatonPhysicalAccEva,
    Gift::AutomatonMagicalAtkDef,
    Gift::AutomatonMagicalAccEva,
    Gift::AutomatonElementBoost,
    Gift::WyvernBoostEffect,
    Gift::WyvernPhysicalAccEva,
    Gift::WyvernMagicalAccEva,
    Gift::BreathRecast,
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

        // ============ SubtleBlow (モクシャ効果アップ) ============
        // Mnk: 125/450/1050/1900 で -2/-2/-3/-3% (累積 +2/+4/+7/+10)
        (SubtleBlow, Mnk) => &[(125, 2), (450, 4), (1050, 7), (1900, 10)],
        // Dnc: 80/360/910/1710 で +3/+3/+3/+4 (累積 +3/+6/+9/+13)
        (SubtleBlow, Dnc) => &[(80, 3), (360, 6), (910, 9), (1710, 13)],

        // ============ MartialArtsEffect (Mnk のみ) ============
        // 100/1200 で 格闘攻撃間隔 -5/-5 (累積 -5/-10)
        (MartialArtsEffect, Mnk) => &[(100, -5), (1200, -10)],

        // ============ CounterRate (Mnk のみ) ============
        // 150/500/1125/2000 で 反撃確率 +2/+2/+3/+3% (累積 +2/+4/+7/+10)
        (CounterRate, Mnk) => &[(150, 2), (500, 4), (1125, 7), (2000, 10)],

        // ============ CounterDamage (Mnk のみ) ============
        // 550 で カウンターダメージ +5% (1 ティアのみ)
        (CounterDamage, Mnk) => &[(550, 5)],

        // ============ GuardSkill (Mnk のみ) ============
        // 80/405/980/1805 で +5/+8/+10/+13 (累積 +5/+13/+23/+36)
        (GuardSkill, Mnk) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],

        // ============ CureAmount (ケアル回復量アップ, Whm のみ) ============
        // 125/450/1050/1900 で +5/+5/+6/+7 (累積 +5/+10/+16/+23)
        (CureAmount, Whm) => &[(125, 5), (450, 10), (1050, 16), (1900, 23)],

        // ============ HealingMagicCastTime (回復魔法詠唱時間短縮, Whm のみ) ============
        // 150/500/1125/2000 で -2/-2/-2/-2% (累積 -2/-4/-6/-8)
        (HealingMagicCastTime, Whm) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],

        // ============ RegenAmount (リジェネ回復量アップ, Whm のみ) ============
        // 550 で +5 (1 ティア)
        (RegenAmount, Whm) => &[(550, 5)],

        // ============ HealingMagicSkill (回復魔法スキル, Whm のみ) ============
        // 60/360/910/1710 で +5/+8/+10/+13 (累積 +5/+13/+23/+36)
        (HealingMagicSkill, Whm) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],

        // ============ DivineMagicSkill (神聖魔法スキル, Whm のみ) ============
        // 80/405/980/1805 で +5/+8/+10/+13 (累積 +5/+13/+23/+36)
        (DivineMagicSkill, Whm) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],

        // ============ MagicBurstDamage (マジックバーストダメージ, Blm のみ) ============
        // 80/405/980/1805 で +5/+5/+6/+7% (累積 +5/+10/+16/+23)
        (MagicBurstDamage, Blm) => &[(80, 5), (405, 10), (980, 16), (1805, 23)],

        // ============ MagicDamage (魔法ダメージ, Blm のみ) ============
        // 125/450/1050/1900 で +5/+5/+6/+7 (累積 +5/+10/+16/+23)
        (MagicDamage, Blm) => &[(125, 5), (450, 10), (1050, 16), (1900, 23)],

        // ============ ElementalCelerityEffect (Blm のみ) ============
        // 150/500/1125/2000 で -2/-2/-2/-2% (累積 -2/-4/-6/-8)
        (ElementalCelerityEffect, Blm) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],

        // ============ ElementalMagicSkill (精霊魔法スキル, Blm のみ) ============
        // 45/320/845/1620 で +5/+8/+10/+13 (累積 +5/+13/+23/+36)
        (ElementalMagicSkill, Blm) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],

        // ============ DarkMagicSkill (暗黒魔法スキル, Blm のみ) ============
        // 60/360/910/1710 で +5/+8/+10/+13 (累積 +5/+13/+23/+36)
        (DarkMagicSkill, Blm) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],

        // ============ JobTraitEffectUp (BLU) ============
        // 100/1200 で +1/+2 rank
        (JobTraitEffectUp, Blu) => &[(100, 1), (1200, 2)],

        // ============ 既存 enum の他ジョブ追加 ============
        // ダブルアタック効果アップ (戦士 JP=125 で +2%, 1 ティア)
        (DoubleAttackEffect, War) => &[(125, 2)],

        // トリプルアタック確率アップ (Thf)
        (TripleAttackRate, Thf) => &[(125, 2), (450, 4), (1050, 6), (1900, 8)],

        // フェンサー効果アップ (Bst, TPボーナス +50/+50/+60/+70)
        (FencerEffect, Bst) => &[(150, 50), (500, 100), (1125, 160), (2000, 230)],

        // C.インクリース効果アップ (Thf, Drk, Rng, Dnc, Drg)
        (CritIncreaseEffect, Thf) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],
        (CritIncreaseEffect, Drk) => &[(125, 2), (450, 4), (1050, 6), (1900, 8)],
        (CritIncreaseEffect, Rng) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],
        (CritIncreaseEffect, Dnc) => &[(125, 2), (450, 4), (1050, 6), (1900, 8)],
        // Drg「クリティカルダメージアップ」(意味は同じ与クリダメ %)
        (CritIncreaseEffect, Drg) => &[(80, 2), (405, 4), (980, 6), (1805, 8)],

        // WS ダメージアップ (Drk, Nin)
        (WeaponSkillDamage, Drk) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],
        (WeaponSkillDamage, Nin) => &[(1200, 5)],

        // 神聖魔法スキル (Pld)
        (DivineMagicSkill, Pld) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],
        // ケアル回復量 (Pld JP=1200 で +50, 1 ティア)
        (CureAmount, Pld) => &[(1200, 50)],

        // 回復魔法スキル (Sch)
        (HealingMagicSkill, Sch) => &[(125, 5), (450, 13), (1050, 23), (1900, 36)],

        // 精霊魔法スキル (Sch, Geo)
        (ElementalMagicSkill, Sch) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],
        (ElementalMagicSkill, Geo) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],

        // 暗黒魔法スキル (Drk, Sch, Geo)
        (DarkMagicSkill, Drk) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],
        (DarkMagicSkill, Sch) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (DarkMagicSkill, Geo) => &[(125, 5), (450, 13), (1050, 23), (1900, 36)],

        // マジックバーストダメージ (Sch)
        (MagicBurstDamage, Sch) => &[(150, 3), (500, 6), (1125, 9), (2000, 13)],

        // 魔法ダメージ (Geo)
        (MagicDamage, Geo) => &[(150, 3), (500, 6), (1125, 9), (2000, 13)],

        // マーシャルアーツ (Pup, JP=550 で -5, 1 ティア)
        (MartialArtsEffect, Pup) => &[(550, -5)],

        // ストアTP (Sam)
        (StoreTp, Sam) => &[(125, 2), (450, 4), (1050, 6), (1900, 8)],

        // 青魔の MDEF/MATK/MEVA (slot がずれる: MDEF=4, MATK=5, MEVA=6)
        (MagicDefense, Blu) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (MagicAttack, Blu) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (MagicEvasion, Blu) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],

        // ============ 新規 enum: スキル系 ============
        (EnhancingMagicSkill, Rdm) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],
        (EnhancingMagicSkill, Sch) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],
        (EnhancingMagicSkill, Run) => &[(125, 5), (450, 13), (1050, 23), (1900, 36)],

        (EnfeeblingMagicSkill, Rdm) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],

        (SummoningMagicSkill, Smn) => &[(150, 5), (500, 13), (1125, 23), (2000, 36)],

        (BlueMagicSkill, Blu) => &[(150, 5), (500, 13), (1125, 23), (2000, 36)],

        (NinjutsuSkill, Nin) => &[(125, 5), (450, 13), (1050, 23), (1900, 36)],

        (GeomancySkill, Geo) => &[(45, 5), (320, 13), (845, 23), (1620, 36)],
        (GeomanticBellSkill, Geo) => &[(60, 5), (360, 13), (910, 23), (1710, 36)],

        (SingingSkill, Brd) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],
        (StringInstrumentSkill, Brd) => &[(125, 5), (450, 13), (1050, 23), (1900, 36)],
        (WindInstrumentSkill, Brd) => &[(150, 5), (500, 13), (1125, 23), (2000, 36)],

        // ============ 新規 enum: ジョブ固有効果アップ系 ============
        (BlueMagicEffect, Blu) => &[(550, 5)],

        (EnspellEffect, Rdm) => &[(125, 5), (450, 10), (1050, 16), (1900, 23)],
        (FastCastEffect, Rdm) => &[(150, 1), (500, 2), (1125, 3), (2000, 4)],

        (InquartataEffect, Run) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],
        (EnhanceMagicDurationOnSelf, Run) => &[(100, 10), (1200, 20)],

        (ZanshinRate, Sam) => &[(80, 2), (405, 4), (980, 7), (1805, 10)],
        (SeiganEffect, Sam) => &[(550, 1)],
        (HassoSeiganEffect, Sam) => &[(100, 5), (1200, 10)],

        (TreasureHunterEffect, Thf) => &[(80, 1), (405, 2), (980, 3), (1805, 4)],

        (ShieldMasteryEffect, Pld) => &[(125, 3), (450, 6), (1050, 10), (1900, 15)],
        (CritReduceEffect, Pld) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],
        (ProtesEffect, Pld) => &[(550, 10)],

        (DreadSpikeEffect, Drk) => &[(1200, 20)],

        (ConserveTpEffect, Rng) => &[(80, 3), (405, 6), (980, 10), (1805, 15)],
        (VelocityShotEffect, Rng) => &[(100, 5), (1200, 10)],
        (TrueshotEffect, Rng) => &[(125, 2), (450, 4), (1050, 6), (1900, 8)],
        (TrueshotEffect, Cor) => &[(150, 2), (500, 4), (1125, 6), (2000, 8)],
        (BarrageEffect, Rng) => &[(550, 1)],

        (ShurikenThrowEffect, Nin) => &[(150, 2), (500, 5), (1125, 9), (2000, 14)],

        (SnapshotEffect, Cor) => &[(100, 5), (1200, 10)],
        (AmmoCostReduction, Cor) => &[(125, 2), (450, 4), (1050, 6), (1900, 8)],
        (QuickDrawRecast, Cor) => &[(550, 10)],

        (DualWieldEffect, Thf) => &[(550, 5)],
        (DualWieldEffect, Dnc) => &[(550, 5)],

        (FinishingMoveCount, Dnc) => &[(100, 2), (1200, 4)],

        (SongCastTime, Brd) => &[(550, 5)],
        (SongEffectDuration, Brd) => &[(1200, 5)],

        // ============ 新規 enum: ペット系 ============
        (PetPhysicalAtkDef, Bst) => &[(80, 15), (405, 38), (980, 68), (1805, 106)],
        (PetPhysicalAccEva, Bst) => &[(125, 10), (450, 25), (1050, 45), (1900, 70)],
        (PetStatus, Bst) => &[(550, 8)],
        (PetTpBonus, Bst) => &[(1200, 40)],

        (AvatarPhysicalAtkDef, Smn) => &[(45, 15), (320, 38), (845, 68), (1620, 106)],
        (AvatarPhysicalAccEva, Smn) => &[(60, 10), (360, 25), (910, 45), (1710, 70)],
        (AvatarMagicalAtkDef, Smn) => &[(80, 5), (405, 13), (980, 23), (1805, 36)],
        (AvatarMagicalAccEva, Smn) => &[(125, 10), (450, 25), (1050, 45), (1900, 70)],
        (AvatarBlessingEffect, Smn) => &[(550, 1)],

        (AutomatonPhysicalAtkDef, Pup) => &[(60, 15), (360, 38), (910, 68), (1710, 106)],
        (AutomatonPhysicalAccEva, Pup) => &[(80, 10), (405, 25), (980, 45), (1805, 70)],
        (AutomatonMagicalAtkDef, Pup) => &[(125, 5), (450, 13), (1050, 23), (1900, 36)],
        (AutomatonMagicalAccEva, Pup) => &[(150, 10), (500, 25), (1125, 45), (2000, 70)],
        (AutomatonElementBoost, Pup) => &[(100, 2), (1200, 4)],

        (WyvernBoostEffect, Drg) => &[(100, 1), (1200, 2)],
        (WyvernPhysicalAccEva, Drg) => &[(125, 10), (450, 25), (1050, 45), (1900, 70)],
        (WyvernMagicalAccEva, Drg) => &[(150, 10), (500, 25), (1125, 45), (2000, 70)],
        (BreathRecast, Drg) => &[(550, 10)],

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

            // ========= SubtleBlow (モクシャ効果アップ) =========
            (SubtleBlow, Mnk) => 10,
            (SubtleBlow, Dnc) => 13,

            // ========= MNK 専用ギフト =========
            (MartialArtsEffect, Mnk) => -10,
            (CounterRate, Mnk) => 10,
            (CounterDamage, Mnk) => 5,
            (GuardSkill, Mnk) => 36,

            // ========= WHM 専用ギフト =========
            (CureAmount, Whm) => 23,
            (HealingMagicCastTime, Whm) => 8,
            (RegenAmount, Whm) => 5,
            (HealingMagicSkill, Whm) => 36,
            (DivineMagicSkill, Whm) => 36,

            // ========= BLM 専用ギフト =========
            (MagicBurstDamage, Blm) => 23,
            (MagicDamage, Blm) => 23,
            (ElementalCelerityEffect, Blm) => 8,
            (ElementalMagicSkill, Blm) => 36,
            (DarkMagicSkill, Blm) => 36,

            // ========= BLU JobTraitEffectUp =========
            (JobTraitEffectUp, Blu) => 2,

            // ========= 既存 enum 追加分 (新規 (job, gift) ペア) =========
            (DoubleAttackEffect, War) => 2,
            (TripleAttackRate, Thf) => 8,
            (FencerEffect, Bst) => 230,
            (CritIncreaseEffect, Thf) => 8,
            (CritIncreaseEffect, Drk) => 8,
            (CritIncreaseEffect, Rng) => 8,
            (CritIncreaseEffect, Dnc) => 8,
            (CritIncreaseEffect, Drg) => 8,
            (WeaponSkillDamage, Drk) => 8,
            (WeaponSkillDamage, Nin) => 5,
            (DivineMagicSkill, Pld) => 36,
            (CureAmount, Pld) => 50,
            (HealingMagicSkill, Sch) => 36,
            (ElementalMagicSkill, Sch) => 36,
            (ElementalMagicSkill, Geo) => 36,
            (DarkMagicSkill, Drk) => 36,
            (DarkMagicSkill, Sch) => 36,
            (DarkMagicSkill, Geo) => 36,
            (MagicBurstDamage, Sch) => 13,
            (MagicDamage, Geo) => 13,
            (MartialArtsEffect, Pup) => -5,
            (StoreTp, Sam) => 8,
            (MagicDefense, Blu) => 36,
            (MagicAttack, Blu) => 36,
            (MagicEvasion, Blu) => 36,

            // ========= 新規 enum: スキル系 =========
            (EnhancingMagicSkill, Rdm) => 36,
            (EnhancingMagicSkill, Sch) => 36,
            (EnhancingMagicSkill, Run) => 36,
            (EnfeeblingMagicSkill, Rdm) => 36,
            (SummoningMagicSkill, Smn) => 36,
            (BlueMagicSkill, Blu) => 36,
            (NinjutsuSkill, Nin) => 36,
            (GeomancySkill, Geo) => 36,
            (GeomanticBellSkill, Geo) => 36,
            (SingingSkill, Brd) => 36,
            (StringInstrumentSkill, Brd) => 36,
            (WindInstrumentSkill, Brd) => 36,

            // ========= 新規 enum: ジョブ固有効果アップ系 =========
            (BlueMagicEffect, Blu) => 5,
            (EnspellEffect, Rdm) => 23,
            (FastCastEffect, Rdm) => 4,
            (InquartataEffect, Run) => 8,
            (EnhanceMagicDurationOnSelf, Run) => 20,
            (ZanshinRate, Sam) => 10,
            (SeiganEffect, Sam) => 1,
            (HassoSeiganEffect, Sam) => 10,
            (TreasureHunterEffect, Thf) => 4,
            (ShieldMasteryEffect, Pld) => 15,
            (CritReduceEffect, Pld) => 8,
            (ProtesEffect, Pld) => 10,
            (DreadSpikeEffect, Drk) => 20,
            (ConserveTpEffect, Rng) => 15,
            (VelocityShotEffect, Rng) => 10,
            (TrueshotEffect, Rng) => 8,
            (TrueshotEffect, Cor) => 8,
            (BarrageEffect, Rng) => 1,
            (ShurikenThrowEffect, Nin) => 14,
            (SnapshotEffect, Cor) => 10,
            (AmmoCostReduction, Cor) => 8,
            (QuickDrawRecast, Cor) => 10,
            (DualWieldEffect, Thf) => 5,
            (DualWieldEffect, Dnc) => 5,
            (FinishingMoveCount, Dnc) => 4,
            (SongCastTime, Brd) => 5,
            (SongEffectDuration, Brd) => 5,

            // ========= 新規 enum: ペット系 =========
            (PetPhysicalAtkDef, Bst) => 106,
            (PetPhysicalAccEva, Bst) => 70,
            (PetStatus, Bst) => 8,
            (PetTpBonus, Bst) => 40,
            (AvatarPhysicalAtkDef, Smn) => 106,
            (AvatarPhysicalAccEva, Smn) => 70,
            (AvatarMagicalAtkDef, Smn) => 36,
            (AvatarMagicalAccEva, Smn) => 70,
            (AvatarBlessingEffect, Smn) => 1,
            (AutomatonPhysicalAtkDef, Pup) => 106,
            (AutomatonPhysicalAccEva, Pup) => 70,
            (AutomatonMagicalAtkDef, Pup) => 36,
            (AutomatonMagicalAccEva, Pup) => 70,
            (AutomatonElementBoost, Pup) => 4,
            (WyvernBoostEffect, Drg) => 2,
            (WyvernPhysicalAccEva, Drg) => 70,
            (WyvernMagicalAccEva, Drg) => 70,
            (BreathRecast, Drg) => 10,

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
