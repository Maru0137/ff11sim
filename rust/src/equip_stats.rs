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

use std::sync::LazyLock;

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

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|e| panic!("invalid pattern {pattern:?}: {e}"))
}

// ---------------------------------------------------------------------------
// 前処理 (大小を区別する。JS 側も `/g` で i フラグを付けていない)
// ---------------------------------------------------------------------------

static RE_UNITY: LazyLock<Regex> = LazyLock::new(|| {
    // "Unity Ranking: Attack+10～15" → " Attack+15" (最大値を採用)
    re(&format!(
        r"Unity{WS}+Ranking:{WS}*([A-Za-z][0-9A-Za-z_{ws}]*?){WS}*[+-]{WS}*[0-9]+{WS}*[～~]{WS}*([0-9]+)",
        ws = r" \t\n\r"
    ))
});

static RE_PET: LazyLock<Regex> = LazyLock::new(|| re(r"(?:Pet|Avatar|Wyvern|Automaton):[^:]*"));

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

    text = RE_UNITY.replace_all(&text, " $1+$2").into_owned();
    text = RE_PET.replace_all(&text, "").into_owned();

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
}

impl EquipStats {
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
}

// ---------------------------------------------------------------------------
// 抽出ヘルパー
// ---------------------------------------------------------------------------

/// 符号付きの値を **全出現ぶん合算** する。JS の `matchSigned` に対応。
fn match_signed(text: &str, pattern: &str) -> i32 {
    let re = re(pattern);
    let mut total = 0;
    for caps in re.captures_iter(text).flatten() {
        let sign = if &caps[1] == "-" { -1 } else { 1 };
        total += sign * caps[2].parse::<i32>().unwrap_or(0);
    }
    total
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
    };

    // ALL BP: 7 つの基本パラメータすべてに加算する。
    // JS 同様、個別指定を設定した「後」に加算するため合算になる。
    let all_bp = match_signed(&text, &format!(r"all{WS}*bp{WS}*([+-]){WS}*([0-9]+)"));
    if all_bp != 0 {
        s.add_all_base_params(all_bp);
    }

    s
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
}
