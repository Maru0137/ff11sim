use crate::data_loader::JOB_STATUS_GRADES;
use crate::status::{Grade, StatusKind};
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, VariantArray};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumCount,
    EnumIter,
    VariantArray,
    Enum,
    Serialize,
    Deserialize,
)]
pub enum Job {
    War,
    Mnk,
    Whm,
    Blm,
    Rdm,
    Thf,
    Pld,
    Drk,
    Bst,
    Brd,
    Rng,
    Sam,
    Nin,
    Drg,
    Smn,
    Blu,
    Cor,
    Pup,
    Dnc,
    Sch,
    Geo,
    Run,
}

impl Job {
    pub fn status_grade(&self, kind: StatusKind) -> Option<Grade> {
        JOB_STATUS_GRADES[*self][kind]
    }
}

// ---------------------------------------------------------------------------
// Job Traits (ジョブ特性)
//
// データソース: https://wiki.ffo.jp/html/450.html
//
// 実装方針:
//   - 既存 12 特性 (AttackBonus 〜 SkillchainBonus) は実値を保持
//   - 新規 74 特性は wiki から trait_levels (習得レベル) のみ取り込み、
//     trait_cumulative はプレースホルダー (PLACEHOLDER_TRAIT = &[0])。
//     効果値 (%, 段階値) は別タスクで個別に実装する。
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobTrait {
    // wiki ジョブ特性一覧 (https://wiki.ffo.jp/html/450.html) の表示順に合わせる
    AttackBonus,
    DefenseBonus,
    AccuracyBonus,
    EvasionBonus,
    MagicAttackBonus,
    MagicDefenseBonus,
    MagicAccuracyBonus,
    MagicEvasionBonus,
    MaxHpBoost,
    MaxHpBoost2,
    MaxMpBoost,
    MaxDamageBoost,
    AutoRegen,
    AutoRefresh,
    DoubleAttack,
    TripleAttack,
    MartialArts,
    Counter,
    SubtleBlow,
    KickAttacks,
    Daken,
    DualWield,
    Zanshin,
    StoreTp,
    RapidShot,
    ClearMind,
    ConserveMp,
    FastCast,
    UndeadKiller,
    ArcanaKiller,
    DemonKiller,
    DragonKiller,
    VerminKiller,
    BirdKiller,
    AmorphKiller,
    LizardKiller,
    AquanKiller,
    PlantoidKiller,
    BeastKiller,
    ResistVirus,
    ResistPetrify,
    ResistGravity,
    ResistSleep,
    ResistParalyze,
    ResistSlow,
    ResistSilence,
    ResistPoison,
    ResistBlind,
    ResistBind,
    ResistAmnesia,
    Alertness,
    Stealth,
    Gilfinder,
    TreasureHunter,
    Assassin,
    DivineVeil,
    ShieldMastery,
    ShieldBarrier,
    Smite,
    CritIncrease,
    CritReduce,
    DeadAim,
    TandemHit,
    TandemSubtleBlow,
    TacticalParry,
    TacticalGuard,
    ExtremeGuard,
    StoutServant,
    Recycle,
    TrueShot,
    BloodBoon,
    SkillchainBonus,
    WeaponSkillDamage,
    Fencer,
    ConserveTp,
    Strafe,
    MagicAcumen,
    MagicBurstBonus,
    DivineBenison,
    ElementalCelerity,
    TranquilHeart,
    DesperateBlows,
    StalwartSoul,
    CardinalChant,
    Tenacity,
    Inquartata,
}

// ---------------------------------------------------------------------------
// Cumulative bonus values (rank N → cumulative effect at that rank)
// Index 0 = rank 1, index 1 = rank 2, ...
// ---------------------------------------------------------------------------

const ATTACK_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72, 84, 96];
const DEFENSE_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72];
const MAGIC_DEFENSE_BONUS: &[i32] = &[10, 12, 14, 16, 18, 20, 22];
const MAX_HP_BOOST: &[i32] = &[30, 60, 120, 180, 240, 280];
const MAX_HP_BOOST2: &[i32] = &[150, 300, 450];
const MAX_MP_BOOST: &[i32] = &[10, 20, 40, 60, 80, 100];
const EVASION_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72];
const ACCURACY_BONUS: &[i32] = &[10, 22, 35, 48, 60, 72];
const MAGIC_ATTACK_BONUS: &[i32] = &[20, 24, 28, 32, 36, 40];
// Store TP I-V: SAM Lv10/30/50/70/90, cumulative +10/+15/+20/+25/+30
const STORE_TP: &[i32] = &[10, 15, 20, 25, 30];
// Double Attack: WAR Lv25/50/75/85/99, cumulative +10/+12/+14/+16/+18 (%)
const DOUBLE_ATTACK: &[i32] = &[10, 12, 14, 16, 18];
// Skillchain Bonus: 全ジョブ共通 rank1=8%, rank2=12%, rank3=16%, rank4=20%, rank5=23%
// (https://wiki.ffo.jp/html/20337.html 参照)
const SKILLCHAIN_BONUS: &[i32] = &[8, 12, 16, 20, 23];

// 魔法命中率アップ (https://wiki.ffo.jp/html/28672.html)
// BLU Lv99 で rank 1 (+10)、ギフト「ジョブ特性効果アップ」(100JP) で rank 2 (累積 +22)。
// rank 3 は wiki に記載なし → 累積配列を 2 要素に止めて clamp で rank 2 値にフォールバック。
const MAGIC_ACCURACY_BONUS: &[i32] = &[10, 22];

// 魔法回避率アップ (https://wiki.ffo.jp/html/28673.html)
// 同上。BLU Lv99 = rank 1 (+10)、ギフト 100 JP で rank 2 (累積 +22)。
const MAGIC_EVASION_BONUS: &[i32] = &[10, 22];

// オートリジェネ (https://wiki.ffo.jp/html/2675.html)
// 3 秒ごとに HP を回復。rank 1 = +1, rank 2 = +2, rank 3 = +3
const AUTO_REGEN: &[i32] = &[1, 2, 3];

// オートリフレシュ (https://wiki.ffo.jp/html/2723.html)
// 3 秒ごとに MP を回復。rank 1 = +1, rank 2 = +2 (SMN Lv90 のみ rank 2)
// 注: BLU の「ジョブ特性効果アップ」ギフトは AutoRefresh に適用されない (除外特性)
const AUTO_REFRESH: &[i32] = &[1, 2];

// トリプルアタック (https://wiki.ffo.jp/html/1634.html)
// 約 5%/6% の確率で 3 回攻撃。rank 1 = 5%, rank 2 = 6%
// 注: BLU の「ジョブ特性効果アップ」ギフトは TripleAttack に適用されない (除外特性)
// THF の TA ギフト (G125/G450/G1050/G1900 で +8/+10/+12/+14 累積) は別途実装が必要
const TRIPLE_ATTACK: &[i32] = &[5, 6];

// カウンター (https://wiki.ffo.jp/html/567.html)
// 反撃発動率。rank 1 = 8%, rank 2 = 12%
const COUNTER: &[i32] = &[8, 12];

// モクシャ (https://wiki.ffo.jp/html/3066.html)
// 与TP減少。rank 1=5, 2=10, 3=15, 4=20, 5=25, 6=27
const SUBTLE_BLOW: &[i32] = &[5, 10, 15, 20, 25, 27];

// 蹴撃 (https://wiki.ffo.jp/html/3265.html)
// 蹴撃発動率。rank 1=10%, 2=12%, 3=14%
const KICK_ATTACKS: &[i32] = &[10, 12, 14];

// 打剣 (https://wiki.ffo.jp/html/31980.html)
// 手裏剣自動投擲発動率。rank 1=20%, 2=25%, 3=30%, 4=35%, 5=40%
// (G150/G500/G1125/G2000 でさらに上昇するが NIN ジョブポイントギフトの仕組み)
const DAKEN: &[i32] = &[20, 25, 30, 35, 40];

// 二刀流 (https://wiki.ffo.jp/html/449.html)
// 攻撃間隔短縮率 (% reduction)。rank 1=10, 2=15, 3=25, 4=30, 5=35, 6=40
// (rank 6 は BLU 1200JP「ジョブ特性効果アップ」専用)
const DUAL_WIELD: &[i32] = &[10, 15, 25, 30, 35, 40];

// 残心 (https://wiki.ffo.jp/html/3390.html)
// ミス時再攻撃発動率。rank 1=15%, 2=25%, 3=35%, 4=45%, 5=50%
// (G80/G405/G980/G1805 で SAM ジョブポイントギフトによりさらに上昇)
const ZANSHIN: &[i32] = &[15, 25, 35, 45, 50];

// ラピッドショット (https://wiki.ffo.jp/html/2986.html)
// 遠隔攻撃間隔短縮の発動率。rank 1 ≒ 25% と推測
// rank 2/3 の正確な値は wiki に記載なし → 配列を [25] に止めて clamp で同値にフォールバック
const RAPID_SHOT: &[i32] = &[25];

// クリアマインド (https://wiki.ffo.jp/html/1725.html)
// ヒーリング中の MP 回復量。基準値 12/tick に対するボーナス。
// 累積値: rank1=15, 2=18, 3=21, 4=24, 5=27, 6=30 (基準12 + 各 +3/rank)
// → 基準を除いたボーナスとして [3, 6, 9, 12, 15, 18] を保持
const CLEAR_MIND: &[i32] = &[3, 6, 9, 12, 15, 18];

// コンサーブMP (https://wiki.ffo.jp/html/3314.html)
// 消費 MP 軽減発動率。rank1=25%, 2=28%, 3=31%, 4=34%, 5=37%, 6=40%, 7=43%
const CONSERVE_MP: &[i32] = &[25, 28, 31, 34, 37, 40, 43];

// ファストキャスト (https://wiki.ffo.jp/html/1717.html)
// 詠唱時間短縮 (% 表現)。FC 係数 0.05/rank。rank 1=5, ..., rank 6=30
const FAST_CAST: &[i32] = &[5, 10, 15, 20, 25, 30];

// 各種キラー特性 (https://wiki.ffo.jp/html/2673.html 他)
// 全キラー共通: rank1=8%, rank2=10%, rank3=12% (rank 3 は BLU ギフト専用)
const KILLER: &[i32] = &[8, 10, 12];

// 各種レジスト特性 (例: https://wiki.ffo.jp/html/3256.html 他)
// 全レジスト共通: rank1=10%, rank2=15%, rank3=20%, rank4=25%, rank5=30%
const RESIST_DEFAULT: &[i32] = &[10, 15, 20, 25, 30];

// 警戒 (https://wiki.ffo.jp/html/2984.html): RNG Lv5、視覚感知範囲縮小 (バイナリ)
// ステルス (https://wiki.ffo.jp/html/2393.html): NIN Lv5/86、聴覚感知範囲縮小 (2 ランクだが効果は段階的でない)
// ギルスティール (https://wiki.ffo.jp/html/1677.html): THF Lv5/85, BLU Lv90、ギル取得約1.5倍 (バイナリ)
// アサシン (https://wiki.ffo.jp/html/888.html): THF Lv60、だまし討ち強化 (バイナリ)
// 女神の慈悲 (https://wiki.ffo.jp/html/3246.html): WHM Lv50、ナ系魔法/イレース範囲化 (バイナリ)
// シールドバリア (https://wiki.ffo.jp/html/37984.html): PLD Lv70、プロテス強化 (バイナリ)
// トランキルハート (https://wiki.ffo.jp/html/23693.html): WHM/RDM/SCH、回復魔法敵対心軽減 (バイナリ・スキル依存)
// バイナリ系: 習得済みなら値=1、未習得なら 0 を返す
const TRAIT_BINARY: &[i32] = &[1];
// 2 ランクのバイナリ系特性。wiki に具体的効果値の記載がないため、
// rank 番号をそのまま値として扱う (rank 1 → 1, rank 2 → 2)。
const TRAIT_BINARY_2: &[i32] = &[1, 2];

// トレジャーハンター (https://wiki.ffo.jp/html/1678.html): TH レベルそのもの。THF Lv15/45/90 で +1/+2/+3, BLU Lv98 で +1
const TREASURE_HUNTER: &[i32] = &[1, 2, 3];

// シールドマスタリー (https://wiki.ffo.jp/html/3316.html)
// 盾防御発動時の TP ボーナス。rank1=+10, 2=+20, 3=+30, 4=+40
const SHIELD_MASTERY: &[i32] = &[10, 20, 30, 40];

// スマイト (https://wiki.ffo.jp/html/35682.html)
// 攻撃力アップ。rank1=+10%, 2=+15%, 3=+20%, 4=+25%, 5=+30%
const SMITE: &[i32] = &[10, 15, 20, 25, 30];

// C.インクリース (https://wiki.ffo.jp/html/20004.html)
// クリティカルダメージボーナス。rank1=+5%, 2=+8%, 3=+11%, 4=+14%
const CRIT_INCREASE: &[i32] = &[5, 8, 11, 14];

// C.リデュース (https://wiki.ffo.jp/html/20045.html)
// 被クリティカルダメージ軽減。rank1=-5%, 2=-8%, 3=-11%, 4=-14%
// 値は軽減量を正の整数で保持 (=実際の効果は -5%)
const CRIT_REDUCE: &[i32] = &[5, 8, 11, 14];

// デッドエイム (https://wiki.ffo.jp/html/23695.html)
// 遠隔クリティカルダメージボーナス。rank1=+10%, 2=+20%, 3=+30%, 4=+35%, 5=+40%, 6=+45%
const DEAD_AIM: &[i32] = &[10, 20, 30, 35, 40, 45];

// タンデムヒット (https://wiki.ffo.jp/html/38004.html)
// ペット連携時の命中/魔命ボーナス。rank5=+50 のみ wiki 記載、ranks 1-4 は線形補間と仮定
const TANDEM_HIT: &[i32] = &[10, 20, 30, 40, 50];

// タンデムモクシャ (https://wiki.ffo.jp/html/38005.html)
// ペット連携時の与TP減少。rank3=モクシャII+14 ベース、rank 1/2 は推定
const TANDEM_SUBTLE_BLOW: &[i32] = &[5, 10, 14];

// タクティカルパリー (https://wiki.ffo.jp/html/20336.html)
// 受け流し発動時の TP ボーナス。rank1=+20, 2=+30, 3=+40, 4=+50
const TACTICAL_PARRY: &[i32] = &[20, 30, 40, 50];

// タクティカルガード (https://wiki.ffo.jp/html/20277.html)
// ガード発動時の TP ボーナス。rank1=+30, 2=+45, 3=+60
const TACTICAL_GUARD: &[i32] = &[30, 45, 60];

// エクストリームガード (https://wiki.ffo.jp/html/20048.html)
// 盾防御時の固定値ダメージ軽減。rank1=2, 2=4, 3=6, 4=8
const EXTREME_GUARD: &[i32] = &[2, 4, 6, 8];

// スタウトサーヴァント (https://wiki.ffo.jp/html/19979.html)
// ペット被ダメージ軽減 (%)。rank1=5, 2=7, 3=9
const STOUT_SERVANT: &[i32] = &[5, 7, 9];

// リサイクル (https://wiki.ffo.jp/html/8018.html)
// 矢弾消費せず遠隔攻撃の発動率。rank1=+10%, 2=+20%, 3=+30%, 4=+40%
const RECYCLE: &[i32] = &[10, 20, 30, 40];

// トゥルーショット (https://wiki.ffo.jp/html/20222.html)
// 適正距離での遠隔ダメージボーナス。rank1=+3%, 2=+5%, 3=+7%
const TRUE_SHOT: &[i32] = &[3, 5, 7];

// ブラッドブーン (https://wiki.ffo.jp/html/20335.html)
// 契約の履行 MP 消費軽減発動率。rank1=20%, 2=23%, 3=26%, 4=29%
const BLOOD_BOON: &[i32] = &[20, 23, 26, 29];

// WS ダメージアップ (https://wiki.ffo.jp/html/37765.html)
// rank1=+7%, 2=+10%, 3=+13%, 4=+16%, 5=+19%, 6=+21%
const WEAPON_SKILL_DAMAGE: &[i32] = &[7, 10, 13, 16, 19, 21];

// フェンサー (https://wiki.ffo.jp/html/20275.html)
// クリ発動率ボーナスと WS TP ボーナスの 2 つの効果を持ち、それぞれ rank ごとに
// スケールが異なる。単一の値を保持する形式に合わないため、cumulative には
// rank 番号をそのまま入れて、効果は呼び出し側で rank ベースで計算する。
// 上限は rank 8。
const FENCER: &[i32] = &[1, 2, 3, 4, 5, 6, 7, 8];

// コンサーブTP (https://wiki.ffo.jp/html/18171.html)
// WS 後 TP 残存発動率。rank1=15%, 2=18%, 3=21%, 4=24%, 5=26%
const CONSERVE_TP: &[i32] = &[15, 18, 21, 24, 26];

// ストレイフ (https://wiki.ffo.jp/html/1902.html)
// 飛竜ブレス命中率アップ。具体的な数値は wiki に記載なし → ランク数のみ保持
const STRAFE: &[i32] = &[1, 2, 3, 4];

// マジックアキュメン (https://wiki.ffo.jp/html/20007.html)
// 精霊/暗黒魔法ダメージ時の TP。rank1=+25, 2=+50, 3=+75, 4=+100, 5=+125
const MAGIC_ACUMEN: &[i32] = &[25, 50, 75, 100, 125];

// MB.ボーナス (https://wiki.ffo.jp/html/20225.html)
// マジックバーストダメージボーナス。rank1=+5%, 2=+7%, 3=+9%, 4=+11%, 5=+13%
const MAGIC_BURST_BONUS: &[i32] = &[5, 7, 9, 11, 13];

// ディバインベニゾン (https://wiki.ffo.jp/html/20046.html)
// ナ系/イレース詠唱時間短縮 (rank1=10%..rank5=50%) と敵対心 -5/-10/-15/-20/-25 の
// 2 つの効果を持ち、それぞれスケールが異なる。単一の値を保持する形式に合わないため、
// cumulative には rank 番号をそのまま入れて、効果は呼び出し側で rank ベースで計算する。
const DIVINE_BENISON: &[i32] = &[1, 2, 3, 4, 5];

// エレメントセレリティ (https://wiki.ffo.jp/html/22228.html)
// 精霊魔法詠唱時間短縮 (% 表現)。rank1=10, 2=15, 3=20, 4=25, 5=30
const ELEMENTAL_CELERITY: &[i32] = &[10, 15, 20, 25, 30];

// デスペレートブロー (https://wiki.ffo.jp/html/3250.html)
// ラストリゾート両手武器攻撃間隔短縮。rank1=+5%, 2=+10%, 3=+15% ヘイスト
const DESPERATE_BLOWS: &[i32] = &[5, 10, 15];

// ストルワートソウル (https://wiki.ffo.jp/html/23694.html)
// 暗黒 HP 消費軽減。rank1=15%軽減, rank2=50%軽減
const STALWART_SOUL: &[i32] = &[15, 50];

// カーディナルチャント (https://wiki.ffo.jp/html/28474.html)
// 方角依存で複数ステータスに効果が乗るためスケールが一律でない。単一の値を
// 保持する形式に合わないため、cumulative には rank 番号をそのまま入れて、
// 効果は呼び出し側で rank ベースで計算する。
const CARDINAL_CHANT: &[i32] = &[1, 2, 3];

// テナシティ (https://wiki.ffo.jp/html/28271.html)
// 状態異常レジスト発動率。rank1=+5%, 2=+7%, 3=+9%, 4=+11%, 5=+13%, 6=+15%
const TENACITY: &[i32] = &[5, 7, 9, 11, 13, 15];

// インクァルタタ (https://wiki.ffo.jp/html/28305.html)
// 受け流し率ボーナス。rank1=+5%, 2=+7%, 3=+9%, 4=+11%, 5=+13%, 6=+15%, 7=+17%, 8=+19%
const INQUARTATA: &[i32] = &[5, 7, 9, 11, 13, 15, 17, 19];

// マーシャルアーツ (https://wiki.ffo.jp/html/3240.html)
// 格闘武器の攻撃間隔短縮。値は負 (隔短縮量)。
// rank 1=-80, rank 2=-100, rank 3=-120, rank 4=-140, rank 5=-160, rank 6=-180, rank 7=-200
// 注: 攻撃間隔計算はまだ未実装のため、現時点では値の保持のみ
const MARTIAL_ARTS: &[i32] = &[-80, -100, -120, -140, -160, -180, -200];

// ダメージ上限アップ (https://wiki.ffo.jp/html/37531.html)
// 「攻防関数上限」を rank ごとに +0.1 引き上げる特性。
// 整数表現として「0.1 単位」で扱い、rank 1 = 10 (= +1.0 を 10 倍したスケール) ではなく
// 0.1 単位そのまま 1, 2, 3, 4, 5 として扱う方が直感的だが、
// 既存 i32 シグネチャに合わせて「100 倍したパーセント」相当 (10 = +0.1) で保存する。
// (※ 攻防関数の上限はまだダメージ計算に組み込まれていないため、現時点では値の保持のみ)
const MAX_DAMAGE_BOOST: &[i32] = &[10, 20, 30, 40, 50];

/// プレースホルダー: 新規スケルトン特性用 (効果値未調査)。
/// 8 ランクまでの (job, trait) に対応するため十分な長さを確保。
/// 個別の特性実装時にこの参照を専用定数に差し替える。
const PLACEHOLDER_TRAIT: &[i32] = &[0; 10];

impl JobTrait {
    /// 累積効果値テーブル (rank=1 → cumulative[0], rank=2 → cumulative[1], ...)。
    /// バイナリ特性は `&[1]` (1 ランク) または `&[1, 1]` (2 ランク扱い)。
    fn cumulative(&self) -> &'static [i32] {
        // wiki ジョブ特性一覧 (https://wiki.ffo.jp/html/450.html) の表示順
        match self {
            JobTrait::AttackBonus => ATTACK_BONUS,
            JobTrait::DefenseBonus => DEFENSE_BONUS,
            JobTrait::AccuracyBonus => ACCURACY_BONUS,
            JobTrait::EvasionBonus => EVASION_BONUS,
            JobTrait::MagicAttackBonus => MAGIC_ATTACK_BONUS,
            JobTrait::MagicDefenseBonus => MAGIC_DEFENSE_BONUS,
            JobTrait::MagicAccuracyBonus => MAGIC_ACCURACY_BONUS,
            JobTrait::MagicEvasionBonus => MAGIC_EVASION_BONUS,
            JobTrait::MaxHpBoost => MAX_HP_BOOST,
            JobTrait::MaxHpBoost2 => MAX_HP_BOOST2,
            JobTrait::MaxMpBoost => MAX_MP_BOOST,
            JobTrait::MaxDamageBoost => MAX_DAMAGE_BOOST,
            JobTrait::AutoRegen => AUTO_REGEN,
            JobTrait::AutoRefresh => AUTO_REFRESH,
            JobTrait::DoubleAttack => DOUBLE_ATTACK,
            JobTrait::TripleAttack => TRIPLE_ATTACK,
            JobTrait::MartialArts => MARTIAL_ARTS,
            JobTrait::Counter => COUNTER,
            JobTrait::SubtleBlow => SUBTLE_BLOW,
            JobTrait::KickAttacks => KICK_ATTACKS,
            JobTrait::Daken => DAKEN,
            JobTrait::DualWield => DUAL_WIELD,
            JobTrait::Zanshin => ZANSHIN,
            JobTrait::StoreTp => STORE_TP,
            JobTrait::RapidShot => RAPID_SHOT,
            JobTrait::ClearMind => CLEAR_MIND,
            JobTrait::ConserveMp => CONSERVE_MP,
            JobTrait::FastCast => FAST_CAST,
            // キラー系は全て共通の累積値
            JobTrait::UndeadKiller
            | JobTrait::ArcanaKiller
            | JobTrait::DemonKiller
            | JobTrait::DragonKiller
            | JobTrait::VerminKiller
            | JobTrait::BirdKiller
            | JobTrait::AmorphKiller
            | JobTrait::LizardKiller
            | JobTrait::AquanKiller
            | JobTrait::PlantoidKiller
            | JobTrait::BeastKiller => KILLER,
            // レジスト系は全て共通の累積値
            JobTrait::ResistVirus
            | JobTrait::ResistPetrify
            | JobTrait::ResistGravity
            | JobTrait::ResistSleep
            | JobTrait::ResistParalyze
            | JobTrait::ResistSlow
            | JobTrait::ResistSilence
            | JobTrait::ResistPoison
            | JobTrait::ResistBlind
            | JobTrait::ResistBind
            | JobTrait::ResistAmnesia => RESIST_DEFAULT,
            JobTrait::Alertness => TRAIT_BINARY,
            JobTrait::Stealth => TRAIT_BINARY_2,
            JobTrait::Gilfinder => TRAIT_BINARY_2,
            JobTrait::TreasureHunter => TREASURE_HUNTER,
            JobTrait::Assassin => TRAIT_BINARY,
            JobTrait::DivineVeil => TRAIT_BINARY,
            JobTrait::ShieldMastery => SHIELD_MASTERY,
            JobTrait::ShieldBarrier => TRAIT_BINARY,
            JobTrait::Smite => SMITE,
            JobTrait::CritIncrease => CRIT_INCREASE,
            JobTrait::CritReduce => CRIT_REDUCE,
            JobTrait::DeadAim => DEAD_AIM,
            JobTrait::TandemHit => TANDEM_HIT,
            JobTrait::TandemSubtleBlow => TANDEM_SUBTLE_BLOW,
            JobTrait::TacticalParry => TACTICAL_PARRY,
            JobTrait::TacticalGuard => TACTICAL_GUARD,
            JobTrait::ExtremeGuard => EXTREME_GUARD,
            JobTrait::StoutServant => STOUT_SERVANT,
            JobTrait::Recycle => RECYCLE,
            JobTrait::TrueShot => TRUE_SHOT,
            JobTrait::BloodBoon => BLOOD_BOON,
            JobTrait::SkillchainBonus => SKILLCHAIN_BONUS,
            JobTrait::WeaponSkillDamage => WEAPON_SKILL_DAMAGE,
            JobTrait::Fencer => FENCER,
            JobTrait::ConserveTp => CONSERVE_TP,
            JobTrait::Strafe => STRAFE,
            JobTrait::MagicAcumen => MAGIC_ACUMEN,
            JobTrait::MagicBurstBonus => MAGIC_BURST_BONUS,
            JobTrait::DivineBenison => DIVINE_BENISON,
            JobTrait::ElementalCelerity => ELEMENTAL_CELERITY,
            JobTrait::TranquilHeart => TRAIT_BINARY,
            JobTrait::DesperateBlows => DESPERATE_BLOWS,
            JobTrait::StalwartSoul => STALWART_SOUL,
            JobTrait::CardinalChant => CARDINAL_CHANT,
            JobTrait::Tenacity => TENACITY,
            JobTrait::Inquartata => INQUARTATA,
        }
    }
}

// ---------------------------------------------------------------------------
// Acquisition levels per (trait, job).
// 空スライスはそのジョブが習得しないことを意味する。
// データソース: https://wiki.ffo.jp/html/450.html
// ---------------------------------------------------------------------------

impl Job {
    /// 指定されたジョブ特性をこのジョブが習得するレベル一覧。
    /// rank 1 → `levels[0]`、rank 2 → `levels[1]`、…の習得レベル。
    /// 空スライスは未習得を意味する。
    fn trait_levels(&self, trait_kind: JobTrait) -> &'static [i32] {
        let job = *self;
        match (trait_kind, job) {
            // ============ 物理攻撃力アップ (AttackBonus) ============
            (JobTrait::AttackBonus, Job::War) => &[30, 65, 91],
            (JobTrait::AttackBonus, Job::Drk) => &[10, 30, 50, 70, 76, 83, 91, 99],
            (JobTrait::AttackBonus, Job::Drg) => &[10, 91],

            // ============ 物理防御力アップ (DefenseBonus) ============
            (JobTrait::DefenseBonus, Job::War) => &[10, 45, 86],
            (JobTrait::DefenseBonus, Job::Pld) => &[10, 30, 50, 70, 76, 91],

            // ============ 魔法防御力アップ (MagicDefenseBonus) ============
            (JobTrait::MagicDefenseBonus, Job::Whm) => &[10, 30, 50, 70, 81, 91],
            (JobTrait::MagicDefenseBonus, Job::Rdm) => &[25, 45, 96],
            (JobTrait::MagicDefenseBonus, Job::Run) => &[10, 30, 50, 70, 76, 91, 99],

            // ============ 物理命中率アップ (AccuracyBonus) ============
            (JobTrait::AccuracyBonus, Job::Rng) => &[10, 30, 50, 70, 86, 96],
            (JobTrait::AccuracyBonus, Job::Drg) => &[30, 60, 76],
            (JobTrait::AccuracyBonus, Job::Dnc) => &[30, 60, 76],
            (JobTrait::AccuracyBonus, Job::Run) => &[50, 70, 90],

            // ============ 物理回避率アップ (EvasionBonus) ============
            (JobTrait::EvasionBonus, Job::Thf) => &[10, 30, 50, 70, 76, 88],
            (JobTrait::EvasionBonus, Job::Pup) => &[20, 40, 60, 76],
            (JobTrait::EvasionBonus, Job::Dnc) => &[15, 45, 75, 86],

            // ============ 魔法攻撃力アップ (MagicAttackBonus) ============
            (JobTrait::MagicAttackBonus, Job::Blm) => &[10, 30, 50, 70, 81, 91],
            (JobTrait::MagicAttackBonus, Job::Rdm) => &[20, 40, 86],

            // ============ 魔法命中率アップ (MagicAccuracyBonus) ============

            // ============ 魔法回避率アップ (MagicEvasionBonus) ============

            // ============ HPmaxアップ (MaxHpBoost) ============
            (JobTrait::MaxHpBoost, Job::War) => &[30, 50, 70, 90],
            (JobTrait::MaxHpBoost, Job::Mnk) => &[15, 25, 35, 45, 55, 65],
            (JobTrait::MaxHpBoost, Job::Pld) => &[45, 85],
            (JobTrait::MaxHpBoost, Job::Nin) => &[20, 40, 60, 80, 99],
            (JobTrait::MaxHpBoost, Job::Run) => &[20, 40, 60, 80, 99],

            // ============ HPmaxアップII (MaxHpBoost2) ============
            (JobTrait::MaxHpBoost2, Job::Mnk) => &[75, 85, 95],

            // ============ MPmaxアップ (MaxMpBoost) ============
            (JobTrait::MaxMpBoost, Job::Smn) => &[10, 30, 50, 70, 76, 96],
            (JobTrait::MaxMpBoost, Job::Sch) => &[30, 88],
            (JobTrait::MaxMpBoost, Job::Geo) => &[30, 60, 90],

            // ============ ダメージ上限アップ (MaxDamageBoost) ============
            (JobTrait::MaxDamageBoost, Job::War) => &[40, 80],
            (JobTrait::MaxDamageBoost, Job::Mnk) => &[30, 60, 90],
            (JobTrait::MaxDamageBoost, Job::Rdm) => &[40],
            (JobTrait::MaxDamageBoost, Job::Thf) => &[50],
            (JobTrait::MaxDamageBoost, Job::Drk) => &[20, 40, 55, 70, 80],
            (JobTrait::MaxDamageBoost, Job::Bst) => &[45, 90],
            (JobTrait::MaxDamageBoost, Job::Rng) => &[30, 60, 90],
            (JobTrait::MaxDamageBoost, Job::Sam) => &[40, 80],
            (JobTrait::MaxDamageBoost, Job::Nin) => &[50],
            (JobTrait::MaxDamageBoost, Job::Drg) => &[30, 60, 90],
            (JobTrait::MaxDamageBoost, Job::Pup) => &[45, 90],
            (JobTrait::MaxDamageBoost, Job::Dnc) => &[45, 90],

            // ============ オートリジェネ (AutoRegen) ============
            (JobTrait::AutoRegen, Job::Whm) => &[25, 76],
            (JobTrait::AutoRegen, Job::Run) => &[35, 65, 95],

            // ============ オートリフレシュ (AutoRefresh) ============
            (JobTrait::AutoRefresh, Job::Pld) => &[35],
            (JobTrait::AutoRefresh, Job::Smn) => &[25, 90],

            // ============ ダブルアタック (DoubleAttack) ============
            (JobTrait::DoubleAttack, Job::War) => &[25, 50, 75, 85, 99],

            // ============ トリプルアタック (TripleAttack) ============
            (JobTrait::TripleAttack, Job::Thf) => &[55, 95],

            // ============ マーシャルアーツ (MartialArts) ============
            (JobTrait::MartialArts, Job::Mnk) => &[1, 16, 31, 46, 61, 75, 82],
            (JobTrait::MartialArts, Job::Pup) => &[25, 50, 75, 86, 97],

            // ============ カウンター (Counter) ============
            (JobTrait::Counter, Job::Mnk) => &[10, 81],

            // ============ モクシャ (SubtleBlow) ============
            (JobTrait::SubtleBlow, Job::Mnk) => &[5, 20, 40, 65, 91],
            (JobTrait::SubtleBlow, Job::Nin) => &[15, 30, 45, 60, 75, 91],
            (JobTrait::SubtleBlow, Job::Dnc) => &[25, 45, 65, 86],

            // ============ 蹴撃 (KickAttacks) ============
            (JobTrait::KickAttacks, Job::Mnk) => &[51, 71, 76],

            // ============ 打剣 (Daken) ============
            (JobTrait::Daken, Job::Nin) => &[25, 40, 55, 70, 95],

            // ============ 二刀流 (DualWield) ============
            (JobTrait::DualWield, Job::Thf) => &[83, 90, 98],
            (JobTrait::DualWield, Job::Nin) => &[10, 25, 45, 65, 83],
            (JobTrait::DualWield, Job::Dnc) => &[20, 40, 60, 80],

            // ============ 残心 (Zanshin) ============
            (JobTrait::Zanshin, Job::Sam) => &[20, 35, 50, 65, 95],

            // ============ ストアTP (StoreTp) ============
            (JobTrait::StoreTp, Job::Sam) => &[10, 30, 50, 70, 90],

            // ============ ラピッドショット (RapidShot) ============
            (JobTrait::RapidShot, Job::Rng) => &[15, 76],
            (JobTrait::RapidShot, Job::Cor) => &[15, 91],

            // ============ クリアマインド (ClearMind) ============
            (JobTrait::ClearMind, Job::Whm) => &[20, 35, 50, 65, 80, 96],
            (JobTrait::ClearMind, Job::Blm) => &[15, 30, 45, 60, 75, 96],
            (JobTrait::ClearMind, Job::Rdm) => &[31, 53, 75, 91],
            (JobTrait::ClearMind, Job::Smn) => &[15, 30, 45, 60, 75, 91],
            (JobTrait::ClearMind, Job::Sch) => &[20, 35, 50, 65, 76, 91],
            (JobTrait::ClearMind, Job::Geo) => &[20, 40, 60, 80, 99],

            // ============ コンサーブMP (ConserveMp) ============
            (JobTrait::ConserveMp, Job::Blm) => &[20, 76, 86],
            (JobTrait::ConserveMp, Job::Sch) => &[25, 96],
            (JobTrait::ConserveMp, Job::Geo) => &[10, 25, 40, 55, 70, 85, 99],

            // ============ ファストキャスト (FastCast) ============
            // RDM は rank 1 を習得しない (Lv15 で直接 rank 2 を習得)。
            // count ベースのランク算出に合わせるため rank 2 の習得 Lv (15) を
            // 重複させて rank 1 をスキップする。
            (JobTrait::FastCast, Job::Rdm) => &[15, 15, 35, 55, 76, 90],

            // ============ 各種キラー特性 ============
            (JobTrait::UndeadKiller, Job::Pld) => &[5, 86],
            (JobTrait::ArcanaKiller, Job::Drk) => &[25, 86],
            (JobTrait::DemonKiller, Job::Sam) => &[40, 86],
            (JobTrait::DragonKiller, Job::Drg) => &[25, 86],
            (JobTrait::VerminKiller, Job::Bst) => &[10, 76],
            (JobTrait::BirdKiller, Job::Bst) => &[20, 79],
            (JobTrait::AmorphKiller, Job::Bst) => &[30, 82],
            (JobTrait::LizardKiller, Job::Bst) => &[40, 85],
            (JobTrait::AquanKiller, Job::Bst) => &[50, 88],
            (JobTrait::PlantoidKiller, Job::Bst) => &[60, 91],
            (JobTrait::BeastKiller, Job::Bst) => &[70, 94],

            // ============ 各種レジスト特性 ============
            (JobTrait::ResistVirus, Job::War) => &[15, 35, 55, 70, 81],
            (JobTrait::ResistPetrify, Job::Rdm) => &[10, 30, 50, 70, 81],
            (JobTrait::ResistGravity, Job::Thf) => &[20, 40, 60, 75, 81],
            (JobTrait::ResistSleep, Job::Pld) => &[20, 40, 60, 75, 81],
            (JobTrait::ResistParalyze, Job::Drk) => &[20, 40, 60, 75, 81],
            (JobTrait::ResistParalyze, Job::Cor) => &[5, 25, 45, 65, 81],
            (JobTrait::ResistSlow, Job::Bst) => &[15, 35, 55, 75, 81],
            (JobTrait::ResistSlow, Job::Smn) => &[20, 40, 60, 81],
            (JobTrait::ResistSlow, Job::Pup) => &[10, 50, 70, 81],
            (JobTrait::ResistSlow, Job::Dnc) => &[20, 55, 81],
            (JobTrait::ResistSilence, Job::Brd) => &[5, 25, 45, 65, 81],
            (JobTrait::ResistSilence, Job::Sch) => &[10, 40, 70, 81],
            (JobTrait::ResistPoison, Job::Rng) => &[20, 40, 60, 81],
            (JobTrait::ResistBlind, Job::Sam) => &[5, 25, 45, 65, 81],
            (JobTrait::ResistBind, Job::Nin) => &[10, 30, 50, 70, 81],
            (JobTrait::ResistAmnesia, Job::Bst) => &[15, 35, 55, 75, 95],
            (JobTrait::ResistAmnesia, Job::Cor) => &[30, 50, 70, 90],
            (JobTrait::ResistAmnesia, Job::Pup) => &[15, 35, 55, 75, 95],

            // ============ 警戒 / ステルス / その他 THF系 ============
            (JobTrait::Alertness, Job::Rng) => &[5],
            (JobTrait::Stealth, Job::Nin) => &[5, 86],
            (JobTrait::Gilfinder, Job::Thf) => &[5, 85],
            // トレジャーハンター I/II/III は wiki 上は別項だが、
            // 仕組み上は同一特性の段階的強化 (THF Lv15→rank1, Lv45→rank2, Lv90→rank3) なので統合。
            (JobTrait::TreasureHunter, Job::Thf) => &[15, 45, 90],
            (JobTrait::Assassin, Job::Thf) => &[60],

            // ============ 女神の慈悲 / シールド系 ============
            (JobTrait::DivineVeil, Job::Whm) => &[50],
            (JobTrait::ShieldMastery, Job::War) => &[80, 87, 94],
            (JobTrait::ShieldMastery, Job::Rdm) => &[87, 97],
            (JobTrait::ShieldMastery, Job::Pld) => &[25, 50, 75, 96],
            (JobTrait::ShieldBarrier, Job::Pld) => &[70],

            // ============ スマイト ============
            (JobTrait::Smite, Job::War) => &[35, 65, 95],
            (JobTrait::Smite, Job::Mnk) => &[40, 80],
            (JobTrait::Smite, Job::Drk) => &[15, 35, 55, 75, 95],
            (JobTrait::Smite, Job::Drg) => &[40, 80],
            (JobTrait::Smite, Job::Pup) => &[60],

            // ============ クリティカル系 ============
            (JobTrait::CritIncrease, Job::War) => &[78, 84],
            (JobTrait::CritIncrease, Job::Thf) => &[78, 84, 91, 97],
            (JobTrait::CritIncrease, Job::Drk) => &[85, 95],
            (JobTrait::CritIncrease, Job::Dnc) => &[80, 88, 99],
            (JobTrait::CritReduce, Job::Pld) => &[79, 85, 91, 96],
            (JobTrait::CritReduce, Job::Brd) => &[80, 91],
            (JobTrait::CritReduce, Job::Drg) => &[85, 95],
            (JobTrait::CritReduce, Job::Pup) => &[85, 95],
            (JobTrait::DeadAim, Job::Rng) => &[50, 60, 70, 80, 90, 99],

            // ============ ペット連携系 (BST) ============
            (JobTrait::TandemHit, Job::Bst) => &[30, 45, 60, 75, 90],
            (JobTrait::TandemSubtleBlow, Job::Bst) => &[40, 60, 80],

            // ============ タクティカル系 (受け流し / ガード時 TP) ============
            (JobTrait::TacticalParry, Job::Drk) => &[88, 98],
            (JobTrait::TacticalParry, Job::Nin) => &[77, 87, 97],
            (JobTrait::TacticalParry, Job::Dnc) => &[77, 84, 91, 98],
            (JobTrait::TacticalParry, Job::Run) => &[40, 60, 85],
            (JobTrait::TacticalGuard, Job::Mnk) => &[77, 87, 97],
            (JobTrait::TacticalGuard, Job::Pup) => &[80, 90],
            (JobTrait::ExtremeGuard, Job::War) => &[80, 88, 99],
            (JobTrait::ExtremeGuard, Job::Whm) => &[85, 95],
            (JobTrait::ExtremeGuard, Job::Pld) => &[77, 82, 88, 93],

            // ============ ペット系 / 遠隔系 ============
            (JobTrait::StoutServant, Job::Bst) => &[78, 88, 98],
            (JobTrait::StoutServant, Job::Smn) => &[85, 95],
            (JobTrait::StoutServant, Job::Pup) => &[78, 88, 98],
            (JobTrait::Recycle, Job::Rng) => &[20, 35, 50, 65],
            (JobTrait::Recycle, Job::Cor) => &[35, 65, 95],
            (JobTrait::TrueShot, Job::Rng) => &[78, 88, 98],
            (JobTrait::TrueShot, Job::Cor) => &[85, 95],
            (JobTrait::BloodBoon, Job::Smn) => &[60, 70, 80, 90],

            // ============ Skillchain Bonus (連携ボーナス) ============
            // (https://wiki.ffo.jp/html/20337.html)
            (JobTrait::SkillchainBonus, Job::Mnk) => &[85, 95],
            (JobTrait::SkillchainBonus, Job::Nin) => &[85, 95],
            (JobTrait::SkillchainBonus, Job::Sam) => &[78, 88, 98],
            (JobTrait::SkillchainBonus, Job::Dnc) => &[45, 58, 71, 84, 97],

            // ============ 派生・特殊 ============
            (JobTrait::WeaponSkillDamage, Job::Drg) => &[45, 55, 65, 75, 85, 95],
            (JobTrait::Fencer, Job::War) => &[45, 58, 71, 84, 97],
            (JobTrait::Fencer, Job::Bst) => &[80],
            (JobTrait::Fencer, Job::Brd) => &[85, 95],
            (JobTrait::ConserveTp, Job::Rng) => &[80, 91],
            (JobTrait::ConserveTp, Job::Drg) => &[45, 58, 71, 84, 97],
            (JobTrait::ConserveTp, Job::Dnc) => &[77, 87, 97],
            (JobTrait::Strafe, Job::Drg) => &[20, 40, 60, 80],
            (JobTrait::MagicAcumen, Job::Blm) => &[85, 95],
            (JobTrait::MagicAcumen, Job::Drk) => &[45, 58, 71, 84, 97],
            (JobTrait::MagicAcumen, Job::Sch) => &[78, 88, 98],
            (JobTrait::MagicBurstBonus, Job::Blm) => &[45, 58, 71, 84, 97],
            (JobTrait::MagicBurstBonus, Job::Rdm) => &[85, 95],
            (JobTrait::MagicBurstBonus, Job::Nin) => &[80, 90],
            (JobTrait::MagicBurstBonus, Job::Sch) => &[79, 89, 99],
            (JobTrait::DivineBenison, Job::Whm) => &[50, 60, 70, 80, 90],
            (JobTrait::ElementalCelerity, Job::Blm) => &[50, 60, 70, 80, 90],
            (JobTrait::ElementalCelerity, Job::Geo) => &[55, 80],
            (JobTrait::TranquilHeart, Job::Whm) => &[21],
            (JobTrait::TranquilHeart, Job::Rdm) => &[26],
            (JobTrait::TranquilHeart, Job::Sch) => &[30],
            (JobTrait::DesperateBlows, Job::Drk) => &[15, 30, 45],
            (JobTrait::StalwartSoul, Job::Drk) => &[45, 90],
            (JobTrait::CardinalChant, Job::Geo) => &[25, 45, 85],
            (JobTrait::Tenacity, Job::Run) => &[5, 25, 45, 75, 80, 95],
            (JobTrait::Inquartata, Job::Run) => &[15, 45, 75, 90],

            _ => &[],
        }
    }

    /// 指定ジョブ特性のこのジョブ・指定 lv 時点での習得済みランク数 (0 = 未習得)。
    pub fn trait_rank_at_lv(&self, trait_kind: JobTrait, lv: i32) -> usize {
        self.trait_levels(trait_kind)
            .iter()
            .filter(|&&req_lv| lv >= req_lv)
            .count()
    }

    /// このジョブ・指定 lv 時点でのジョブ特性ボーナス (ギフト未考慮)。
    pub fn trait_bonus(&self, trait_kind: JobTrait, lv: i32) -> i32 {
        let rank = self.trait_rank_at_lv(trait_kind, lv);
        trait_kind.value_at_rank(rank)
    }
}

impl JobTrait {
    /// 指定 rank に対応する累積効果値 (rank=0 → 0)。
    /// rank が cumulative 配列長を超える場合は配列末尾の値で clamp。
    pub fn value_at_rank(&self, rank: usize) -> i32 {
        if rank == 0 {
            return 0;
        }
        let cumulative = self.cumulative();
        let idx = std::cmp::min(rank, cumulative.len()) - 1;
        cumulative[idx]
    }

    /// BLU の「ジョブ特性効果アップ」ギフト (https://wiki.ffo.jp/html/34014.html) で
    /// ランクアップの効果を受けない特性。
    /// (Gilfinder / DoubleAttack / AutoRefresh / TripleAttack)
    pub fn is_blu_effect_up_excluded(&self) -> bool {
        matches!(
            self,
            JobTrait::Gilfinder
                | JobTrait::DoubleAttack
                | JobTrait::AutoRefresh
                | JobTrait::TripleAttack
        )
    }
}

// BLU の「ジョブ特性効果アップ」ギフトは src/gift.rs の `Gift::JobTraitEffectUp`
// に統合済み。`Job::Blu.gift_value(Gift::JobTraitEffectUp, total_jp)` で取得する。

#[cfg(test)]
mod tests {
    use super::*;

    /// 連携ボーナス (Skillchain Bonus) ジョブ特性の値検証
    /// データソース: https://wiki.ffo.jp/html/20337.html
    /// 累積値: rank1=8, rank2=12, rank3=16, rank4=20, rank5=23
    #[test]
    fn test_skillchain_bonus_trait_sam() {
        // SAM: Lv78/88/98 で rank1/2/3
        assert_eq!(Job::Sam.trait_bonus(JobTrait::SkillchainBonus, 77), 0);
        assert_eq!(Job::Sam.trait_bonus(JobTrait::SkillchainBonus, 78), 8);
        assert_eq!(Job::Sam.trait_bonus(JobTrait::SkillchainBonus, 87), 8);
        assert_eq!(Job::Sam.trait_bonus(JobTrait::SkillchainBonus, 88), 12);
        assert_eq!(Job::Sam.trait_bonus(JobTrait::SkillchainBonus, 97), 12);
        assert_eq!(Job::Sam.trait_bonus(JobTrait::SkillchainBonus, 98), 16);
        assert_eq!(Job::Sam.trait_bonus(JobTrait::SkillchainBonus, 99), 16);
    }

    #[test]
    fn test_skillchain_bonus_trait_mnk_nin() {
        // MNK / NIN: Lv85/95 で rank1/2
        for &job in &[Job::Mnk, Job::Nin] {
            assert_eq!(job.trait_bonus(JobTrait::SkillchainBonus, 84), 0);
            assert_eq!(job.trait_bonus(JobTrait::SkillchainBonus, 85), 8);
            assert_eq!(job.trait_bonus(JobTrait::SkillchainBonus, 95), 12);
            assert_eq!(job.trait_bonus(JobTrait::SkillchainBonus, 99), 12);
        }
    }

    #[test]
    fn test_skillchain_bonus_trait_dnc() {
        // DNC: Lv45/58/71/84/97 で rank1〜5
        assert_eq!(Job::Dnc.trait_bonus(JobTrait::SkillchainBonus, 44), 0);
        assert_eq!(Job::Dnc.trait_bonus(JobTrait::SkillchainBonus, 45), 8);
        assert_eq!(Job::Dnc.trait_bonus(JobTrait::SkillchainBonus, 58), 12);
        assert_eq!(Job::Dnc.trait_bonus(JobTrait::SkillchainBonus, 71), 16);
        assert_eq!(Job::Dnc.trait_bonus(JobTrait::SkillchainBonus, 84), 20);
        assert_eq!(Job::Dnc.trait_bonus(JobTrait::SkillchainBonus, 97), 23);
        assert_eq!(Job::Dnc.trait_bonus(JobTrait::SkillchainBonus, 99), 23);
    }

    #[test]
    fn test_skillchain_bonus_trait_other_jobs() {
        // 連携ボーナスを習得しないジョブは常に 0
        // BLU は青魔法セットによって特性が変わるため Mnk/Nin/Sam/Dnc 以外は 0 (青魔法未対応)
        for &job in &[
            Job::War,
            Job::Whm,
            Job::Blm,
            Job::Rdm,
            Job::Thf,
            Job::Pld,
            Job::Drk,
            Job::Bst,
            Job::Brd,
            Job::Rng,
            Job::Drg,
            Job::Smn,
            Job::Blu,
            Job::Cor,
            Job::Pup,
            Job::Sch,
            Job::Geo,
            Job::Run,
        ] {
            assert_eq!(
                job.trait_bonus(JobTrait::SkillchainBonus, 99),
                0,
                "{:?} should not have Skillchain Bonus trait",
                job,
            );
        }
    }

    // ===========================================================================
    // 全 22 ジョブ × 全ジョブ特性の Lv99 期待値網羅テスト (issue #9)
    //
    // メインジョブ Lv99 のキャラクターを各ジョブで作成し、
    // 全ジョブ特性が期待値と一致することを確認する。
    //
    // 期待値の出典:
    //   - 既存 12 特性: rust/src/job.rs の trait_levels / trait_cumulative 定義
    //   - 新規特性 (skeleton): trait_cumulative=PLACEHOLDER_TRAIT (=0) のため、
    //     習得有無に関わらず Lv99 値は 0。trait_levels が定義されているかは
    //     `test_trait_levels_defined_for_all_pairs` で検証する。
    //
    // 期待値は trait_levels 定義から手動算出 (将来 trait_levels が変わったら
    // ここも追従する必要あり)。
    // ===========================================================================

    /// 全ジョブ特性 (86 個)
    /// wiki ジョブ特性一覧 (https://wiki.ffo.jp/html/450.html) の表示順
    const ALL_TRAITS: &[JobTrait] = &[
        JobTrait::AttackBonus,
        JobTrait::DefenseBonus,
        JobTrait::AccuracyBonus,
        JobTrait::EvasionBonus,
        JobTrait::MagicAttackBonus,
        JobTrait::MagicDefenseBonus,
        JobTrait::MagicAccuracyBonus,
        JobTrait::MagicEvasionBonus,
        JobTrait::MaxHpBoost,
        JobTrait::MaxHpBoost2,
        JobTrait::MaxMpBoost,
        JobTrait::MaxDamageBoost,
        JobTrait::AutoRegen,
        JobTrait::AutoRefresh,
        JobTrait::DoubleAttack,
        JobTrait::TripleAttack,
        JobTrait::MartialArts,
        JobTrait::Counter,
        JobTrait::SubtleBlow,
        JobTrait::KickAttacks,
        JobTrait::Daken,
        JobTrait::DualWield,
        JobTrait::Zanshin,
        JobTrait::StoreTp,
        JobTrait::RapidShot,
        JobTrait::ClearMind,
        JobTrait::ConserveMp,
        JobTrait::FastCast,
        JobTrait::UndeadKiller,
        JobTrait::ArcanaKiller,
        JobTrait::DemonKiller,
        JobTrait::DragonKiller,
        JobTrait::VerminKiller,
        JobTrait::BirdKiller,
        JobTrait::AmorphKiller,
        JobTrait::LizardKiller,
        JobTrait::AquanKiller,
        JobTrait::PlantoidKiller,
        JobTrait::BeastKiller,
        JobTrait::ResistVirus,
        JobTrait::ResistPetrify,
        JobTrait::ResistGravity,
        JobTrait::ResistSleep,
        JobTrait::ResistParalyze,
        JobTrait::ResistSlow,
        JobTrait::ResistSilence,
        JobTrait::ResistPoison,
        JobTrait::ResistBlind,
        JobTrait::ResistBind,
        JobTrait::ResistAmnesia,
        JobTrait::Alertness,
        JobTrait::Stealth,
        JobTrait::Gilfinder,
        JobTrait::TreasureHunter,
        JobTrait::Assassin,
        JobTrait::DivineVeil,
        JobTrait::ShieldMastery,
        JobTrait::ShieldBarrier,
        JobTrait::Smite,
        JobTrait::CritIncrease,
        JobTrait::CritReduce,
        JobTrait::DeadAim,
        JobTrait::TandemHit,
        JobTrait::TandemSubtleBlow,
        JobTrait::TacticalParry,
        JobTrait::TacticalGuard,
        JobTrait::ExtremeGuard,
        JobTrait::StoutServant,
        JobTrait::Recycle,
        JobTrait::TrueShot,
        JobTrait::BloodBoon,
        JobTrait::SkillchainBonus,
        JobTrait::WeaponSkillDamage,
        JobTrait::Fencer,
        JobTrait::ConserveTp,
        JobTrait::Strafe,
        JobTrait::MagicAcumen,
        JobTrait::MagicBurstBonus,
        JobTrait::DivineBenison,
        JobTrait::ElementalCelerity,
        JobTrait::TranquilHeart,
        JobTrait::DesperateBlows,
        JobTrait::StalwartSoul,
        JobTrait::CardinalChant,
        JobTrait::Tenacity,
        JobTrait::Inquartata,
    ];

    /// メインジョブ Lv99 (サブ無し) でのジョブ特性期待値。
    /// 一覧に無い (job, trait) ペアは 0 (該当ジョブが習得しない、
    /// または新規 skeleton 特性で trait_cumulative=PLACEHOLDER_TRAIT(0))。
    fn expected_trait_at_lv99(job: Job, trait_kind: JobTrait) -> i32 {
        use Job::*;
        use JobTrait::*;
        // wiki ジョブ特性一覧 (https://wiki.ffo.jp/html/450.html) の表示順
        match (trait_kind, job) {
            // --- AttackBonus (cumulative [10,22,35,48,60,72,84,96]) ---
            (AttackBonus, War) => 35, // (30,65,91) rank 3
            (AttackBonus, Drk) => 96, // (10,30,50,70,76,83,91,99) rank 8
            (AttackBonus, Drg) => 22, // (10,91) rank 2

            // --- DefenseBonus (cumulative [10,22,35,48,60,72]) ---
            (DefenseBonus, War) => 35, // (10,45,86) rank 3
            (DefenseBonus, Pld) => 72, // (10,30,50,70,76,91) rank 6

            // --- AccuracyBonus (cumulative [10,22,35,48,60,72]) ---
            (AccuracyBonus, Rng) => 72, // (10,30,50,70,86,96) rank 6
            (AccuracyBonus, Drg) => 35, // (30,60,76) rank 3
            (AccuracyBonus, Dnc) => 35, // (30,60,76) rank 3
            (AccuracyBonus, Run) => 35, // (50,70,90) rank 3

            // --- EvasionBonus (cumulative [10,22,35,48,60,72]) ---
            (EvasionBonus, Thf) => 72, // (10,30,50,70,76,88) rank 6
            (EvasionBonus, Pup) => 48, // (20,40,60,76) rank 4
            (EvasionBonus, Dnc) => 48, // (15,45,75,86) rank 4

            // --- MagicAttackBonus (cumulative [20,24,28,32,36,40]) ---
            (MagicAttackBonus, Blm) => 40, // (10,30,50,70,81,91) rank 6
            (MagicAttackBonus, Rdm) => 28, // (20,40,86) rank 3

            // --- MagicDefenseBonus (cumulative [10,12,14,16,18,20,22]) ---
            (MagicDefenseBonus, Whm) => 20, // (10,30,50,70,81,91) rank 6
            (MagicDefenseBonus, Rdm) => 14, // (25,45,96) rank 3
            (MagicDefenseBonus, Run) => 22, // (10,30,50,70,76,91,99) rank 7

            // --- MagicAccuracyBonus (cumulative [10]) ---

            // --- MagicEvasionBonus (cumulative [10]) ---

            // --- MaxHpBoost (cumulative [30,60,120,180,240,280]) ---
            (MaxHpBoost, War) => 180, // (30,50,70,90) rank 4
            (MaxHpBoost, Mnk) => 280, // (15,25,35,45,55,65) rank 6
            (MaxHpBoost, Pld) => 60,  // (45,85) rank 2
            (MaxHpBoost, Nin) => 240, // (20,40,60,80,99) rank 5
            (MaxHpBoost, Run) => 240, // (20,40,60,80,99) rank 5

            // --- MaxHpBoost2 (cumulative [150,300,450]) ---
            (MaxHpBoost2, Mnk) => 450, // (75,85,95) rank 3

            // --- MaxMpBoost (cumulative [10,20,40,60,80,100]) ---
            (MaxMpBoost, Smn) => 100, // (10,30,50,70,76,96) rank 6
            (MaxMpBoost, Sch) => 20,  // (30,88) rank 2
            (MaxMpBoost, Geo) => 40,  // (30,60,90) rank 3

            // --- MaxDamageBoost (cumulative [10,20,30,40,50]) ---
            // 値は「0.1 × 100 倍」表現 (10 = +0.1 攻防関数上限)
            (MaxDamageBoost, War) => 20, // (40,80) rank 2
            (MaxDamageBoost, Mnk) => 30, // (30,60,90) rank 3
            (MaxDamageBoost, Rdm) => 10, // (40) rank 1
            (MaxDamageBoost, Thf) => 10, // (50) rank 1
            (MaxDamageBoost, Drk) => 50, // (20,40,55,70,80) rank 5
            (MaxDamageBoost, Bst) => 20, // (45,90) rank 2
            (MaxDamageBoost, Rng) => 30, // (30,60,90) rank 3
            (MaxDamageBoost, Sam) => 20, // (40,80) rank 2
            (MaxDamageBoost, Nin) => 10, // (50) rank 1
            (MaxDamageBoost, Drg) => 30, // (30,60,90) rank 3
            (MaxDamageBoost, Pup) => 20, // (45,90) rank 2
            (MaxDamageBoost, Dnc) => 20, // (45,90) rank 2

            // --- AutoRegen (cumulative [1, 2, 3]) ---
            (AutoRegen, Whm) => 2, // (25,76) rank 2
            (AutoRegen, Run) => 3, // (35,65,95) rank 3

            // --- AutoRefresh (cumulative [1, 2]) ---
            (AutoRefresh, Pld) => 1, // (35) rank 1
            (AutoRefresh, Smn) => 2, // (25,90) rank 2

            // --- DoubleAttack (cumulative [10,12,14,16,18]) ---
            (DoubleAttack, War) => 18, // (25,50,75,85,99) rank 5

            // --- TripleAttack (cumulative [5, 6]) ---
            (TripleAttack, Thf) => 6, // (55,95) rank 2

            // --- MartialArts (cumulative [-80,-100,-120,-140,-160,-180,-200]) ---
            (MartialArts, Mnk) => -200, // (1,16,31,46,61,75,82) rank 7
            (MartialArts, Pup) => -160, // (25,50,75,86,97) rank 5

            // --- Counter (cumulative [8, 12]) ---
            (Counter, Mnk) => 12, // (10,81) rank 2

            // --- SubtleBlow (cumulative [5, 10, 15, 20, 25, 27]) ---
            (SubtleBlow, Mnk) => 25, // (5,20,40,65,91) rank 5
            (SubtleBlow, Nin) => 27, // (15,30,45,60,75,91) rank 6
            (SubtleBlow, Dnc) => 20, // (25,45,65,86) rank 4

            // --- KickAttacks (cumulative [10, 12, 14]) ---
            (KickAttacks, Mnk) => 14, // (51,71,76) rank 3

            // --- Daken (cumulative [20, 25, 30, 35, 40]) ---
            (Daken, Nin) => 40, // (25,40,55,70,95) rank 5

            // --- DualWield (cumulative [10, 15, 25, 30, 35, 40]) ---
            (DualWield, Thf) => 25, // (83,90,98) rank 3
            (DualWield, Nin) => 35, // (10,25,45,65,83) rank 5
            (DualWield, Dnc) => 30, // (20,40,60,80) rank 4

            // --- Zanshin (cumulative [15, 25, 35, 45, 50]) ---
            (Zanshin, Sam) => 50, // (20,35,50,65,95) rank 5

            // --- StoreTp (cumulative [10,15,20,25,30]) ---
            (StoreTp, Sam) => 30, // (10,30,50,70,90) rank 5

            // --- RapidShot (cumulative [25] のみ確定、rank 2/3 wiki記載なし → clamp で 25) ---
            (RapidShot, Rng) => 25, // (15,76) rank 2 → cumulative[1]=cumulative.last()=25
            (RapidShot, Cor) => 25, // (15,91) rank 2 → 同上

            // --- ClearMind (cumulative [3, 6, 9, 12, 15, 18]) ---
            (ClearMind, Whm) => 18, // (20,35,50,65,80,96) rank 6
            (ClearMind, Blm) => 18, // (15,30,45,60,75,96) rank 6
            (ClearMind, Rdm) => 12, // (31,53,75,91) rank 4
            (ClearMind, Smn) => 18, // (15,30,45,60,75,91) rank 6
            (ClearMind, Sch) => 18, // (20,35,50,65,76,91) rank 6
            (ClearMind, Geo) => 15, // (20,40,60,80,99) rank 5

            // --- ConserveMP (cumulative [25, 28, 31, 34, 37, 40, 43]) ---
            (ConserveMp, Blm) => 31, // (20,76,86) rank 3
            (ConserveMp, Sch) => 28, // (25,96) rank 2
            (ConserveMp, Geo) => 43, // (10,25,40,55,70,85,99) rank 7

            // --- FastCast (cumulative [5, 10, 15, 20, 25, 30]) ---
            (FastCast, Rdm) => 30, // (15(skip rank 1),15,35,55,76,90) rank 6

            // --- Killer 系 (全て cumulative [8, 10, 12]) ---
            (UndeadKiller, Pld) => 10,   // (5,86) rank 2
            (ArcanaKiller, Drk) => 10,   // (25,86) rank 2
            (DemonKiller, Sam) => 10,    // (40,86) rank 2
            (DragonKiller, Drg) => 10,   // (25,86) rank 2
            (VerminKiller, Bst) => 10,   // (10,76) rank 2
            (BirdKiller, Bst) => 10,     // (20,79) rank 2
            (AmorphKiller, Bst) => 10,   // (30,82) rank 2
            (LizardKiller, Bst) => 10,   // (40,85) rank 2
            (AquanKiller, Bst) => 10,    // (50,88) rank 2
            (PlantoidKiller, Bst) => 10, // (60,91) rank 2
            (BeastKiller, Bst) => 10,    // (70,94) rank 2

            // --- Resist 系 (cumulative [10, 15, 20, 25, 30]) ---
            (ResistVirus, War) => 30,    // (15,35,55,70,81) rank 5
            (ResistPetrify, Rdm) => 30,  // (10,30,50,70,81) rank 5
            (ResistGravity, Thf) => 30,  // (20,40,60,75,81) rank 5
            (ResistSleep, Pld) => 30,    // (20,40,60,75,81) rank 5
            (ResistParalyze, Drk) => 30, // (20,40,60,75,81) rank 5
            (ResistParalyze, Cor) => 30, // (5,25,45,65,81) rank 5
            (ResistSlow, Bst) => 30,     // (15,35,55,75,81) rank 5
            (ResistSlow, Smn) => 25,     // (20,40,60,81) rank 4
            (ResistSlow, Pup) => 25,     // (10,50,70,81) rank 4
            (ResistSlow, Dnc) => 20,     // (20,55,81) rank 3
            (ResistSilence, Brd) => 30,  // (5,25,45,65,81) rank 5
            (ResistSilence, Sch) => 25,  // (10,40,70,81) rank 4
            (ResistPoison, Rng) => 25,   // (20,40,60,81) rank 4
            (ResistBlind, Sam) => 30,    // (5,25,45,65,81) rank 5
            (ResistBind, Nin) => 30,     // (10,30,50,70,81) rank 5
            (ResistAmnesia, Bst) => 30,  // (15,35,55,75,95) rank 5
            (ResistAmnesia, Cor) => 25,  // (30,50,70,90) rank 4
            (ResistAmnesia, Pup) => 30,  // (15,35,55,75,95) rank 5

            // --- Alertness (cumulative [1]) ---
            (Alertness, Rng) => 1, // (5) rank 1

            // --- Stealth (cumulative [1, 2]) ---
            (Stealth, Nin) => 2, // (5,86) rank 2

            // --- Gilfinder (cumulative [1, 2]) ---
            (Gilfinder, Thf) => 2, // (5,85) rank 2

            // --- TreasureHunter (cumulative [1, 2, 3]) ---
            (TreasureHunter, Thf) => 3, // (15,45,90) rank 3

            // --- Assassin (cumulative [1]) ---
            (Assassin, Thf) => 1,

            // --- DivineVeil (cumulative [1]) ---
            (DivineVeil, Whm) => 1,

            // --- ShieldMastery (cumulative [10, 20, 30, 40]) ---
            (ShieldMastery, War) => 30, // (80,87,94) rank 3
            (ShieldMastery, Rdm) => 20, // (87,97) rank 2
            (ShieldMastery, Pld) => 40, // (25,50,75,96) rank 4

            // --- ShieldBarrier (cumulative [1]) ---
            (ShieldBarrier, Pld) => 1,

            // --- Smite (cumulative [10, 15, 20, 25, 30]) ---
            (Smite, War) => 20, // (35,65,95) rank 3
            (Smite, Mnk) => 15, // (40,80) rank 2
            (Smite, Drk) => 30, // (15,35,55,75,95) rank 5
            (Smite, Drg) => 15, // (40,80) rank 2
            (Smite, Pup) => 10, // (60) rank 1

            // --- CritIncrease (cumulative [5, 8, 11, 14]) ---
            (CritIncrease, War) => 8,  // (78,84) rank 2
            (CritIncrease, Thf) => 14, // (78,84,91,97) rank 4
            (CritIncrease, Drk) => 8,  // (85,95) rank 2
            (CritIncrease, Dnc) => 11, // (80,88,99) rank 3

            // --- CritReduce (cumulative [5, 8, 11, 14]) ---
            (CritReduce, Pld) => 14, // (79,85,91,96) rank 4
            (CritReduce, Brd) => 8,  // (80,91) rank 2
            (CritReduce, Drg) => 8,  // (85,95) rank 2
            (CritReduce, Pup) => 8,  // (85,95) rank 2

            // --- DeadAim (cumulative [10, 20, 30, 35, 40, 45]) ---
            (DeadAim, Rng) => 45, // (50,60,70,80,90,99) rank 6

            // --- TandemHit (cumulative [10, 20, 30, 40, 50]) ---
            (TandemHit, Bst) => 50, // (30,45,60,75,90) rank 5

            // --- TandemSubtleBlow (cumulative [5, 10, 14]) ---
            (TandemSubtleBlow, Bst) => 14, // (40,60,80) rank 3

            // --- TacticalParry (cumulative [20, 30, 40, 50]) ---
            (TacticalParry, Drk) => 30, // (88,98) rank 2
            (TacticalParry, Nin) => 40, // (77,87,97) rank 3
            (TacticalParry, Dnc) => 50, // (77,84,91,98) rank 4
            (TacticalParry, Run) => 40, // (40,60,85) rank 3

            // --- TacticalGuard (cumulative [30, 45, 60]) ---
            (TacticalGuard, Mnk) => 60, // (77,87,97) rank 3
            (TacticalGuard, Pup) => 45, // (80,90) rank 2

            // --- ExtremeGuard (cumulative [2, 4, 6, 8]) ---
            (ExtremeGuard, War) => 6, // (80,88,99) rank 3
            (ExtremeGuard, Whm) => 4, // (85,95) rank 2
            (ExtremeGuard, Pld) => 8, // (77,82,88,93) rank 4

            // --- StoutServant (cumulative [5, 7, 9]) ---
            (StoutServant, Bst) => 9, // (78,88,98) rank 3
            (StoutServant, Smn) => 7, // (85,95) rank 2
            (StoutServant, Pup) => 9, // (78,88,98) rank 3

            // --- Recycle (cumulative [10, 20, 30, 40]) ---
            (Recycle, Rng) => 40, // (20,35,50,65) rank 4
            (Recycle, Cor) => 30, // (35,65,95) rank 3

            // --- TrueShot (cumulative [3, 5, 7]) ---
            (TrueShot, Rng) => 7, // (78,88,98) rank 3
            (TrueShot, Cor) => 5, // (85,95) rank 2

            // --- BloodBoon (cumulative [20, 23, 26, 29]) ---
            (BloodBoon, Smn) => 29, // (60,70,80,90) rank 4

            // --- SkillchainBonus (cumulative [8,12,16,20,23]) ---
            (SkillchainBonus, Mnk) => 12, // (85,95) rank 2
            (SkillchainBonus, Nin) => 12, // (85,95) rank 2
            (SkillchainBonus, Sam) => 16, // (78,88,98) rank 3
            (SkillchainBonus, Dnc) => 23, // (45,58,71,84,97) rank 5

            // --- WeaponSkillDamage (cumulative [7, 10, 13, 16, 19, 21]) ---
            (WeaponSkillDamage, Drg) => 21, // (45,55,65,75,85,95) rank 6

            // --- Fencer (cumulative [1, 2, 3, 4, 5, 6, 7, 8] = rank 番号) ---
            (Fencer, War) => 5, // (45,58,71,84,97) rank 5
            (Fencer, Bst) => 1, // (80) rank 1
            (Fencer, Brd) => 2, // (85,95) rank 2

            // --- ConserveTp (cumulative [15, 18, 21, 24, 26]) ---
            (ConserveTp, Rng) => 18, // (80,91) rank 2
            (ConserveTp, Drg) => 26, // (45,58,71,84,97) rank 5
            (ConserveTp, Dnc) => 21, // (77,87,97) rank 3

            // --- Strafe (cumulative [1, 2, 3, 4] placeholder) ---
            (Strafe, Drg) => 4, // (20,40,60,80) rank 4

            // --- MagicAcumen (cumulative [25, 50, 75, 100, 125]) ---
            (MagicAcumen, Blm) => 50,  // (85,95) rank 2
            (MagicAcumen, Drk) => 125, // (45,58,71,84,97) rank 5
            (MagicAcumen, Sch) => 75,  // (78,88,98) rank 3

            // --- MagicBurstBonus (cumulative [5, 7, 9, 11, 13]) ---
            (MagicBurstBonus, Blm) => 13, // (45,58,71,84,97) rank 5
            (MagicBurstBonus, Rdm) => 7,  // (85,95) rank 2
            (MagicBurstBonus, Nin) => 7,  // (80,90) rank 2
            (MagicBurstBonus, Sch) => 9,  // (79,89,99) rank 3

            // --- DivineBenison (cumulative [1, 2, 3, 4, 5] = rank 番号) ---
            (DivineBenison, Whm) => 5, // (50,60,70,80,90) rank 5

            // --- ElementalCelerity (cumulative [10, 15, 20, 25, 30]) ---
            (ElementalCelerity, Blm) => 30, // (50,60,70,80,90) rank 5
            (ElementalCelerity, Geo) => 15, // (55,80) rank 2

            // --- TranquilHeart (cumulative [1]) ---
            (TranquilHeart, Whm) => 1,
            (TranquilHeart, Rdm) => 1,
            (TranquilHeart, Sch) => 1,

            // --- DesperateBlows (cumulative [5, 10, 15]) ---
            (DesperateBlows, Drk) => 15, // (15,30,45) rank 3

            // --- StalwartSoul (cumulative [15, 50]) ---
            (StalwartSoul, Drk) => 50, // (45,90) rank 2

            // --- CardinalChant (cumulative [1, 2, 3] = rank 番号) ---
            (CardinalChant, Geo) => 3, // (25,45,85) rank 3

            // --- Tenacity (cumulative [5, 7, 9, 11, 13, 15]) ---
            (Tenacity, Run) => 15, // (5,25,45,75,80,95) rank 6

            // --- Inquartata (cumulative [5, 7, 9, 11, 13, 15, 17, 19]) ---
            (Inquartata, Run) => 11, // (15,45,75,90) rank 4

            _ => 0,
        }
    }

    /// 全 22 ジョブを Lv99 メインで作成し、全ジョブ特性の値を一括検証する。
    /// Chara::builder 経由で「キャラクターを作った時に」プロパティが期待通りか確認。
    #[test]
    fn test_all_jobs_lv99_traits() {
        use crate::chara::Chara;
        use crate::race::Race;
        use strum::IntoEnumIterator;

        for job in Job::iter() {
            let chara = Chara::builder()
                .race(Race::Hum)
                .main_job(job, 99)
                .master_lv(0)
                .build()
                .unwrap_or_else(|e| panic!("Failed to build {:?}: {}", job, e));

            for &t in ALL_TRAITS {
                let actual = chara.job_trait_total(t);
                let expected = expected_trait_at_lv99(job, t);
                assert_eq!(
                    actual, expected,
                    "{:?} Lv99 / {:?}: expected {}, got {}",
                    job, t, expected, actual
                );
            }
        }
    }

    /// 22 ジョブが全て Job::iter() で網羅されていることを担保
    /// (新ジョブ追加時にこのテストが落ちて、網羅テストの追従漏れに気付ける)
    #[test]
    fn test_all_jobs_count_is_22() {
        use strum::IntoEnumIterator;
        assert_eq!(Job::iter().count(), 22, "FFXI のジョブ数は 22");
    }

    /// 構造テスト: 全 (job, trait) ペアに対して trait_levels / trait_cumulative が
    /// パニックせずに値を返すことを確認する。
    /// 新規 skeleton 特性は効果値が 0 のため値ベースのテストでは検証されないが、
    /// 少なくとも「テーブル定義漏れによる panic」は防げる。
    #[test]
    fn test_trait_levels_defined_for_all_pairs() {
        use strum::IntoEnumIterator;
        for job in Job::iter() {
            for &t in ALL_TRAITS {
                let _ = job.trait_levels(t);
                let _ = t.cumulative();
                // Lv1〜99 で trait_bonus が正常終了
                for lv in [1, 50, 75, 99] {
                    let _ = job.trait_bonus(t, lv);
                }
            }
        }
    }

    /// ALL_TRAITS が 86 個 (既存 12 + 新規 74) であることを担保する。
    /// JobTrait に追加忘れ / 削除忘れがあれば落ちる。
    #[test]
    fn test_all_traits_count() {
        assert_eq!(ALL_TRAITS.len(), 86, "JobTrait は 86 個 (既存12 + 新規74)");
    }
}
