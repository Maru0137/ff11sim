//! 装備の説明文から数値を抽出する。説明文の解釈はこのモジュールが持つ (docs/adr/0018)。
//!
//! 抽出は目的の違う 2 系統がある。
//!
//! - `extract_all_stats` / `extract_skill_bonuses`: **英語**説明文から固定 26 種を
//!   全一致合算する。ステータス計算用。`web/js/equip-stats.js` からの移植 (docs/adr/0010)
//! - `extract_stat_from_description`: **日本語**テキストから呼び出し側が指定した 1 種を
//!   最初の一致だけ取り出す。検索の説明文ステータスソートと、プロパティセットの
//!   ユーザー定義項目 (docs/adr/0015) 用
//!
//! 2 系統は言語も抽出方式も条件セグメントの扱いも異なる。同じ装備で違う値が出うる。
//! 統一は docs/adr/0018 のフォローアップとして別途扱う。
//!
//! # 移植方針: 挙動を変えない (英語側)
//!
//! JS 実装の挙動をそのまま再現する。既知の誤りも含めて移す。具体的には、
//! 説明文に含まれる条件付きセグメント (`In Dynamis:` / `Unity Ranking:` /
//! `Right ear:` / `Set:` など 30 種以上) を JS は体系的に扱っておらず、
//! Unity Ranking は無条件に加算され、`In Dynamis:` は無視されない。
//! これを直すのは条件セグメント対応として別途扱う。移植時に挙動を変えると、
//! JS 実装との全件突き合わせが「移植が正しいか」の判定に使えなくなるため。
//! 突き合わせは `conformance_with_js_over_all_items` テストで行う
//! (移植元だった `web/test/equip-stats-extraction.test.js` は役目を終えて削除済み)。
//!
//! # 正規表現エンジン
//!
//! JS 側は後読み/先読みを 91 箇所で使うため、`regex` crate では移植できない。
//! `fancy-regex` を unicode 機能なしで使う。unicode を切っても日本語リテラルや
//! 文字クラス範囲 (`[ぁ-んァ-ヶ一-龠]`)、全角記号は扱える (実測確認済み)。
//! 失われるのは以下 2 点で、どちらも代替する:
//!
//! - `\s` `\d` `\w` の略記 → 明示的な文字クラスに置き換える
//! - `(?i)` → ヘイスタックを ASCII 小文字化する。ASCII 小文字化は日本語を
//!   変化させないため副作用がない。ただし JS は大小を区別する正規化を先に
//!   行っているので、**正規化 → 小文字化 → 抽出** の順を守る必要がある。

use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, RwLock};

use fancy_regex::Regex;

/// JS の `\s` に相当する文字集合。
/// JS の `\s` は Unicode 空白を含むため、ASCII 空白だけでは不足する。
const WS: &str = r"[ \t\n\r\x0b\x0c\u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]";

/// FFXI の属性アイコン (private use area) → 正規表記。
/// `items.json` の `description_en` は耐性表記をアイコンで持っており、
/// そのままでは正規表現でマッチしない。
const ELEMENT_ICONS: [(char, &str); 8] = [
    ('\u{e000}', "Fire Resistance "),
    ('\u{e001}', "Ice Resistance "),
    ('\u{e002}', "Wind Resistance "),
    ('\u{e003}', "Earth Resistance "),
    ('\u{e004}', "Lightning Resistance "),
    ('\u{e005}', "Water Resistance "),
    ('\u{e006}', "Light Resistance "),
    ('\u{e007}', "Dark Resistance "),
];

/// コンパイル済み正規表現のキャッシュ。
///
/// 抽出は装備 1 件あたり約 80 個のパターンを使うため、呼び出しごとにコンパイルすると
/// 全件処理で 100 万回を超えて実用にならない。パターンは固定の有限集合なので、
/// 初回コンパイル時に `Box::leak` して `'static` にし、以降は使い回す。
static RE_CACHE: LazyLock<RwLock<HashMap<String, &'static Regex>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn re(pattern: &str) -> &'static Regex {
    if let Some(r) = RE_CACHE.read().expect("RE_CACHE poisoned").get(pattern) {
        return r;
    }
    let compiled: &'static Regex = Box::leak(Box::new(
        Regex::new(pattern).unwrap_or_else(|e| panic!("invalid pattern {pattern:?}: {e}")),
    ));
    RE_CACHE
        .write()
        .expect("RE_CACHE poisoned")
        .insert(pattern.to_owned(), compiled);
    compiled
}

// ---------------------------------------------------------------------------
// 前処理 (大小を区別する。JS 側も `/g` で i フラグを付けていない)
// ---------------------------------------------------------------------------

/// 前処理を行う。JS の `extractAllStats` 冒頭と同じ順序で適用する。
/// 順序が重要で、複合形の略記を単純形より先に展開する必要がある。
pub fn normalize(description_en: &str) -> String {
    // リテラルの `\n` を実際の改行にする
    let mut text = description_en.replace("\\n", "\n");

    for (icon, name) in ELEMENT_ICONS {
        if text.contains(icon) {
            text = text.replace(icon, name);
        }
    }

    // "Unity Ranking: Attack+10～15" → " Attack+15" (最大値を採用)
    let unity = format!(
        r"Unity{WS}+Ranking:{WS}*([A-Za-z][0-9A-Za-z_ \t\n\r]*?){WS}*[+-]{WS}*[0-9]+{WS}*[～~]{WS}*([0-9]+)"
    );
    text = re(&unity).replace_all(&text, " $1+$2").into_owned();
    text = re(r"(?:Pet|Avatar|Wyvern|Automaton):[^:]*")
        .replace_all(&text, "")
        .into_owned();

    // 略記の展開。複合形 → 単純形の順を守る。
    for (pat, rep) in [
        (r"Mag\. ?Atk\. Bonus", "Magic Atk. Bonus"),
        (r"M\. ?Def\. ?B\.", "Magic Def. Bonus"),
        (r"Rng\. ?Acc\.", "Ranged Accuracy"),
        (r"Rng\. ?Atk\.", "Ranged Attack"),
        (r"Mag\. ?Acc\.", "Magic Accuracy"),
        (r"Mag\. ?Eva\.", "Magic Evasion"),
        (r"Mag\. ?Dmg\.", "Magic Damage"),
        (r"Mag\. ?Def\.", "Magic Defense"),
    ] {
        text = re(pat).replace_all(&text, rep).into_owned();
    }
    // 単純形は直前が英字/ドットでなく、数値が続く場合のみ
    for (pat, rep) in [
        (r"(?<![A-Za-z.])Acc\.(?=[ \t]*[+-]?[ \t]*[0-9])", "Accuracy"),
        (r"(?<![A-Za-z.])Atk\.(?=[ \t]*[+-]?[ \t]*[0-9])", "Attack"),
        (r"(?<![A-Za-z.])Eva\.(?=[ \t]*[+-]?[ \t]*[0-9])", "Evasion"),
    ] {
        text = re(pat).replace_all(&text, rep).into_owned();
    }

    // "STR/VIT+10" → "STR+10 VIT+10"
    let re_slash = re(r"([A-Z]{2,3}(?:/[A-Z]{2,3})+)[ \t]*([+-][ \t]*[0-9]+%?)");
    text = re_slash
        .replace_all(&text, |caps: &fancy_regex::Captures| {
            let val = &caps[2];
            caps[1]
                .split('/')
                .map(|s| format!("{s}{val}"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .into_owned();

    text
}

// ---------------------------------------------------------------------------
// 抽出結果
// ---------------------------------------------------------------------------

/// 装備 1 件から抽出したステータス。
/// JS は非ゼロのキーだけを持つオブジェクトを返すが、こちらは全項目を持つ構造体とし、
/// 未設定は 0 で表す。キー名は JS 側と一致させる (全件突き合わせのため)。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EquipStats {
    pub hp: i32,
    pub mp: i32,
    pub str_: i32,
    pub dex: i32,
    pub vit: i32,
    pub agi: i32,
    pub int: i32,
    pub mnd: i32,
    pub chr: i32,
    pub hp_pct: i32,
    pub mp_pct: i32,
    pub def: i32,
    pub attack: i32,
    pub accuracy: i32,
    pub evasion: i32,
    pub attack_pct: i32,
    pub ranged_attack: i32,
    pub ranged_accuracy: i32,
    pub magic_attack: i32,
    pub magic_accuracy: i32,
    pub magic_accuracy_skill: i32,
    pub magic_evasion: i32,
    pub magic_damage: i32,
    pub haste_pct: i32,
    pub store_tp: i32,
    pub double_attack_pct: i32,
    pub triple_attack_pct: i32,
    pub quad_attack_pct: i32,
    pub double_attack_damage_pct: i32,
    pub triple_attack_damage_pct: i32,
    pub critical_hit_rate_pct: i32,
    pub critical_hit_damage_pct: i32,
    pub weapon_skill_damage_pct: i32,
    pub subtle_blow: i32,
    pub subtle_blow_2: i32,
    pub tp_bonus: i32,
    pub skillchain_bonus: i32,
    pub physical_damage_limit_pct: i32,
    pub true_shot: i32,
    pub magic_critical_hit_2_pct: i32,
    pub magic_affinity: i32,
    pub magic_burst_damage: i32,
    pub magic_burst_damage_2: i32,
    pub conserve_mp: i32,
    // 再詠唱間隔 (精霊/青は %、歌/忍術は秒。値は装備テキストどおり負数)
    pub elemental_recast_delay_pct: i32,
    pub blue_recast_delay_pct: i32,
    pub song_recast_delay: i32,
    pub ninjutsu_recast_delay: i32,
    pub damage_taken_pct: i32,
    pub physical_damage_taken_pct: i32,
    pub magic_damage_taken_pct: i32,
    pub magic_def_bonus: i32,
    pub dmg: i32,
    pub delay: i32,
    pub regen: i32,
    pub refresh: i32,
    pub regain: i32,
    pub fast_cast_pct: i32,
    pub quick_magic_pct: i32,
    pub snapshot_pct: i32,
    pub rapid_shot_pct: i32,
    pub double_shot_pct: i32,
    pub triple_shot_pct: i32,
    pub double_shot_damage_pct: i32,
    pub triple_shot_damage_pct: i32,
    pub recycle: i32,
    // 属性耐性 8 種
    pub resist_fire: i32,
    pub resist_ice: i32,
    pub resist_wind: i32,
    pub resist_earth: i32,
    pub resist_lightning: i32,
    pub resist_water: i32,
    pub resist_light: i32,
    pub resist_dark: i32,
    // 状態異常レジスト 16 種
    pub resist_sleep: i32,
    pub resist_paralysis: i32,
    pub resist_bind: i32,
    pub resist_silence: i32,
    pub resist_gravity: i32,
    pub resist_slow: i32,
    pub resist_petrification: i32,
    pub resist_stun: i32,
    pub resist_poison: i32,
    pub resist_charm: i32,
    pub resist_blind: i32,
    pub resist_curse: i32,
    pub resist_virus: i32,
    pub resist_amnesia: i32,
    pub resist_terror: i32,
    pub resist_death: i32,
}

impl EquipStats {
    /// JS 側の出力と突き合わせるための `(キー名, 値)` 一覧。
    /// キー名は移植元の JS 実装が返していたオブジェクトと一致させてある
    /// (JS との全件突き合わせに使うため。移植元は削除済み)。
    pub fn entries(&self) -> Vec<(&'static str, i32)> {
        vec![
            ("hp", self.hp),
            ("mp", self.mp),
            ("str", self.str_),
            ("dex", self.dex),
            ("vit", self.vit),
            ("agi", self.agi),
            ("int", self.int),
            ("mnd", self.mnd),
            ("chr", self.chr),
            ("hp_pct", self.hp_pct),
            ("mp_pct", self.mp_pct),
            ("def", self.def),
            ("attack", self.attack),
            ("accuracy", self.accuracy),
            ("evasion", self.evasion),
            ("attack_pct", self.attack_pct),
            ("ranged_attack", self.ranged_attack),
            ("ranged_accuracy", self.ranged_accuracy),
            ("magic_attack", self.magic_attack),
            ("magic_accuracy", self.magic_accuracy),
            ("magic_accuracy_skill", self.magic_accuracy_skill),
            ("magic_evasion", self.magic_evasion),
            ("magic_damage", self.magic_damage),
            ("haste_pct", self.haste_pct),
            ("store_tp", self.store_tp),
            ("double_attack_pct", self.double_attack_pct),
            ("triple_attack_pct", self.triple_attack_pct),
            ("quad_attack_pct", self.quad_attack_pct),
            ("double_attack_damage_pct", self.double_attack_damage_pct),
            ("triple_attack_damage_pct", self.triple_attack_damage_pct),
            ("critical_hit_rate_pct", self.critical_hit_rate_pct),
            ("critical_hit_damage_pct", self.critical_hit_damage_pct),
            ("weapon_skill_damage_pct", self.weapon_skill_damage_pct),
            ("subtle_blow", self.subtle_blow),
            ("subtle_blow_2", self.subtle_blow_2),
            ("tp_bonus", self.tp_bonus),
            ("skillchain_bonus", self.skillchain_bonus),
            ("physical_damage_limit_pct", self.physical_damage_limit_pct),
            ("true_shot", self.true_shot),
            ("magic_critical_hit_2_pct", self.magic_critical_hit_2_pct),
            ("magic_affinity", self.magic_affinity),
            ("magic_burst_damage", self.magic_burst_damage),
            ("magic_burst_damage_2", self.magic_burst_damage_2),
            ("conserve_mp", self.conserve_mp),
            (
                "elemental_recast_delay_pct",
                self.elemental_recast_delay_pct,
            ),
            ("blue_recast_delay_pct", self.blue_recast_delay_pct),
            ("song_recast_delay", self.song_recast_delay),
            ("ninjutsu_recast_delay", self.ninjutsu_recast_delay),
            ("damage_taken_pct", self.damage_taken_pct),
            ("physical_damage_taken_pct", self.physical_damage_taken_pct),
            ("magic_damage_taken_pct", self.magic_damage_taken_pct),
            ("magic_def_bonus", self.magic_def_bonus),
            ("dmg", self.dmg),
            ("delay", self.delay),
            ("regen", self.regen),
            ("refresh", self.refresh),
            ("regain", self.regain),
            ("fast_cast_pct", self.fast_cast_pct),
            ("quick_magic_pct", self.quick_magic_pct),
            ("snapshot_pct", self.snapshot_pct),
            ("rapid_shot_pct", self.rapid_shot_pct),
            ("double_shot_pct", self.double_shot_pct),
            ("triple_shot_pct", self.triple_shot_pct),
            ("double_shot_damage_pct", self.double_shot_damage_pct),
            ("triple_shot_damage_pct", self.triple_shot_damage_pct),
            ("recycle", self.recycle),
            ("resist_fire", self.resist_fire),
            ("resist_ice", self.resist_ice),
            ("resist_wind", self.resist_wind),
            ("resist_earth", self.resist_earth),
            ("resist_lightning", self.resist_lightning),
            ("resist_water", self.resist_water),
            ("resist_light", self.resist_light),
            ("resist_dark", self.resist_dark),
            ("resist_sleep", self.resist_sleep),
            ("resist_paralysis", self.resist_paralysis),
            ("resist_bind", self.resist_bind),
            ("resist_silence", self.resist_silence),
            ("resist_gravity", self.resist_gravity),
            ("resist_slow", self.resist_slow),
            ("resist_petrification", self.resist_petrification),
            ("resist_stun", self.resist_stun),
            ("resist_poison", self.resist_poison),
            ("resist_charm", self.resist_charm),
            ("resist_blind", self.resist_blind),
            ("resist_curse", self.resist_curse),
            ("resist_virus", self.resist_virus),
            ("resist_amnesia", self.resist_amnesia),
            ("resist_terror", self.resist_terror),
            ("resist_death", self.resist_death),
        ]
    }

    /// 別の `EquipStats` を項目ごとに加算する。JS の `sumStats` に対応。
    pub fn add(&mut self, other: &EquipStats) {
        self.hp += other.hp;
        self.mp += other.mp;
        self.str_ += other.str_;
        self.dex += other.dex;
        self.vit += other.vit;
        self.agi += other.agi;
        self.int += other.int;
        self.mnd += other.mnd;
        self.chr += other.chr;
        self.hp_pct += other.hp_pct;
        self.mp_pct += other.mp_pct;
        self.def += other.def;
        self.attack += other.attack;
        self.accuracy += other.accuracy;
        self.evasion += other.evasion;
        self.attack_pct += other.attack_pct;
        self.ranged_attack += other.ranged_attack;
        self.ranged_accuracy += other.ranged_accuracy;
        self.magic_attack += other.magic_attack;
        self.magic_accuracy += other.magic_accuracy;
        self.magic_accuracy_skill += other.magic_accuracy_skill;
        self.magic_evasion += other.magic_evasion;
        self.magic_damage += other.magic_damage;
        self.haste_pct += other.haste_pct;
        self.store_tp += other.store_tp;
        self.double_attack_pct += other.double_attack_pct;
        self.triple_attack_pct += other.triple_attack_pct;
        self.quad_attack_pct += other.quad_attack_pct;
        self.double_attack_damage_pct += other.double_attack_damage_pct;
        self.triple_attack_damage_pct += other.triple_attack_damage_pct;
        self.critical_hit_rate_pct += other.critical_hit_rate_pct;
        self.critical_hit_damage_pct += other.critical_hit_damage_pct;
        self.weapon_skill_damage_pct += other.weapon_skill_damage_pct;
        self.subtle_blow += other.subtle_blow;
        self.subtle_blow_2 += other.subtle_blow_2;
        self.tp_bonus += other.tp_bonus;
        self.skillchain_bonus += other.skillchain_bonus;
        self.physical_damage_limit_pct += other.physical_damage_limit_pct;
        self.true_shot += other.true_shot;
        self.magic_critical_hit_2_pct += other.magic_critical_hit_2_pct;
        self.magic_affinity += other.magic_affinity;
        self.magic_burst_damage += other.magic_burst_damage;
        self.magic_burst_damage_2 += other.magic_burst_damage_2;
        self.conserve_mp += other.conserve_mp;
        self.elemental_recast_delay_pct += other.elemental_recast_delay_pct;
        self.blue_recast_delay_pct += other.blue_recast_delay_pct;
        self.song_recast_delay += other.song_recast_delay;
        self.ninjutsu_recast_delay += other.ninjutsu_recast_delay;
        self.damage_taken_pct += other.damage_taken_pct;
        self.physical_damage_taken_pct += other.physical_damage_taken_pct;
        self.magic_damage_taken_pct += other.magic_damage_taken_pct;
        self.magic_def_bonus += other.magic_def_bonus;
        self.dmg += other.dmg;
        self.delay += other.delay;
        self.regen += other.regen;
        self.refresh += other.refresh;
        self.regain += other.regain;
        self.fast_cast_pct += other.fast_cast_pct;
        self.quick_magic_pct += other.quick_magic_pct;
        self.snapshot_pct += other.snapshot_pct;
        self.rapid_shot_pct += other.rapid_shot_pct;
        self.double_shot_pct += other.double_shot_pct;
        self.triple_shot_pct += other.triple_shot_pct;
        self.double_shot_damage_pct += other.double_shot_damage_pct;
        self.triple_shot_damage_pct += other.triple_shot_damage_pct;
        self.recycle += other.recycle;
        self.resist_fire += other.resist_fire;
        self.resist_ice += other.resist_ice;
        self.resist_wind += other.resist_wind;
        self.resist_earth += other.resist_earth;
        self.resist_lightning += other.resist_lightning;
        self.resist_water += other.resist_water;
        self.resist_light += other.resist_light;
        self.resist_dark += other.resist_dark;
        self.resist_sleep += other.resist_sleep;
        self.resist_paralysis += other.resist_paralysis;
        self.resist_bind += other.resist_bind;
        self.resist_silence += other.resist_silence;
        self.resist_gravity += other.resist_gravity;
        self.resist_slow += other.resist_slow;
        self.resist_petrification += other.resist_petrification;
        self.resist_stun += other.resist_stun;
        self.resist_poison += other.resist_poison;
        self.resist_charm += other.resist_charm;
        self.resist_blind += other.resist_blind;
        self.resist_curse += other.resist_curse;
        self.resist_virus += other.resist_virus;
        self.resist_amnesia += other.resist_amnesia;
        self.resist_terror += other.resist_terror;
        self.resist_death += other.resist_death;
    }

    /// 7 つの基本パラメータ (STR〜CHR) に一括加算する。
    fn add_all_base_params(&mut self, v: i32) {
        self.str_ += v;
        self.dex += v;
        self.vit += v;
        self.agi += v;
        self.int += v;
        self.mnd += v;
        self.chr += v;
    }

    /// 8 属性の耐性すべてに加算する。
    fn add_all_element_resists(&mut self, v: i32) {
        for f in [
            &mut self.resist_fire,
            &mut self.resist_ice,
            &mut self.resist_wind,
            &mut self.resist_earth,
            &mut self.resist_lightning,
            &mut self.resist_water,
            &mut self.resist_light,
            &mut self.resist_dark,
        ] {
            *f += v;
        }
    }

    /// デス耐性を除く 15 種の状態異常レジストに加算する。
    /// wiki 795.html: 「全状態異常のレジスト効果アップ」はデス耐性に作用しない。
    fn add_all_status_resists_except_death(&mut self, v: i32) {
        for f in [
            &mut self.resist_sleep,
            &mut self.resist_paralysis,
            &mut self.resist_bind,
            &mut self.resist_silence,
            &mut self.resist_gravity,
            &mut self.resist_slow,
            &mut self.resist_petrification,
            &mut self.resist_stun,
            &mut self.resist_poison,
            &mut self.resist_charm,
            &mut self.resist_blind,
            &mut self.resist_curse,
            &mut self.resist_virus,
            &mut self.resist_amnesia,
            &mut self.resist_terror,
        ] {
            *f += v;
        }
    }
}

// ---------------------------------------------------------------------------
// 抽出ヘルパー
// ---------------------------------------------------------------------------

/// 符号付きの値を **全出現ぶん合算** する。JS の `matchSigned` に対応。
/// 符号が省略可能な表記 (`"Snapshot"5`) にも使う。空文字の符号は `+` 扱い。
fn match_signed(text: &str, pattern: &str) -> i32 {
    let re = re(pattern);
    let mut total = 0;
    for caps in re.captures_iter(text).flatten() {
        let sign = if &caps[1] == "-" { -1 } else { 1 };
        total += sign * caps[2].parse::<i32>().unwrap_or(0);
    }
    total
}

/// コロン形式 (`DEF:77`) を **先頭 1 件だけ** 取る。JS の `matchColon` に対応。
///
/// 非グローバルなので、`In Dynamis: DEF:22` のような条件付きの値は
/// 先に書かれた無条件の値があれば拾われない。これは条件を解釈した結果ではなく
/// 偶然そうなっているだけである (docs/tech-debt/equip-stats-js-quirks.md)。
fn match_colon(text: &str, pattern: &str) -> i32 {
    re(pattern)
        .captures(text)
        .ok()
        .flatten()
        .and_then(|c| c[1].parse::<i32>().ok())
        .unwrap_or(0)
}

/// 装備説明文からステータスを抽出する。JS の `extractAllStats` に対応。
pub fn extract_all_stats(description_en: &str) -> EquipStats {
    if description_en.is_empty() {
        return EquipStats::default();
    }
    // 大小を区別する正規化を先に済ませてから小文字化する。
    // JS は正規化に i フラグを付けず、抽出にだけ付けているため。
    let text = normalize(description_en).to_ascii_lowercase();

    // 基本 9 ステータス (フラット)。`(?![0-9]*%)` でパーセント表記を除外する。
    let flat = |name: &str| {
        match_signed(
            &text,
            &format!(r"(?<![a-z]){name}{WS}*([+-]){WS}*([0-9]+)(?![0-9]*%)"),
        )
    };
    // 基本パラメータは `(?=[+-])` を挟む点だけ HP/MP と異なる (JS 側の書き分けを踏襲)。
    let base_param = |name: &str| {
        match_signed(
            &text,
            &format!(r"(?<![a-z]){name}{WS}*(?=[+-])([+-]){WS}*([0-9]+)(?![0-9]*%)"),
        )
    };
    let percent = |name: &str| {
        match_signed(
            &text,
            &format!(r"(?<![a-z]){name}{WS}*([+-]){WS}*([0-9]+)%"),
        )
    };

    let mut s = EquipStats {
        hp: flat("hp"),
        mp: flat("mp"),
        str_: base_param("str"),
        dex: base_param("dex"),
        vit: base_param("vit"),
        agi: base_param("agi"),
        int: base_param("int"),
        mnd: base_param("mnd"),
        chr: base_param("chr"),
        hp_pct: percent("hp"),
        mp_pct: percent("mp"),
        ..Default::default()
    };

    let signed = |p: &str| match_signed(&text, p);

    // === 防御 (コロン形式) ===
    s.def = match_colon(&text, r"def:([0-9]+)");

    // === 戦闘系。特定形を先に判定して誤マッチを避ける ===
    s.ranged_attack = signed(&format!(r"ranged attack{WS}*([+-]){WS}*([0-9]+)(?!%)"));
    s.ranged_accuracy = signed(&format!(r"ranged accuracy{WS}*([+-]){WS}*([0-9]+)"));
    // 属性別 Magic Atk. Bonus (`dark elemental "magic atk. bonus"+28`) は通常の魔攻に積まれない。
    // プライム武器 (アウゲー/ダデュコ/テロパノ) は引用符なしの非省略形
    // `Magic Attack Bonus +52` を使うため両形を受ける
    // 後読みは引用符あり/なし両方の開始位置で属性別を除外する
    s.magic_attack = signed(&format!(
        r#"(?<!elemental )(?<!elemental ")"?magic at(?:k\.?|tack) bonus"?{WS}*([+-]){WS}*([0-9]+)"#
    ));
    s.magic_accuracy = signed(&format!(r"magic accuracy{WS}*([+-]){WS}*([0-9]+)"));
    // 装備の "Magic Accuracy skill +X" は通常の魔命と別枠。
    s.magic_accuracy_skill = signed(&format!(r"magic accuracy skill{WS}*([+-]){WS}*([0-9]+)"));
    s.magic_evasion = signed(&format!(r"magic evasion{WS}*([+-]){WS}*([0-9]+)"));
    s.magic_damage = signed(&format!(r"magic damage{WS}*([+-]){WS}*([0-9]+)"));

    // 素の Attack/Accuracy/Evasion。後読みで Ranged/Magic 版を除外する。
    s.attack = signed(&format!(
        r"(?<!ranged )(?<![a-z])attack{WS}*([+-]){WS}*([0-9]+)(?!%)"
    ));
    s.accuracy = signed(&format!(
        r"(?<!ranged )(?<!magic )(?<!skill )(?<!burst )(?<![a-z])accuracy{WS}*([+-]){WS}*([0-9]+)"
    ));
    s.evasion = signed(&format!(
        r"(?<!magic )(?<![a-z])evasion{WS}*([+-]){WS}*([0-9]+)"
    ));
    s.attack_pct = signed(&format!(
        r"(?<!ranged )(?<![a-z])attack{WS}*([+-]){WS}*([0-9]+)%"
    ));

    // === レート・特殊系 ===
    s.haste_pct = signed(&format!(r"haste{WS}*([+-]){WS}*([0-9]+)%"));
    s.store_tp = signed(&format!(r#""?store tp"?{WS}*([+-]){WS}*([0-9]+)"#));
    // damage 系を先に判定する (部分一致回避)
    s.double_attack_damage_pct =
        signed(&format!(r"double attack damage{WS}*([+-]){WS}*([0-9]+)%?"));
    s.double_attack_pct = signed(&format!(r#""double attack"{WS}*([+-]){WS}*([0-9]+)%"#));
    s.triple_attack_damage_pct =
        signed(&format!(r"triple attack damage{WS}*([+-]){WS}*([0-9]+)%?"));
    s.triple_attack_pct = signed(&format!(r#""triple attack"{WS}*([+-]){WS}*([0-9]+)%"#));
    s.quad_attack_pct = signed(&format!(r#""quadruple attack"{WS}*([+-]){WS}*([0-9]+)%"#));
    s.critical_hit_damage_pct = signed(&format!(r"critical hit damage{WS}*([+-]){WS}*([0-9]+)%"));
    s.critical_hit_rate_pct = signed(&format!(r"critical hit rate{WS}*([+-]){WS}*([0-9]+)%"));

    // WS ダメージ: 属性ゴルゲット/ベルトや Rune Algol の値は "Latent effect:" 以下にあり
    // 発動条件付きのため常時加算しない。除外したうえで **先頭 1 件だけ** を採る。
    {
        // `(?s)` で `.` を改行にもマッチさせる (JS 側の `[\s\S]*` 相当)。
        // `\s` は unicode 機能を切ると使えないため。
        let non_latent = re(r"(?s)latent effect:.*").replace(&text, "").into_owned();
        s.weapon_skill_damage_pct = re(&format!(r"weapon skill damage{WS}*([+-]){WS}*([0-9]+)%"))
            .captures(&non_latent)
            .ok()
            .flatten()
            .map(|c| {
                let sign = if &c[1] == "-" { -1 } else { 1 };
                sign * c[2].parse::<i32>().unwrap_or(0)
            })
            .unwrap_or(0);
    }

    // モクシャ II を先に判定する (モクシャの部分一致回避)
    s.subtle_blow_2 = signed(&format!(r#""subtle blow ii"{WS}*([+-]){WS}*([0-9]+)"#));
    s.subtle_blow = signed(&format!(r#""subtle blow"{WS}*([+-]){WS}*([0-9]+)"#));

    // TP ボーナス。ペットや特定アビ専用の修飾語が直前にある場合は除外する。
    s.tp_bonus = signed(&format!(
        r#"(?<!avatar: )(?<!wyvern: )(?<!automaton: )(?<!all jumps )"?tp bonus"?{WS}*([+-]){WS}*([0-9]+)"#
    ));
    s.skillchain_bonus = signed(&format!(r#""?skillchain bonus"?{WS}*([+-]){WS}*([0-9]+)"#));
    s.physical_damage_limit_pct =
        signed(&format!(r"physical damage limit{WS}*([+-]){WS}*([0-9]+)%"));
    s.true_shot = signed(&format!(r#""true shot"{WS}*([+-]){WS}*([0-9]+)"#));
    s.magic_critical_hit_2_pct = signed(&format!(
        r"magic crit(?:ical|\.) hit rate ii{WS}*([+-]){WS}*([0-9]+)%"
    ));
    s.magic_affinity = signed(&format!(r"(?<![a-z])affinity{WS}*([+-]){WS}*([0-9]+)"));

    // === マジックバースト / 詠唱系 ===
    // II を先に判定する必要はない (I のパターンは "damage" 直後に符号を要求するため
    // "damage ii" には一致しない)。数値なしの旧表記
    // "Bonus damage added to magic burst" はどちらにも一致しない
    s.magic_burst_damage_2 = signed(&format!(
        r"magic burst damage{WS}*ii{WS}*([+-]){WS}*([0-9]+)"
    ));
    s.magic_burst_damage = signed(&format!(r"magic burst damage{WS}*([+-]){WS}*([0-9]+)"));
    s.conserve_mp = signed(&format!(r#""conserve mp"{WS}*([+-]){WS}*([0-9]+)"#));
    // 再詠唱間隔: 精霊/青は %、歌/忍術は秒 (符号込みで負数として抽出)
    s.elemental_recast_delay_pct = signed(&format!(
        r"elemental magic recast delay{WS}*([+-]){WS}*([0-9]+)%"
    ));
    s.blue_recast_delay_pct = signed(&format!(
        r"blue magic recast delay{WS}*([+-]){WS}*([0-9]+)%"
    ));
    s.song_recast_delay = signed(&format!(r"song recast delay{WS}*([+-]){WS}*([0-9]+)"));
    s.ninjutsu_recast_delay = signed(&format!(r"ninjutsu recast delay{WS}*([+-]){WS}*([0-9]+)"));

    // === 被ダメージ系 ===
    s.damage_taken_pct = signed(&format!(
        r"(?<!physical )(?<!magic )damage taken{WS}*([+-]){WS}*([0-9]+)%"
    ));
    s.physical_damage_taken_pct =
        signed(&format!(r"physical damage taken{WS}*([+-]){WS}*([0-9]+)%"));
    s.magic_damage_taken_pct = signed(&format!(r"magic damage taken{WS}*([+-]){WS}*([0-9]+)%"));
    s.magic_def_bonus = signed(&format!(r#""magic def\.? bonus"{WS}*([+-]){WS}*([0-9]+)"#));

    // === 自動回復・自動 TP ===
    s.regen = signed(&format!(r#""?regen"?{WS}*([+-]){WS}*([0-9]+)"#));
    s.refresh = signed(&format!(r#""?refresh"?{WS}*([+-]){WS}*([0-9]+)"#));
    s.regain = signed(&format!(r#""?regain"?{WS}*([+-]){WS}*([0-9]+)"#));

    // === 詠唱・ジョブアビ短縮系 ===
    // 表記揺れが大きく、符号も % も省略されうる ("Snapshot"5 など)。
    s.fast_cast_pct = signed(&format!(r#""?fast cast"?{WS}*([+-]?){WS}*([0-9]+)%?"#));
    s.quick_magic_pct = signed(&format!(r#""?quick magic"?{WS}*([+-]?){WS}*([0-9]+)%?"#));
    s.snapshot_pct = signed(&format!(r#""?snapshot"?{WS}*([+-]?){WS}*([0-9]+)%?"#));
    s.rapid_shot_pct = signed(&format!(r#""?rapid shot"?{WS}*([+-]?){WS}*([0-9]+)%?"#));

    // === 遠隔プロパティ ===
    // ダブル/トリプルショットは "+N%" 形式のみ対象。
    // "Double Shot" damage+N (ダメージ増) や Enhances "..." effect (効果アップ) は
    // 別プロパティのため一致させない
    s.double_shot_pct = signed(&format!(r#""double shot"{WS}*([+-]){WS}*([0-9]+)%"#));
    s.triple_shot_pct = signed(&format!(r#""triple shot"{WS}*([+-]){WS}*([0-9]+)%"#));
    // ダメージ増は "damage +N" / "damage+N" の空白ゆれあり・% なし
    s.double_shot_damage_pct = signed(&format!(
        r#""double shot"{WS}*damage{WS}*([+-]){WS}*([0-9]+)%?"#
    ));
    s.triple_shot_damage_pct = signed(&format!(
        r#""triple shot"{WS}*damage{WS}*([+-]){WS}*([0-9]+)%?"#
    ));
    // リサイクルは単位なし ("Recycle"+15)。数値なしの Adds "Recycle" effect は対象外
    s.recycle = signed(&format!(r#""recycle"{WS}*([+-]){WS}*([0-9]+)"#));

    // === 属性レジスト ===
    let elem_resist = |elem: &str| {
        match_signed(
            &text,
            &format!(r"(?:{elem} resistance|resist {elem}){WS}*([+-]){WS}*([0-9]+)"),
        )
    };
    s.resist_fire = elem_resist("fire");
    s.resist_ice = elem_resist("ice");
    s.resist_wind = elem_resist("wind");
    s.resist_earth = elem_resist("earth");
    s.resist_lightning = elem_resist("lightning");
    s.resist_water = elem_resist("water");
    s.resist_light = elem_resist("light");
    s.resist_dark = elem_resist("dark");

    // === 状態異常レジスト ===
    // 通常 14 種は `"Resist X"+N`、Terror と Death だけ表記が変則的。
    let status_resist = |pat: &str| match_signed(&text, &format!(r"{pat}{WS}*([+-]){WS}*([0-9]+)"));
    s.resist_sleep = status_resist(r#""?resist sleep"?"#);
    s.resist_paralysis = status_resist(r#""?resist paralyze"?"#);
    s.resist_bind = status_resist(r#""?resist bind"?"#);
    s.resist_silence = status_resist(r#""?resist silence"?"#);
    s.resist_gravity = status_resist(r#""?resist gravity"?"#);
    s.resist_slow = status_resist(r#""?resist slow"?"#);
    s.resist_petrification = status_resist(r#""?resist petrify"?"#);
    s.resist_stun = status_resist(r#""?resist stun"?"#);
    s.resist_poison = status_resist(r#""?resist poison"?"#);
    s.resist_charm = status_resist(r#""?resist charm"?"#);
    s.resist_blind = status_resist(r#""?resist blind"?"#);
    s.resist_curse = status_resist(r#""?resist curse"?"#);
    s.resist_virus = status_resist(r#""?resist virus"?"#);
    s.resist_amnesia = status_resist(r#""?resist amnesia"?"#);
    s.resist_terror = status_resist(r#"(?:terror resistance|"?resist terror"?)"#);
    s.resist_death =
        status_resist(r#"(?:resistance against "death"|"death" resistance|"?resist death"?)"#);

    // 全状態異常レジスト一括 (デス耐性を除く 15 種)。
    // JS は `A || B` で、A が 0 のときだけ B を評価する。
    let all_status = {
        let a = signed(&format!(
            r"resistance to all status ailments{WS}*([+-]){WS}*([0-9]+)"
        ));
        if a != 0 {
            a
        } else {
            signed(&format!(
                r"all status ailments? resistance{WS}*([+-]){WS}*([0-9]+)"
            ))
        }
    };
    if all_status != 0 {
        s.add_all_status_resists_except_death(all_status);
    }

    // === 武器 (コロン形式) ===
    s.dmg = match_colon(&text, r"dmg:\+?([0-9]+)");
    s.delay = match_colon(&text, r"delay:\+?([0-9]+)");

    // ALL BP: 7 つの基本パラメータすべてに加算する。
    // JS 同様、個別指定を設定した「後」に加算するため合算になる。
    let all_bp = signed(&format!(r"all{WS}*bp{WS}*([+-]){WS}*([0-9]+)"));
    if all_bp != 0 {
        s.add_all_base_params(all_bp);
    }

    // 全属性耐性: 8 属性すべてに加減算する。
    let all_element = signed(&format!(
        r"(?:all{WS}+elemental{WS}+resistances|resist{WS}+all{WS}+elements){WS}*([+-]){WS}*([0-9]+)"
    ));
    if all_element != 0 {
        s.add_all_element_resists(all_element);
    }

    s
}

// ---------------------------------------------------------------------------
// スキルボーナス
// ---------------------------------------------------------------------------

/// 戦闘 19 + 魔法 14 = 33 種のスキル。キーは `SkillKind` のシリアライズ名に合わせる。
const SKILL_PATTERNS: [(&str, &str); 33] = [
    // 武器スキル 15 種。"skill" を必須にして "Evasion+5" 等との衝突を避ける。
    ("HandToHand", r"(?<![a-z])hand-to-hand +skill"),
    ("Dagger", r"(?<![a-z])dagger +skill"),
    ("GreatSword", r"(?<![a-z])great +sword +skill"),
    ("Sword", r"(?<!great )(?<![a-z])sword +skill"),
    ("GreatAxe", r"(?<![a-z])great +axe +skill"),
    ("Axe", r"(?<!great )(?<![a-z])axe +skill"),
    ("Scythe", r"(?<![a-z])scythe +skill"),
    ("Polearm", r"(?<![a-z])polearm +skill"),
    ("GreatKatana", r"(?<![a-z])great +katana +skill"),
    ("Katana", r"(?<!great )(?<![a-z])katana +skill"),
    ("Club", r"(?<![a-z])club +skill"),
    ("Staff", r"(?<![a-z])staff +skill"),
    ("Archery", r"(?<![a-z])archery +skill"),
    ("Marksmanship", r"(?<![a-z])marksmanship +skill"),
    ("Throwing", r"(?<![a-z])throwing +skill"),
    // 防御スキル 4 種
    ("Guarding", r"(?<![a-z])guarding +skill"),
    ("Evasion", r"(?<![a-z])evasion +skill"),
    ("Shield", r"(?<![a-z])shield +skill"),
    ("Parrying", r"(?<![a-z])parrying +skill"),
    // 魔法スキル 14 種。Geomancy と Handbell だけ "skill" 省略形もある。
    ("Divine", r"(?<![a-z])divine +magic +skill"),
    ("Healing", r"(?<![a-z])healing +magic +skill"),
    ("Enhancing", r"(?<![a-z])enhancing +magic +skill"),
    ("Enfeebling", r"(?<![a-z])enfeebling +magic +skill"),
    ("Elemental", r"(?<![a-z])elemental +magic +skill"),
    ("Dark", r"(?<![a-z])dark +magic +skill"),
    ("Summoning", r"(?<![a-z])summoning +magic +skill"),
    ("Ninjutsu", r"(?<![a-z])ninjutsu +skill"),
    ("Singing", r"(?<![a-z])singing +skill"),
    ("StringInstrument", r"(?<![a-z])string +instrument +skill"),
    ("WindInstrument", r"(?<![a-z])wind +instrument +skill"),
    ("BlueMagic", r"(?<![a-z])blue +magic +skill"),
    ("Geomancy", r"(?<![a-z])geomancy(?: +skill)?"),
    ("Handbell", r"(?<![a-z])handbell(?: +skill)?"),
];

/// 「全魔法スキル」一括加算の対象 14 種。
const ALL_MAGIC_SKILLS: [&str; 14] = [
    "Divine",
    "Healing",
    "Enhancing",
    "Enfeebling",
    "Elemental",
    "Dark",
    "Summoning",
    "Ninjutsu",
    "Singing",
    "StringInstrument",
    "WindInstrument",
    "BlueMagic",
    "Geomancy",
    "Handbell",
];

/// 装備説明文からスキルボーナスを抽出する。JS の `extractSkillBonuses` に対応。
/// 戻り値は非ゼロのエントリのみ (JS の挙動どおり)。
pub fn extract_skill_bonuses(description_en: &str) -> BTreeMap<&'static str, i32> {
    let mut result: BTreeMap<&'static str, i32> = BTreeMap::new();
    if description_en.is_empty() {
        return result;
    }
    let mut text = description_en.replace("\\n", "\n");

    // "A/B magic skill +X" を 2 エントリに展開する。
    // 末尾に modifier (magic/instrument) か "skill" がある場合のみ展開し、
    // "STR/VIT+10" 形式には適用しない。JS 側はここだけ i フラグ付きなので、
    // 小文字化の前に大小無視で処理する必要がある。ここでは明示的な文字クラスで表す。
    let slash = re(
        r"([A-Za-z][0-9A-Za-z_-]*(?:/[A-Za-z][0-9A-Za-z_-]*)+)((?: |\t|\n)+(?:[Mm][Aa][Gg][Ii][Cc]|[Ii][Nn][Ss][Tt][Rr][Uu][Mm][Ee][Nn][Tt]))?((?: |\t|\n)+[Ss][Kk][Ii][Ll][Ll])?[ \t]*([+-][ \t]*[0-9]+)",
    );
    text = slash
        .replace_all(&text, |caps: &fancy_regex::Captures| {
            let modifier = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let skill_word = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            if modifier.is_empty() && skill_word.is_empty() {
                return caps[0].to_string();
            }
            let val = &caps[4];
            caps[1]
                .split('/')
                .map(|n| format!("{}{modifier}{skill_word} {val}", n.trim()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .into_owned();

    let text = text.to_ascii_lowercase();

    for (key, pattern) in SKILL_PATTERNS {
        let v = match_signed(&text, &format!(r"{pattern}{WS}*([+-]){WS}*([0-9]+)"));
        if v != 0 {
            *result.entry(key).or_insert(0) += v;
        }
    }

    // 全魔法スキル一括: "Magic skills +N" / "All magic skills +N" (複数形)。
    // "Healing magic skill +5" は単数形なのでマッチしない。
    let all_magic = match_signed(
        &text,
        &format!(r"(?<![a-z])(?:all{WS}+)?magic{WS}+skills{WS}*([+-]){WS}*([0-9]+)"),
    );
    if all_magic != 0 {
        for key in ALL_MAGIC_SKILLS {
            *result.entry(key).or_insert(0) += all_magic;
        }
    }

    result.retain(|_, v| *v != 0);
    result
}

// ---------------------------------------------------------------------------
// オーグメントの日本語表記 → 英語表記
// ---------------------------------------------------------------------------

/// 日本語のオーグメント表記を、抽出可能な英語表記へ置き換える対応表。
///
/// **並び順に意味がある。** 単純な順次置換なので、長い表記を先に置かないと
/// 部分一致で壊れる (例: 「魔法クリティカルヒットII」を「魔法クリティカルヒット率」
/// より先に、「攻」を最後に置く)。`web/js/constants.js` の順序をそのまま保つこと。
static AUGMENT_JA_TO_EN: [(&str, &str); 88] = [
    ("ウェポンスキルのダメージ", "Weapon skill damage"),
    ("マジックバーストダメージ", "Magic burst damage"),
    // 順序重要: 「マジックバースト命中」→「マジックバースト」の順。
    // 「マジックバースト+N」(ソーサラストール系) は MB ダメージの表記揺れ
    ("マジックバースト命中", "Magic burst accuracy"),
    ("マジックバースト", "Magic burst damage"),
    ("コンサーブMP", "\"Conserve MP\""),
    ("精霊魔法の再詠唱間隔", "Elemental magic recast delay"),
    ("青魔法の再詠唱間隔", "Blue magic recast delay"),
    ("歌再詠唱間隔", "Song recast delay"),
    ("忍術再詠唱間隔", "Ninjutsu recast delay"),
    ("魔法クリティカルヒットII", "Magic Crit. Hit Rate II"),
    ("魔法クリティカルヒット率", "Magic Critical hit rate"),
    ("物理ダメージ上限", "Physical damage limit"),
    ("被物理ダメージ", "Physical damage taken"),
    ("被魔法ダメージ", "Magic damage taken"),
    ("クリティカルヒットダメージ", "Critical hit damage"),
    ("クリティカルヒット率", "Critical hit rate"),
    ("トリプルアタックダメージ", "Triple Attack damage"),
    ("トリプルアタック", "\"Triple Attack\""),
    ("ダブルアタックダメージ", "Double Attack damage"),
    ("ダブルアタック", "\"Double Attack\""),
    ("クワッドアタック", "\"Quadruple Attack\""),
    ("モクシャII", "\"Subtle Blow II\""),
    ("モクシャ", "\"Subtle Blow\""),
    ("魔法ダメージ", "Magic Damage"),
    // 個別魔法スキル名 (「魔法スキル」より先に置換しないと「強化魔法スキル」→「強化Magic skills」になり
    //                   汎用パターンで全 14 スキルに加算される regression が発生する)
    ("神聖魔法スキル", "Divine magic skill"),
    ("回復魔法スキル", "Healing magic skill"),
    ("強化魔法スキル", "Enhancing magic skill"),
    ("弱体魔法スキル", "Enfeebling magic skill"),
    ("精霊魔法スキル", "Elemental magic skill"),
    ("暗黒魔法スキル", "Dark magic skill"),
    ("召喚魔法スキル", "Summoning magic skill"),
    ("青魔法スキル", "Blue magic skill"),
    ("風水魔法スキル", "Geomancy skill"),
    ("忍術スキル", "Ninjutsu skill"),
    ("歌唱スキル", "Singing skill"),
    ("弦楽器スキル", "String instrument skill"),
    ("管楽器スキル", "Wind instrument skill"),
    ("風水鈴スキル", "Handbell skill"),
    // 全魔法スキル一括加算 (extractSkillBonuses で 14 種に展開される)
    ("魔法スキル", "Magic skills"),
    // 状態異常レジスト
    // 順序重要: 「全状態異常のレジスト」を個別レジストより先に置く (汎用パターン後置)
    ("全状態異常のレジスト", "Resistance to all status ailments"),
    // デス/テラーは items.json で「耐性」表記、EN も独自形式 (引用符 + resistance / 引用符なし)
    ("デス耐性", "\"Death\" resistance"),
    ("テラー耐性", "Terror resistance"),
    // 残り 14 種: JA「レジストX」 ↔ EN「"Resist X"」 (引用符付き、X は短縮形)
    ("レジストアムネジア", "\"Resist Amnesia\""),
    ("レジストペトリ", "\"Resist Petrify\""),
    ("レジストパライズ", "\"Resist Paralyze\""),
    ("レジストグラビデ", "\"Resist Gravity\""),
    ("レジストグラビティ", "\"Resist Gravity\""),
    ("レジストサイレス", "\"Resist Silence\""),
    ("レジストスリープ", "\"Resist Sleep\""),
    ("レジストポイズン", "\"Resist Poison\""),
    ("レジストチャーム", "\"Resist Charm\""),
    ("レジストブライン", "\"Resist Blind\""),
    ("レジストカース", "\"Resist Curse\""),
    ("レジストウィルス", "\"Resist Virus\""),
    ("レジストバインド", "\"Resist Bind\""),
    ("レジストスロウ", "\"Resist Slow\""),
    ("レジストスタン", "\"Resist Stun\""),
    // 属性耐性 (装備の「耐火+15」等を EN 化)
    // 順序重要: 全耐性 系 → 個別耐性 (個別が「全」の文字列に部分マッチしないが念のため先に置く)
    ("全属性耐性", "All elemental resistances"),
    ("全耐性", "All elemental resistances"),
    ("耐火", "Fire Resistance"),
    ("耐氷", "Ice Resistance"),
    ("耐風", "Wind Resistance"),
    ("耐土", "Earth Resistance"),
    ("耐雷", "Lightning Resistance"),
    ("耐水", "Water Resistance"),
    ("耐光", "Light Resistance"),
    ("耐闇", "Dark Resistance"),
    ("被ダメージ", "Damage taken"),
    ("ストアTP", "\"Store TP\""),
    ("TPボーナス", "\"TP Bonus\""),
    ("連携ボーナス", "\"Skillchain Bonus\""),
    // 連携ダメージ +N% (Mpaca 系オーグメント等) は内部的に Skillchain Bonus と同種扱い
    ("連携ダメージ", "\"Skillchain Bonus\""),
    ("トゥルーショット", "\"True Shot\""),
    // ダメージ増を先に変換する (ダブルショット単体の部分一致回避)
    ("ダブルショットダメージ", "\"Double Shot\" damage"),
    ("ダブルショット", "\"Double Shot\""),
    ("トリプルショットダメージ", "\"Triple Shot\" damage"),
    ("トリプルショット", "\"Triple Shot\""),
    ("リサイクル", "\"Recycle\""),
    ("アフィニティ", "Affinity"),
    ("ヘイスト", "Haste"),
    ("魔回避", "Magic Evasion"),
    ("飛攻", "Ranged Attack"),
    ("飛命", "Ranged Accuracy"),
    ("魔命", "Magic Accuracy"),
    ("魔攻", "\"Magic Atk. Bonus\""),
    ("回避", "Evasion"),
    ("命中", "Accuracy"),
    ("攻", "Attack"),
];

/// 日本語のオーグメント表記を英語表記に変換する。
/// JS の `convertAugmentJaToEn` (web/js/utils.js) に対応。
pub fn convert_augment_ja_to_en(text: &str) -> String {
    let mut result = text.to_string();
    for (ja, en) in AUGMENT_JA_TO_EN {
        if result.contains(ja) {
            result = result.replace(ja, en);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// 任意名の抽出 (日本語テキスト向け)
//
// 上の `extract_all_stats` が「英語説明文から固定 26 種を全一致合算」なのに対し、
// こちらは「日本語テキストから呼び出し側が指定した 1 種を最初の一致だけ」取り出す。
// 検索の説明文ステータスソート (item_search) と、プロパティセットのユーザー定義項目
// (docs/adr/0015) が使う。もとは item_search.rs にあったが、説明文の解釈は
// このモジュールが持つ (docs/adr/0018)。
// ---------------------------------------------------------------------------
/// 説明文中の「条件ラベル」の適用範囲 (`:` の直後から行末まで) を返す。
///
/// 日本語説明文では `ペット:` `潜在能力:` `右耳:` のようなコロン付きラベルが、
/// その行の残り全体の適用対象・適用条件を表す
/// (例: `防21 ペット:命中+3 モクシャ+3` の 命中/モクシャ はどちらもペットのもの)。
/// キャラクター本体に常時乗る値ではないので、抽出の対象から外す。
/// 範囲が行末までで折り返し行に及ばないのは、実データの折り返し行が
/// 本体の効果に戻っているため (例 ＰＮチュリダル+2
/// `オートマトン:魔命+9` / `ファストキャスト効果アップ`)。
///
/// ラベルの判定条件を「非 ASCII 文字を含むこと」にしているのは、英語説明文の
/// `DMG:+165 Delay:+240 STR+10` のような `ステータス:値` 表記を条件ラベルと
/// 誤認しないため (description_ja が無い装備は description_en で代替する)。
/// ラベル自身は範囲に含めないので、`DEF:77` から `DEF` を引く従来の用法は残る。
///
/// 行の区切りは実際の改行とリテラルの `\n` の両方を見る。items.json の
/// description_ja は改行をリテラルの `\n` で持つ (crate::items)。
fn conditional_label_scopes(chars: &[char]) -> Vec<(usize, usize)> {
    // リテラルの `\n` は大文字化を通った後なので `\N` になっている
    let line_break_at = |i: usize| -> Option<usize> {
        match chars.get(i) {
            Some('\n') => Some(1),
            Some('\\') if matches!(chars.get(i + 1), Some('n') | Some('N')) => Some(2),
            _ => None,
        }
    };
    let mut scopes = Vec::new();
    // 現在のトークン (直前の空白/コロン/行頭以降) の開始位置と、非 ASCII を含むか
    let mut token_start = 0usize;
    let mut token_has_non_ascii = false;
    let mut i = 0usize;
    while i < chars.len() {
        if let Some(width) = line_break_at(i) {
            i += width;
            token_start = i;
            token_has_non_ascii = false;
            continue;
        }
        let c = chars[i];
        if c == ':' && i > token_start && token_has_non_ascii {
            let mut end = i + 1;
            while end < chars.len() && line_break_at(end).is_none() {
                end += 1;
            }
            scopes.push((i + 1, end));
            i = end;
            continue;
        }
        if c == ':' || c.is_whitespace() {
            token_start = i + 1;
            token_has_non_ascii = false;
        } else if !c.is_ascii() {
            token_has_non_ascii = true;
        }
        i += 1;
    }
    scopes
}

/// 説明文からステータス値を取り出す。ソート用。
/// 全角英数と `＋` `－` `―` を半角に直してから、最初に一致した値を返す。
/// 条件ラベル (`ペット:` など) の配下は対象外 (`conditional_label_scopes`)。
pub fn extract_stat_from_description(description: &str, stat_name: &str) -> i32 {
    if description.is_empty() || stat_name.is_empty() {
        return 0;
    }
    let normalized: String = description
        .chars()
        .map(|c| {
            let code = c as u32;
            match code {
                0xFF21..=0xFF3A => char::from_u32(code - 0xFF21 + 0x41).unwrap_or(c),
                0xFF41..=0xFF5A => char::from_u32(code - 0xFF41 + 0x61).unwrap_or(c),
                0xFF10..=0xFF19 => char::from_u32(code - 0xFF10 + 0x30).unwrap_or(c),
                0xFF0B => '+',
                0xFF0D | 0x2015 => '-',
                _ => c,
            }
        })
        .flat_map(|c| c.to_uppercase())
        .collect();
    let needle = stat_name.to_uppercase();

    // JS 側は `${stat}\s*(?::\s*)?([+\-]?)\s*(\d+)` を i フラグ付きで 1 回だけ照合する。
    // 大小は両辺を大文字化して揃えてあるので、ここでは手で走査する。
    let bytes: Vec<char> = normalized.chars().collect();
    let pat: Vec<char> = needle.chars().collect();
    let scopes = conditional_label_scopes(&bytes);
    let mut i = 0usize;
    while i + pat.len() <= bytes.len() {
        if bytes[i..i + pat.len()] != pat[..] {
            i += 1;
            continue;
        }
        if scopes.iter().any(|&(s, e)| i >= s && i < e) {
            i += 1;
            continue;
        }
        let mut j = i + pat.len();
        let skip_ws = |j: &mut usize| {
            while *j < bytes.len() && bytes[*j].is_whitespace() {
                *j += 1;
            }
        };
        skip_ws(&mut j);
        if j < bytes.len() && bytes[j] == ':' {
            j += 1;
            skip_ws(&mut j);
        }
        let mut sign = 1;
        if j < bytes.len() && (bytes[j] == '+' || bytes[j] == '-') {
            if bytes[j] == '-' {
                sign = -1;
            }
            j += 1;
        }
        skip_ws(&mut j);
        let start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > start {
            let num: String = bytes[start..j].iter().collect();
            return sign * num.parse::<i32>().unwrap_or(0);
        }
        i += 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::item_by_id;

    fn desc(id: u32) -> String {
        item_by_id(id)
            .unwrap_or_else(|| panic!("id {id} not found in items.json"))
            .description_en
            .clone()
            .unwrap_or_else(|| panic!("id {id} has no description_en"))
    }

    // --- 前処理: リテラル改行 ---------------------------------------------

    #[test]
    fn normalize_converts_literal_newline() {
        assert_eq!(normalize(r"Attack+5\nAccuracy+3"), "Attack+5\nAccuracy+3");
    }

    // --- 前処理: 属性アイコン (F1) ----------------------------------------

    #[test]
    fn normalize_expands_element_icons() {
        let out = normalize("\u{e000}+10 \u{e007}-5");
        assert_eq!(out, "Fire Resistance +10 Dark Resistance -5");
    }

    #[test]
    fn normalize_expands_icons_in_real_item() {
        // 実データでアイコンを含む装備。正規化後は英語表記になる。
        let out = normalize(&desc(10287));
        assert!(
            !out.chars().any(|c| ('\u{e000}'..='\u{e007}').contains(&c)),
            "PUA が残っている: {out}"
        );
    }

    // --- 前処理: Unity Ranking (F2) ---------------------------------------

    #[test]
    fn normalize_unity_ranking_takes_max() {
        assert_eq!(
            normalize("Unity Ranking: Attack+10～15"),
            " Attack+15",
            "全角チルダ U+FF5E"
        );
        assert_eq!(normalize("Unity Ranking: Evasion+15~20"), " Evasion+20");
    }

    // --- 前処理: Pet セグメント除去 (E1) ----------------------------------

    #[test]
    fn normalize_strips_pet_segment_until_next_colon() {
        // 次の ":" まで (改行を跨ぐ) を除去する。
        // Pet 行は折り返しで複数行に分かれることがあり、すべてペット用として扱う。
        let out = normalize("HP+100\nPet: Accuracy+15\nRanged Accuracy+15\nHaste+5%");
        assert!(out.contains("HP+100"));
        assert!(!out.contains("Accuracy"), "Pet 行が残っている: {out}");
        assert!(!out.contains("Haste"), "折り返し行も除去する: {out}");
    }

    #[test]
    fn normalize_pet_strip_eats_following_label() {
        // JS の既知の癖を再現する。`[^:]*` が次のコロン直前まで食うため、
        // Pet セグメントの後に続く "DEF:" などのラベルまで消える。
        // node で確認: "HP+100\nPet: X\nDEF:10" → "HP+100\n:10"
        // 実データでは Lion Tamer (17961, DEF) と Neo Animator (21433, DMG) の 2 件が該当する。
        // 挙動を変えないという移植方針 (docs/adr/0010) に従い、そのまま再現する。
        let out = normalize("HP+100\nPet: Accuracy+15\nDEF:10");
        assert_eq!(out, "HP+100\n:10");
    }

    // --- 前処理: 略記展開 (A1) --------------------------------------------

    #[test]
    fn normalize_expands_abbreviations() {
        assert_eq!(normalize("Rng. Acc.+15"), "Ranged Accuracy+15");
        assert_eq!(normalize("Rng.Atk.+20"), "Ranged Attack+20");
        assert_eq!(normalize("Mag. Acc.+10"), "Magic Accuracy+10");
        assert_eq!(normalize("Mag. Atk. Bonus+13"), "Magic Atk. Bonus+13");
        assert_eq!(normalize("M.Def.B.+5"), "Magic Def. Bonus+5");
    }

    #[test]
    fn normalize_expands_simple_abbreviations_only_before_number() {
        assert_eq!(normalize("Acc.+10"), "Accuracy+10");
        // 数値が続かない場合は展開しない
        assert_eq!(normalize("Acc. bonus"), "Acc. bonus");
    }

    #[test]
    fn normalize_expands_abbreviations_in_real_item() {
        let out = normalize(&desc(10293));
        assert!(!out.contains("Rng. Acc."), "略記が残っている: {out}");
    }

    // --- 前処理: スラッシュ結合 (G) ---------------------------------------
    // 実データには出現しないが、JS が対応しているので移植する。

    #[test]
    fn normalize_expands_slash_separated_stats() {
        assert_eq!(normalize("STR/VIT+10"), "STR+10 VIT+10");
        assert_eq!(normalize("STR/DEX/AGI-5"), "STR-5 DEX-5 AGI-5");
    }

    // --- 前処理: 大小の順序 (A2) ------------------------------------------

    #[test]
    fn normalize_is_case_sensitive_like_js() {
        // JS 側の正規化は i フラグを付けていない。小文字入力は展開されない。
        assert_eq!(normalize("rng. acc.+15"), "rng. acc.+15");
    }

    // =====================================================================
    // 基本ステータス
    //
    // 期待値は移植時に JS 実装を実際に動かして得た値。思い込みで書くと
    // 移植の検証にならないため、当時は JS をリファレンス実装として扱った
    // (取得に使ったハーネス scripts/dump_equip_stats.js は移植完了後に削除済み)。
    // =====================================================================

    #[test]
    fn basic_stats_from_real_item() {
        // ヒポメネソックス+1
        // 対象: id 27410
        let s = extract_all_stats(&desc(27410));
        assert_eq!(s.hp, 13);
        assert_eq!(s.mp, 14);
        assert_eq!(s.str_, 10);
        assert_eq!(s.dex, 11);
        assert_eq!(s.vit, 10);
        assert_eq!(s.agi, 33);
        assert_eq!(s.int, 17);
        assert_eq!(s.mnd, 19);
        assert_eq!(s.chr, 34);
    }

    #[test]
    fn signed_values_are_summed() {
        // JS: 'HP+10 HP+20' -> {"hp": 30}
        assert_eq!(extract_all_stats("HP+10 HP+20").hp, 30);
    }

    #[test]
    fn negative_values() {
        // JS: 'STR-3' -> {"str": -3}
        assert_eq!(extract_all_stats("STR-3").str_, -3);
    }

    // --- パーセント表記の分離 (E2) ----------------------------------------

    #[test]
    fn percent_does_not_leak_into_flat_stat() {
        // JS: 'HP+10%' -> {"hp_pct": 10}  (hp は付かない)
        let s = extract_all_stats("HP+10%");
        assert_eq!(s.hp, 0, "パーセント表記が flat hp に混入している");
        assert_eq!(s.hp_pct, 10);
    }

    #[test]
    fn flat_and_percent_coexist() {
        // JS: 'HP+50 HP+10%' -> {"hp": 50, "hp_pct": 10}
        let s = extract_all_stats("HP+50 HP+10%");
        assert_eq!(s.hp, 50);
        assert_eq!(s.hp_pct, 10);
    }

    #[test]
    fn percent_from_real_item() {
        // マタンサハーネス: DEF:77 HP+8% STR+15 DEX+15 VIT+15 ...
        // 対象: id 10255
        let s = extract_all_stats(&desc(10255));
        assert_eq!(s.hp_pct, 8);
        assert_eq!(s.hp, 0);
        assert_eq!(s.str_, 15);
        assert_eq!(s.dex, 15);
        assert_eq!(s.vit, 15);
    }

    // --- ALL BP (D1 / D2) -------------------------------------------------

    #[test]
    fn all_bp_applies_to_seven_base_params() {
        // JS: 'ALL BP+10'
        let s = extract_all_stats("ALL BP+10");
        assert_eq!(
            (s.str_, s.dex, s.vit, s.agi, s.int, s.mnd, s.chr),
            (10, 10, 10, 10, 10, 10, 10)
        );
        // HP/MP は対象外
        assert_eq!((s.hp, s.mp), (0, 0));
    }

    #[test]
    fn all_bp_adds_to_individual_stat() {
        // JS: 'STR+5 ALL BP+10' -> str 15, 他は 10
        let s = extract_all_stats("STR+5 ALL BP+10");
        assert_eq!(s.str_, 15);
        assert_eq!(s.dex, 10);
    }

    #[test]
    fn all_bp_range_notation_takes_first_match() {
        // ホクスニピアス: "Mastery Rank: All BP -30 to +30"
        // JS は最初の符号付き値 (-30) を採用する。"to +30" は "All BP" が
        // 直前に無いためマッチしない。Unity Ranking のような最大値採用は行わない。
        // 対象: id 26120 (JS でも全項目 -30 になる)
        // 挙動を変えない方針 (docs/adr/0010) によりそのまま再現する。
        let s = extract_all_stats(&desc(26120));
        assert_eq!(
            (s.str_, s.dex, s.vit, s.agi, s.int, s.mnd, s.chr),
            (-30, -30, -30, -30, -30, -30, -30)
        );
    }

    // --- 空入力 -----------------------------------------------------------

    #[test]
    fn double_triple_shot_and_recycle() {
        let s = extract_all_stats(r#""Double Shot"+5% "Triple Shot"+10% "Recycle"+15"#);
        assert_eq!(s.double_shot_pct, 5);
        assert_eq!(s.triple_shot_pct, 10);
        assert_eq!(s.recycle, 15);
    }

    #[test]
    fn prime_weapon_unquoted_magic_attack_bonus() {
        // テロパノセイバー (22188): プライム武器は引用符なしの
        // "Magic Attack Bonus +52" 表記 (通常装備は `"Magic Atk. Bonus"+52`)
        let s = extract_all_stats(
            "DMG:157 Delay:233 Accuracy+32 Magic Accuracy+32 Magic Attack Bonus +52\nMagic damage +250 Sword skill +255\nParrying skill +255 Magic Accuracy skill +255\nHaste+5%\nPhysical damage limit +5%\nDamage taken -5%",
        );
        assert_eq!(s.magic_attack, 52);
        assert_eq!(s.magic_damage, 250);
        assert_eq!(s.magic_accuracy, 32);
        // 属性別魔攻の除外は引用符なし形でも維持される
        let s = extract_all_stats(r#"Dark Elemental "Magic Atk. Bonus"+28"#);
        assert_eq!(s.magic_attack, 0);
    }

    #[test]
    fn magic_burst_damage_and_ii() {
        // 実データは "Magic burst damage +10" / "Magic Burst damage +7" /
        // "Magic burst damage II +5" の表記ゆれがある
        let s = extract_all_stats("Magic burst damage +10\nMagic burst damage II +5");
        assert_eq!(s.magic_burst_damage, 10);
        assert_eq!(s.magic_burst_damage_2, 5);
        // 数値なしの旧表記は対象外
        let s = extract_all_stats("Bonus damage added to magic burst");
        assert_eq!(s.magic_burst_damage, 0);
        assert_eq!(s.magic_burst_damage_2, 0);
    }

    #[test]
    fn magic_burst_accuracy_is_not_accuracy() {
        // "Magic burst accuracy+20" (Peda. M.Board +2 等) は MB命中であり、
        // 命中にも MB ダメージにも合算しない
        let s = extract_all_stats("Magic burst accuracy+20");
        assert_eq!(s.accuracy, 0);
        assert_eq!(s.magic_accuracy, 0);
        assert_eq!(s.magic_burst_damage, 0);
    }

    #[test]
    fn conserve_mp_and_recast_delays() {
        let s = extract_all_stats(
            "\"Conserve MP\"+6 Elemental magic recast delay -15%\nBlue magic recast delay -16%\nSong recast delay -3 Ninjutsu recast delay -1",
        );
        assert_eq!(s.conserve_mp, 6);
        assert_eq!(s.elemental_recast_delay_pct, -15);
        assert_eq!(s.blue_recast_delay_pct, -16);
        assert_eq!(s.song_recast_delay, -3);
        assert_eq!(s.ninjutsu_recast_delay, -1);
    }

    #[test]
    fn double_triple_shot_damage() {
        // 実データは "damage +11" (空白入り) と "damage+8" の両形がある
        let s = extract_all_stats(r#""Double Shot" damage +11 "Triple Shot" damage+10"#);
        assert_eq!(s.double_shot_damage_pct, 11);
        assert_eq!(s.triple_shot_damage_pct, 10);
        // ダメージ増は発動率 (+N%) には混入しない
        assert_eq!(s.double_shot_pct, 0);
        assert_eq!(s.triple_shot_pct, 0);
    }

    #[test]
    fn double_shot_damage_and_effect_variants_are_excluded() {
        // ダメージ増・効果アップ・数値なしリサイクルは発動率/軽減量の抽出対象外
        let s = extract_all_stats(
            r#""Double Shot" damage+8 Enhances "Triple Shot" effect Adds "Recycle" effect"#,
        );
        assert_eq!(s.double_shot_pct, 0);
        assert_eq!(s.triple_shot_pct, 0);
        assert_eq!(s.recycle, 0);
    }

    #[test]
    fn empty_description_yields_default() {
        assert_eq!(extract_all_stats(""), EquipStats::default());
    }

    // --- 合算 (JS の sumStats 相当) ---------------------------------------

    #[test]
    fn add_sums_every_field() {
        // 全フィールドの加算漏れを検出する。
        // entries() が全項目を列挙していることを利用し、
        // 「1 を足したものを 2 つ足すと全項目が 2 になる」ことで確認する。
        let mut one = EquipStats::default();
        // 各フィールドに 1 を入れる方法が無いので、加算の可換性で代用する:
        // a に b を足した結果が、項目ごとの和になっているかを見る。
        let a = extract_all_stats("HP+1 MP+2 STR+3 DEF:4 Attack+5");
        let b = extract_all_stats("HP+10 MP+20 STR+30 Accuracy+40");
        one.add(&a);
        one.add(&b);
        assert_eq!(one.hp, 11);
        assert_eq!(one.mp, 22);
        assert_eq!(one.str_, 33);
        assert_eq!(one.def, 4, "b 側が 0 の項目も保持される");
        assert_eq!(one.attack, 5);
        assert_eq!(one.accuracy, 40, "a 側が 0 の項目も加算される");
    }

    #[test]
    fn add_covers_every_field_without_omission() {
        // add() の加算漏れを機械的に検出する。
        //
        // 全項目に値が入った EquipStats を作り、自分自身を足す。加算漏れがあると
        // その項目だけ倍にならないので、entries() を突き合わせれば漏れが分かる。
        // 全項目に値を入れるには、各項目を 1 にした状態を作る必要がある。
        // 直接フィールドを触らずに済ませるため、ここでは entries() の値を見て
        // 「倍になっていない項目」を洗い出す方式にする。
        let base = all_fields_nonzero();
        let mut doubled = base.clone();
        doubled.add(&base);

        let missed: Vec<&str> = base
            .entries()
            .iter()
            .zip(doubled.entries().iter())
            .filter(|((k, v), (k2, v2))| {
                debug_assert_eq!(k, k2);
                *v != 0 && *v2 != v * 2
            })
            .map(|((k, _), _)| *k)
            .collect();

        assert!(
            missed.is_empty(),
            "add() で加算されていない項目がある: {missed:?}"
        );
        // 全項目に値が入っていることも確認する (テスト自体が空振りしないように)
        assert!(
            base.entries().iter().all(|(_, v)| *v != 0),
            "テスト用の値が全項目に入っていない"
        );
    }

    /// 全項目が非ゼロの `EquipStats` を作る。`add()` の網羅性検証に使う。
    fn all_fields_nonzero() -> EquipStats {
        let mut s = EquipStats::default();
        let mut n = 1;
        // entries() と同じ順序でフィールドを埋める。項目を増やしたらここも足す。
        macro_rules! fill {
            ($($f:ident),* $(,)?) => {
                $( s.$f = { n += 1; n }; )*
            };
        }
        fill!(
            hp,
            mp,
            str_,
            dex,
            vit,
            agi,
            int,
            mnd,
            chr,
            hp_pct,
            mp_pct,
            def,
            attack,
            accuracy,
            evasion,
            attack_pct,
            ranged_attack,
            ranged_accuracy,
            magic_attack,
            magic_accuracy,
            magic_accuracy_skill,
            magic_evasion,
            magic_damage,
            haste_pct,
            store_tp,
            double_attack_pct,
            triple_attack_pct,
            quad_attack_pct,
            double_attack_damage_pct,
            triple_attack_damage_pct,
            critical_hit_rate_pct,
            critical_hit_damage_pct,
            weapon_skill_damage_pct,
            subtle_blow,
            subtle_blow_2,
            tp_bonus,
            skillchain_bonus,
            physical_damage_limit_pct,
            true_shot,
            magic_critical_hit_2_pct,
            magic_affinity,
            magic_burst_damage,
            magic_burst_damage_2,
            conserve_mp,
            elemental_recast_delay_pct,
            blue_recast_delay_pct,
            song_recast_delay,
            ninjutsu_recast_delay,
            damage_taken_pct,
            physical_damage_taken_pct,
            magic_damage_taken_pct,
            magic_def_bonus,
            dmg,
            delay,
            regen,
            refresh,
            regain,
            fast_cast_pct,
            quick_magic_pct,
            snapshot_pct,
            rapid_shot_pct,
            double_shot_pct,
            triple_shot_pct,
            double_shot_damage_pct,
            triple_shot_damage_pct,
            recycle,
            resist_fire,
            resist_ice,
            resist_wind,
            resist_earth,
            resist_lightning,
            resist_water,
            resist_light,
            resist_dark,
            resist_sleep,
            resist_paralysis,
            resist_bind,
            resist_silence,
            resist_gravity,
            resist_slow,
            resist_petrification,
            resist_stun,
            resist_poison,
            resist_charm,
            resist_blind,
            resist_curse,
            resist_virus,
            resist_amnesia,
            resist_terror,
            resist_death,
        );
        s
    }

    // =====================================================================
    // 全件突き合わせ (移植の完了判定に使ったもの)
    //
    // 移植時に JS 実装との一致を全 items.json に対して検証した。手書きの
    // アサーションより遥かに広い範囲を押さえられるため、移植の完了判定は
    // これで行った (docs/adr/0010 の Confirmation)。全 15,504 件で不一致 0。
    //
    // **移植元の JS 実装と期待値生成ハーネスは削除済みなので、現在この比較は
    // 実行できない。** 期待値 JSON を別途用意すれば動く形で残してある。
    // 抽出ロジックを大きく変えるときに、変更前後の出力を突き合わせる用途で使える。
    //
    // 実行方法:
    //   JS_STATS=<期待値JSON> cargo test --lib conformance -- --nocapture
    //
    // 期待値 JSON の形式: { "<item_id>": { "stats": {...}, "skills": {...} } }
    // 環境変数が無い場合はスキップする。
    // =====================================================================

    #[test]
    fn conformance_with_js_over_all_items() {
        let Ok(path) = std::env::var("JS_STATS") else {
            eprintln!("JS_STATS 未設定のためスキップ");
            return;
        };
        let raw =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} を読めない: {e}"));
        let js: serde_json::Value = serde_json::from_str(&raw).expect("JSON parse failed");
        let js = js.as_object().expect("top level must be an object");

        let mut mismatches: Vec<String> = Vec::new();
        let mut compared = 0usize;

        for (id_str, entry) in js {
            let id: u32 = id_str.parse().expect("id must be numeric");
            let Some(item) = item_by_id(id) else { continue };
            let expected = entry["stats"].as_object().expect("stats must be an object");

            let got = extract_all_stats(item.description_en.as_deref().unwrap_or(""));
            compared += 1;

            for (key, value) in got.entries() {
                let want = expected.get(key).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                if want != value {
                    mismatches.push(format!(
                        "id={id} ({}) {key}: js={want} rust={value}",
                        item.en
                    ));
                }
            }
            // JS 側にしか無いキー (Rust の構造体に未定義) の検出
            for key in expected.keys() {
                if !got.entries().iter().any(|(k, _)| k == key) {
                    mismatches.push(format!("id={id} ({}) {key}: Rust に未実装", item.en));
                }
            }

            // スキルボーナスも同じ入力で突き合わせる
            let want_skills = entry["skills"].as_object().expect("skills must be object");
            let got_skills = extract_skill_bonuses(item.description_en.as_deref().unwrap_or(""));
            for (key, value) in &got_skills {
                let want = want_skills.get(*key).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                if want != *value {
                    mismatches.push(format!(
                        "id={id} ({}) skill/{key}: js={want} rust={value}",
                        item.en
                    ));
                }
            }
            for (key, v) in want_skills {
                let want = v.as_i64().unwrap_or(0) as i32;
                if want != 0 && !got_skills.contains_key(key.as_str()) {
                    mismatches.push(format!(
                        "id={id} ({}) skill/{key}: js={want} rust=(なし)",
                        item.en
                    ));
                }
            }
        }

        eprintln!("比較件数: {compared}, 不一致: {}", mismatches.len());
        for m in mismatches.iter().take(40) {
            eprintln!("  {m}");
        }
        assert!(
            mismatches.is_empty(),
            "JS と {} 件の不一致がある (先頭 40 件を表示)",
            mismatches.len()
        );
    }

    #[test]
    fn extract_stat_handles_colon_and_fullwidth() {
        assert_eq!(extract_stat_from_description("DEF:77", "DEF"), 77);
        assert_eq!(extract_stat_from_description("DMG:+165", "DMG"), 165);
        assert_eq!(extract_stat_from_description("ＳＴＲ＋５", "STR"), 5);
        assert_eq!(extract_stat_from_description("STR-3", "STR"), -3);
        assert_eq!(extract_stat_from_description("防77", "防"), 77);
        assert_eq!(extract_stat_from_description("Attack+10", "attack"), 10);
        assert_eq!(extract_stat_from_description("なにもない", "STR"), 0);
    }

    // プロパティセットのユーザー定義項目 (日本語プロパティ名) が前提とする挙動
    #[test]
    fn extract_stat_handles_japanese_property_names() {
        assert_eq!(extract_stat_from_description("二刀流+5", "二刀流"), 5);
        assert_eq!(extract_stat_from_description("二刀流＋１０", "二刀流"), 10);
        // 説明文の途中にあっても一致する
        assert_eq!(
            extract_stat_from_description("ダブルアタック+3 二刀流+2", "二刀流"),
            2
        );
        // 複数一致は最初の 1 件のみ (合算しない)
        assert_eq!(
            extract_stat_from_description("二刀流+3 何か 二刀流+4", "二刀流"),
            3
        );
        assert_eq!(extract_stat_from_description("ストアTP+5", "二刀流"), 0);
    }

    // 条件ラベル配下 (ペット/潜在能力など) は本体の値ではないので拾わない
    #[test]
    fn extract_stat_skips_conditional_label_scope() {
        // イーガダブレット (11338): ペット行の 命中/モクシャ はどちらもペットのもの
        let iga = "防21 ペット:命中+3 モクシャ+3";
        assert_eq!(extract_stat_from_description(iga, "命中"), 0);
        assert_eq!(extract_stat_from_description(iga, "モクシャ"), 0);
        // ラベルより前は本体の値として拾う (モエパパストーン 10817)
        assert_eq!(
            extract_stat_from_description("防5 CHR+5 ヘイスト+5% ペット:ヘイスト+5%", "ヘイスト"),
            5
        );
        // ペット以外の条件ラベルも同じ扱い
        assert_eq!(
            extract_stat_from_description("防2 潜在能力:命中+50 飛命+50", "命中"),
            0
        );
        assert_eq!(
            extract_stat_from_description("右耳:ダブルアタック+7% モクシャ+5", "モクシャ"),
            0
        );
        // ラベル自身は範囲に含めない (`ペット:命中` のような指定は従来どおり拾える)
        assert_eq!(
            extract_stat_from_description("ペット:命中+10", "ペット:命中"),
            10
        );
    }

    #[test]
    fn conditional_label_scope_ends_at_line_break() {
        // 折り返し行は本体の効果に戻る (ＰＮチュリダル+2 10727 と同じ形)
        // description_ja の改行はリテラルの `\n` (crate::items)
        let literal = r"防42 命中+9 オートマトン:魔命+9\n敵対心-2";
        assert_eq!(extract_stat_from_description(literal, "敵対心"), -2);
        assert_eq!(extract_stat_from_description(literal, "魔命"), 0);
        // 実際の改行でも同じ
        let real = "防42 命中+9 オートマトン:魔命+9\n敵対心-2";
        assert_eq!(extract_stat_from_description(real, "敵対心"), -2);
    }

    #[test]
    fn conditional_label_scope_ignores_ascii_labels() {
        // description_ja が無い装備は description_en で代替するため、
        // 英語の `ステータス:値` 表記を条件ラベルと誤認してはいけない
        assert_eq!(
            extract_stat_from_description("DMG:+165 Delay:+240 Accuracy+20", "Accuracy"),
            20
        );
        assert_eq!(
            extract_stat_from_description("DMG:+165 Delay:+240", "Delay"),
            240
        );
    }
}
