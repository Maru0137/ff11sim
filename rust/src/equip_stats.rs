//! 装備の説明文 (`description_en`) から数値を抽出する。
//!
//! `web/js/equip-stats.js` からの移植 (docs/adr/0010)。
//!
//! # 移植方針: 挙動を変えない
//!
//! JS 実装の挙動をそのまま再現する。既知の誤りも含めて移す。具体的には、
//! 説明文に含まれる条件付きセグメント (`In Dynamis:` / `Unity Ranking:` /
//! `Right ear:` / `Set:` など 30 種以上) を JS は体系的に扱っておらず、
//! Unity Ranking は無条件に加算され、`In Dynamis:` は無視されない。
//! これを直すのは条件セグメント対応として別途扱う。ここで挙動を変えると、
//! `web/test/equip-stats-extraction.test.js` の 71 アサーションが
//! 「移植が正しいか」の判定に使えなくなるため。
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
    /// キー名は `web/js/equip-stats.js` の返すオブジェクトと一致させる。
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
    s.magic_attack = signed(&format!(
        r#"(?<!elemental )"magic atk\.? bonus"{WS}*([+-]){WS}*([0-9]+)"#
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
        r"(?<!ranged )(?<!magic )(?<!skill )(?<![a-z])accuracy{WS}*([+-]){WS}*([0-9]+)"
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
    // 期待値は `node scripts/dump_equip_stats.js` の出力から取っている。
    // 思い込みで書くと移植の検証にならないため、JS をリファレンスとする。
    // =====================================================================

    #[test]
    fn basic_stats_from_real_item() {
        // ヒポメネソックス+1
        // node scripts/dump_equip_stats.js --id 27410
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
        // node --text 'HP+10 HP+20' -> {"hp": 30}
        assert_eq!(extract_all_stats("HP+10 HP+20").hp, 30);
    }

    #[test]
    fn negative_values() {
        // node --text 'STR-3' -> {"str": -3}
        assert_eq!(extract_all_stats("STR-3").str_, -3);
    }

    // --- パーセント表記の分離 (E2) ----------------------------------------

    #[test]
    fn percent_does_not_leak_into_flat_stat() {
        // node --text 'HP+10%' -> {"hp_pct": 10}  (hp は付かない)
        let s = extract_all_stats("HP+10%");
        assert_eq!(s.hp, 0, "パーセント表記が flat hp に混入している");
        assert_eq!(s.hp_pct, 10);
    }

    #[test]
    fn flat_and_percent_coexist() {
        // node --text 'HP+50 HP+10%' -> {"hp": 50, "hp_pct": 10}
        let s = extract_all_stats("HP+50 HP+10%");
        assert_eq!(s.hp, 50);
        assert_eq!(s.hp_pct, 10);
    }

    #[test]
    fn percent_from_real_item() {
        // マタンサハーネス: DEF:77 HP+8% STR+15 DEX+15 VIT+15 ...
        // node scripts/dump_equip_stats.js --id 10255
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
        // node --text 'ALL BP+10'
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
        // node --text 'STR+5 ALL BP+10' -> str 15, 他は 10
        let s = extract_all_stats("STR+5 ALL BP+10");
        assert_eq!(s.str_, 15);
        assert_eq!(s.dex, 10);
    }

    #[test]
    fn all_bp_range_notation_takes_first_match() {
        // ホクスニピアス: "Mastery Rank: All BP -30 to +30"
        // JS は最初の符号付き値 (-30) を採用する。"to +30" は "All BP" が
        // 直前に無いためマッチしない。Unity Ranking のような最大値採用は行わない。
        // node scripts/dump_equip_stats.js --id 26120 -> すべて -30
        // 挙動を変えない方針 (docs/adr/0010) によりそのまま再現する。
        let s = extract_all_stats(&desc(26120));
        assert_eq!(
            (s.str_, s.dex, s.vit, s.agi, s.int, s.mnd, s.chr),
            (-30, -30, -30, -30, -30, -30, -30)
        );
    }

    // --- 空入力 -----------------------------------------------------------

    #[test]
    fn empty_description_yields_default() {
        assert_eq!(extract_all_stats(""), EquipStats::default());
    }

    // =====================================================================
    // 全件突き合わせ
    //
    // JS 実装との一致を全 items.json に対して検証する。手書きのアサーションより
    // 遥かに広い範囲を押さえられるので、移植の完了判定はこれで行う
    // (docs/adr/0010 の Confirmation)。
    //
    // 実行方法:
    //   node scripts/dump_equip_stats.js --all > /tmp/js-stats.json
    //   JS_STATS=/tmp/js-stats.json cargo test --lib conformance -- --nocapture
    //
    // 環境変数が無い場合はスキップする (CI では JS を動かさないため)。
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
}
