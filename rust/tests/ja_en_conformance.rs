//! 日本語説明文からの抽出を、英語説明文からの抽出と全件突き合わせる
//! (docs/adr/0019)。
//!
//! 装備解釈の入力を `description_ja` に一本化したので、正規化層
//! (`convert_augment_ja_to_en`) の語彙が足りているかを機械的に確かめる。
//! JS リファレンス削除後 (docs/adr/0018) に失われていた全件検証の後継でもある。
//!
//! **完全一致は目標にしない。** 差は次の 4 種類で、どれも英語に合わせて再現するのは
//! 誤りの移植になる:
//!
//! - **条件セグメントの扱いの差**: 日本語側は `ペット:` `潜在能力:` `ユニティランク:`
//!   などの配下を読まないが、英語側は無条件に加算する。意図した差で、日本語側が正しい。
//!   `コンビネーション:` だけは両方が読む
//! - **英語側の癖**: `Breath Damage taken` を汎用の被ダメージとして数える、
//!   `Dark Elemental Magic Attack` の `Attack` を攻撃力として数える、など
//! - **英語側の表記ゆれ・取りこぼし**: `DMG250` (コロン無し)、`Haste10%`、
//!   `Mag. Def. Bonus` (略記の変種)、`Res. all ele.` など
//! - **データ差**: 日本語と英語で数値自体が違う装備 (例: エニフツッケット 防46 / DEF:49)
//!
//! そこで件数ではなく**分類できるかどうか**を見る。件数を上限にすると、CI が上流から
//! 再生成する items.json の中身が変わるたびに落ちる (実際に手元 15,452 件 /
//! CI 15,473 件で 2 件ずれて落ちた)。
//!
//! 内訳を見たいときは `cargo test --release --test ja_en_conformance -- --nocapture`。

use std::collections::BTreeMap;

use ff11sim::equip_stats::{
    convert_augment_ja_to_en, extract_all_stats, extract_skill_bonuses, strip_conditional_labels,
};
use ff11sim::items::ITEMS;

/// 英語側の癖・表記ゆれを示す目印 (小文字で照合する)。英語説明文にこれがあれば、
/// 差はそれで説明できる。日本語側が正しいので、一致させずに差のまま残す。
const EN_QUIRK_MARKERS: &[(&str, &str)] = &[
    (
        "breath damage taken",
        "ブレスダメージを汎用の被ダメージとして数える",
    ),
    (
        "synergy damage taken",
        "錬成窯ダメージを汎用の被ダメージとして数える",
    ),
    (
        "elemental",
        "属性限定の魔攻/魔命を汎用のものとして数える (改行を挟むと除外が効かない)",
    ),
    (
        "jump\" attack",
        "ジャンプ限定の攻撃力を汎用の攻撃力% として数える",
    ),
    ("converted to mp", "被ダメージの MP 変換を MP として数える"),
    ("based on amount of", "追撃+1 の 1 を攻撃力として数える"),
    ("mag. def. bonus", "魔防の略記の変種を展開できていない"),
    ("m. def. b.", "同上"),
    ("magic defense", "魔防を別項目として書いている"),
    ("res. all ele.", "全耐性の表記ゆれを拾えていない"),
    ("all resistances", "同上"),
    ("all ailment resistance", "同上 (状態異常)"),
    ("subtle blow+", "引用符なしのモクシャを拾えていない"),
    (
        "magic critical hit",
        "魔法クリティカルの表記ゆれ (rate の有無)",
    ),
    (
        "unity ranking",
        "ユニティランクの大小が揃っておらず範囲最大値を取れない",
    ),
    ("attr.+", "ALLBP の表記が揃っていない"),
    ("recovered while healing", "ヒーリング HP/MP の表記ゆれ"),
    ("shadow images", "空蝉枚数に応じた値を条件なしで数える"),
    ("damage taken +", "被ダメージの符号・空白の揺れ"),
    ("instruments skill", "楽器スキルが複数形で書かれている"),
    (
        "\" damage +",
        "ダブル/トリプルアタックダメージの表記を拾えていない",
    ),
];

/// 上の分類に当てはまらない差の上限。2026-08-15 時点の実測は 94 件で、
/// 中身は**日本語と英語でデータそのものが違う装備**:
/// エニフツッケット (防46 / DEF:49)、メアナーケープ (命中+7 / Ranged Accuracy+7)、
/// スルマスタッフ (魔命スキル+84 / +85) など。どちらが正しいかはゲーム側の話で、
/// 規則では埋められない。
///
/// 0 にできないのでゼロ件は求めず、実測 + 余裕で上限を置く。上流の items.json は
/// CI が再生成するため件数が数件動く (実際に手元 15,452 件 / CI 15,473 件で
/// 総数が 2 件ずれた)。
///
/// **これが増えたら**、正規化層の語彙が抜けたか、英語側の新しい癖が現れたか、
/// データ差が増えたかのいずれか。`--nocapture` で内訳を見て、語彙なら
/// `AUGMENT_JA_TO_EN` に足し、癖なら `EN_QUIRK_MARKERS` に理由付きで足す。
const MAX_UNEXPLAINED: usize = 150;

/// `DMG:` / `Delay:` のコロンが無い英語表記 (`DMG250` / `DMG+1`) を検出する。
fn has_colonless_dmg(en: &str) -> bool {
    ["DMG", "Delay"].iter().any(|k| {
        en.match_indices(k)
            .any(|(i, _)| !en[i + k.len()..].starts_with(':'))
    })
}

/// `Haste10%` / `Haste+5'` のような英語側の打ち間違いを検出する。
fn has_broken_haste(en: &str) -> bool {
    en.match_indices("Haste").any(|(i, _)| {
        let rest = &en[i + "Haste".len()..];
        rest.starts_with(|c: char| c.is_ascii_digit()) || rest.contains('\'')
    })
}

#[test]
fn ja_en_diffs_are_all_explainable() {
    let mut per_key: BTreeMap<&str, usize> = BTreeMap::new();
    let mut samples: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut compared = 0usize;
    let mut by_condition = 0usize;
    let mut by_quirk = 0usize;
    let mut unexplained: Vec<String> = Vec::new();

    for item in ITEMS.iter() {
        let (Some(en), Some(ja)) = (
            item.description_en.as_deref(),
            item.description_ja.as_deref(),
        ) else {
            continue;
        };
        if en.is_empty() || ja.is_empty() {
            continue;
        }
        compared += 1;

        // 実際のパイプラインと同じ順序: 条件ラベル除去 → 正規化 → 抽出 (docs/adr/0019)
        let stripped = strip_conditional_labels(ja);
        let normalized = convert_augment_ja_to_en(&stripped);
        let from_en = extract_all_stats(en);
        let from_ja = extract_all_stats(&normalized);

        let mut diff_keys: Vec<&str> = Vec::new();
        for ((key, a), (_, b)) in from_en.entries().iter().zip(from_ja.entries().iter()) {
            if a != b {
                diff_keys.push(key);
                *per_key.entry(key).or_insert(0) += 1;
                let bucket = samples.entry(key).or_default();
                if bucket.len() < 3 {
                    bucket.push(format!(
                        "id={} {} ({key}: en={a} ja={b})\n     JA: {ja}\n     EN: {en}",
                        item.id, item.ja
                    ));
                }
            }
        }
        if extract_skill_bonuses(en) != extract_skill_bonuses(&normalized) {
            diff_keys.push("(skills)");
            *per_key.entry("(skills)").or_insert(0) += 1;
        }
        if diff_keys.is_empty() {
            continue;
        }

        let en_lower = en.to_lowercase();
        let has_quirk = EN_QUIRK_MARKERS
            .iter()
            .any(|(marker, _)| en_lower.contains(marker))
            || has_colonless_dmg(en)
            || has_broken_haste(en);

        // 条件ラベルが 1 つでもあれば、その配下を読まない差として説明できる
        if stripped != ja {
            by_condition += 1;
        } else if has_quirk {
            by_quirk += 1;
        } else {
            unexplained.push(format!(
                "id={} {} {diff_keys:?}\n     JA: {ja}\n     EN: {en}",
                item.id, item.ja
            ));
        }
    }

    let total = by_condition + by_quirk + unexplained.len();
    println!(
        "比較 {compared} 件 / 差 {total} 件 (条件セグメント {by_condition} / 英語側の癖 {by_quirk} / 未分類 {})",
        unexplained.len()
    );
    let mut ranked: Vec<_> = per_key.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1));
    for (key, n) in ranked.iter().take(15) {
        println!("\n== {key}: {n} 件");
        for s in samples.get(*key).into_iter().flatten() {
            println!("  {s}");
        }
    }

    assert!(
        unexplained.len() <= MAX_UNEXPLAINED,
        "説明の付かない差が {} 件ある (上限 {MAX_UNEXPLAINED} 件):\n{}",
        unexplained.len(),
        unexplained
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
