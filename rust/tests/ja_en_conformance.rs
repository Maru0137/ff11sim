//! 日本語説明文からの抽出が、英語説明文からの抽出と全件一致することを検証する
//! (docs/adr/0019 手順 1)。
//!
//! 装備解釈の入力を `description_ja` に一本化するにあたり、正規化層
//! (`convert_augment_ja_to_en`) の語彙が足りているかを機械的に確かめるためのもの。
//! JS リファレンス削除後 (docs/adr/0018) に失われていた全件検証の後継でもある。
//!
//! 差分の内訳を見たいときは `cargo test --release --test ja_en_conformance -- --nocapture`。

use std::collections::BTreeMap;

use ff11sim::equip_stats::{
    convert_augment_ja_to_en, extract_all_stats, extract_skill_bonuses, strip_conditional_labels,
};
use ff11sim::items::ITEMS;

/// 現時点で差が残っている装備の数。減る分には問題ないので上限として見る。
///
/// 差は 4 種類で、いずれも正規化層の語彙では埋められない (docs/adr/0019):
///
/// - **条件セグメントの扱いの差** (最多): 日本語側は `ペット:` `潜在能力:`
///   `ユニティランク:` などの配下を読まないが、英語側は無条件に加算する。
///   これは手順 3 で入れた**意図した差**で、日本語側が正しい。
///   `コンビネーション:` だけは両方が読む。
/// - **英語側の癖**: `Breath Damage taken` を汎用の被ダメージとして数える、
///   `Dark Elemental Magic Attack` の `Attack` を攻撃力として数える、
///   `"Spirit Jump" attack +17%` を攻撃力% として数える、など。日本語側は数えない。
/// - **英語側の表記ゆれ・取りこぼし**: `DMG250` (コロン無し)、`Haste10%`、
///   `Mag. Def. Bonus` (略記の変種)、`Res. all ele.` など。
/// - **データ差**: 日本語と英語で数値自体が違う装備 (例: エニフツッケット 防46 / DEF:49)。
///
/// **完全一致は目標にしない。** どの種類も英語に合わせて再現するのは誤りの移植になる。
/// このテストは「分類済みより差が増えていないこと」だけを見る。
const MAX_KNOWN_DIFFS: usize = 1467;

/// 日本語と英語で内容自体が違う装備。個別に除外したくなったときに使う。
/// (id, 理由)
const KNOWN_DATA_DIFFS: &[(u32, &str)] = &[];

#[test]
fn ja_extraction_matches_en_over_all_items() {
    let known: BTreeMap<u32, &str> = KNOWN_DATA_DIFFS.iter().copied().collect();

    let mut per_key: BTreeMap<&str, usize> = BTreeMap::new();
    let mut samples: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut items_with_diff = 0usize;
    let mut compared = 0usize;

    for item in ITEMS.iter() {
        let (Some(en), Some(ja)) = (
            item.description_en.as_deref(),
            item.description_ja.as_deref(),
        ) else {
            continue;
        };
        if en.is_empty() || ja.is_empty() || known.contains_key(&item.id) {
            continue;
        }
        compared += 1;

        // 実際のパイプラインと同じ順序: 条件ラベル除去 → 正規化 → 抽出 (docs/adr/0019)
        let normalized = convert_augment_ja_to_en(&strip_conditional_labels(ja));
        let from_en = extract_all_stats(en);
        let from_ja = extract_all_stats(&normalized);
        let mut differs = false;

        for ((key, a), (_, b)) in from_en.entries().iter().zip(from_ja.entries().iter()) {
            if a == b {
                continue;
            }
            differs = true;
            *per_key.entry(key).or_insert(0) += 1;
            let bucket = samples.entry(key).or_default();
            if bucket.len() < 3 {
                bucket.push(format!(
                    "id={} {} ({key}: en={a} ja={b})\n     JA: {ja}\n     EN: {en}",
                    item.id, item.ja
                ));
            }
        }

        let skills_en = extract_skill_bonuses(en);
        let skills_ja = extract_skill_bonuses(&normalized);
        if skills_en != skills_ja {
            differs = true;
            *per_key.entry("(skills)").or_insert(0) += 1;
            let bucket = samples.entry("(skills)").or_default();
            if bucket.len() < 3 {
                bucket.push(format!(
                    "id={} {} (skills: en={skills_en:?} ja={skills_ja:?})\n     JA: {ja}",
                    item.id, item.ja
                ));
            }
        }
        if differs {
            items_with_diff += 1;
        }
    }

    if items_with_diff > 0 {
        let mut ranked: Vec<_> = per_key.iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(a.1));
        println!("比較 {compared} 件 / 差のある装備 {items_with_diff} 件");
        for (key, n) in ranked.iter().take(25) {
            println!("\n== {key}: {n} 件");
            for s in samples.get(*key).into_iter().flatten() {
                println!("  {s}");
            }
        }
    }
    assert!(
        items_with_diff <= MAX_KNOWN_DIFFS,
        "分類済みより多くの差がある ({items_with_diff} 件 > {MAX_KNOWN_DIFFS} 件)。\
         --nocapture で内訳を出せる"
    );
}
