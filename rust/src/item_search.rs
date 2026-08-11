//! 装備検索。`web/js/item-search.js` からの移植 (docs/adr/0010)。
//!
//! [ADR 0009](../../docs/adr/0009-embed-item-data-in-binary.md) で装備データを
//! バイナリに埋め込み JS 側の `items.json` 読み込みを廃止すると決めたため、
//! 検索も Rust 側に持つ必要がある。
//!
//! 移植方針は解釈側と同じで挙動を変えない。JS の比較順序や undefined の扱いを
//! そのまま再現する。
//!
//! JS に残すもの: `getFilterableProperties()` と `getOperators()`。
//! どちらも items.json を参照せず、フィルタ UI の `<select>` を組み立てるための
//! 静的なメタデータ (label を含む UI 文言) なので、DOM を作る側に置く方が自然。

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::items::{ITEMS, Item, ItemCategory};

/// 検索用に正規化する。カタカナ→ひらがな、全角英数→半角、小文字化。
pub fn normalize_for_search(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let code = c as u32;
        let mapped = match code {
            // カタカナ (U+30A1-U+30F6) → ひらがな
            0x30A1..=0x30F6 => char::from_u32(code - 0x60).unwrap_or(c),
            // 全角英大文字 → 半角
            0xFF21..=0xFF3A => char::from_u32(code - 0xFF21 + 0x41).unwrap_or(c),
            // 全角英小文字 → 半角
            0xFF41..=0xFF5A => char::from_u32(code - 0xFF41 + 0x61).unwrap_or(c),
            // 全角数字 → 半角
            0xFF10..=0xFF19 => char::from_u32(code - 0xFF10 + 0x30).unwrap_or(c),
            _ => c,
        };
        out.extend(mapped.to_lowercase());
    }
    out
}

/// 検索用の正規化済み名称。`ITEMS` と同じ並び順で保持する。
struct NormalizedNames {
    en: String,
    ja: String,
    enl: String,
    jal: String,
}

static NORMALIZED: LazyLock<Vec<NormalizedNames>> = LazyLock::new(|| {
    ITEMS
        .iter()
        .map(|i| NormalizedNames {
            en: normalize_for_search(&i.en),
            ja: normalize_for_search(&i.ja),
            enl: normalize_for_search(&i.enl),
            jal: normalize_for_search(&i.jal),
        })
        .collect()
});

/// 検索条件 1 件。JS の `{property, operator, value}` に対応。
#[derive(Debug, Clone, Deserialize)]
pub struct Filter {
    pub property: String,
    pub operator: String,
    pub value: String,
}

/// 検索オプション。JS の `search(options)` の引数に対応する。
///
/// JS 側は camelCase (`sortBy` / `ilv119Only` など) で渡すため、
/// 受け取り側で変換する。未指定のキーは `Default` の値になる。
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchOptions {
    pub query: String,
    pub slot: String,
    pub job: String,
    pub filters: Vec<Filter>,
    pub sort_by: String,
    pub desc_stat: String,
    pub sort_order: String,
    pub ilv119_only: bool,
    pub ilv119_slots: Vec<String>,
    pub limit: usize,
    pub offset: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            slot: String::new(),
            job: String::new(),
            filters: Vec::new(),
            sort_by: "id".to_string(),
            desc_stat: String::new(),
            sort_order: "asc".to_string(),
            ilv119_only: false,
            ilv119_slots: Vec::new(),
            limit: 50,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult<'a> {
    pub items: Vec<&'a Item>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

/// 説明文からステータス値を取り出す。ソート用。
/// 全角英数と `＋` `－` `―` を半角に直してから、最初に一致した値を返す。
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
    let mut i = 0usize;
    while i + pat.len() <= bytes.len() {
        if bytes[i..i + pat.len()] != pat[..] {
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

/// 装備名末尾の `+N` を取り出す。HQ 品を上位に並べるために使う。
pub fn extract_plus_value(name: &str) -> i32 {
    let chars: Vec<char> = name.chars().collect();
    let mut end = chars.len();
    let mut start = end;
    while start > 0 && chars[start - 1].is_ascii_digit() {
        start -= 1;
    }
    if start == end || start == 0 || chars[start - 1] != '+' {
        return 0;
    }
    if end > chars.len() {
        end = chars.len();
    }
    chars[start..end]
        .iter()
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

fn category_str(c: ItemCategory) -> &'static str {
    match c {
        ItemCategory::Weapon => "Weapon",
        ItemCategory::Armor => "Armor",
    }
}

/// フィルタ 1 件を評価する。JS の `applyFilter` に対応。
fn apply_filter(item: &Item, f: &Filter) -> bool {
    let op = f.operator.as_str();

    // 配列型 (jobs / slots / races)
    let array_value: Option<&Vec<String>> = match f.property.as_str() {
        "jobs" => Some(&item.jobs),
        "slots" => Some(&item.slots),
        "races" => Some(&item.races),
        _ => None,
    };
    if let Some(values) = array_value {
        let needle = f.value.to_uppercase();
        let found = values.iter().any(|v| v.to_uppercase().contains(&needle));
        return match op {
            "contains" => found,
            "not_contains" => !found,
            _ => false,
        };
    }

    // 数値型。JS は undefined/null を `!=` と `not_contains` でのみ true にする。
    let number_value: Option<Option<i32>> = match f.property.as_str() {
        "id" => Some(Some(item.id as i32)),
        "level" => Some(Some(item.level)),
        "item_level" => Some(item.item_level),
        "damage" => Some(item.damage),
        "delay" => Some(item.delay),
        "skill" => Some(item.skill),
        _ => None,
    };
    if let Some(v) = number_value {
        let Some(v) = v else {
            return op == "!=" || op == "not_contains";
        };
        let Ok(n) = f.value.trim().parse::<f64>() else {
            return false;
        };
        let v = v as f64;
        return match op {
            "=" => v == n,
            "!=" => v != n,
            ">=" => v >= n,
            "<=" => v <= n,
            ">" => v > n,
            "<" => v < n,
            _ => false,
        };
    }

    // 文字列型
    let string_value: Option<&str> = match f.property.as_str() {
        "en" => Some(&item.en),
        "ja" => Some(&item.ja),
        "category" => Some(category_str(item.category)),
        _ => None,
    };
    let Some(v) = string_value else {
        // JS では未知プロパティは undefined になる
        return op == "!=" || op == "not_contains";
    };
    let v = v.to_lowercase();
    let needle = f.value.to_lowercase();
    match op {
        "contains" => v.contains(&needle),
        "=" => v == needle,
        "!=" => v != needle,
        "starts" => v.starts_with(&needle),
        "ends" => v.ends_with(&needle),
        _ => false,
    }
}

const PRIORITY_SLOTS: [&str; 7] = ["main", "sub", "head", "body", "hands", "legs", "feet"];

/// 装備を検索する。JS の `search(options)` に対応。
pub fn search(opts: &SearchOptions) -> SearchResult<'static> {
    let mut results: Vec<(usize, &'static Item)> = ITEMS.iter().enumerate().collect();

    // 名称検索 (正規化して部分一致)
    if !opts.query.is_empty() {
        let q = normalize_for_search(&opts.query);
        results.retain(|(i, _)| {
            let n = &NORMALIZED[*i];
            n.en.contains(&q) || n.ja.contains(&q) || n.enl.contains(&q) || n.jal.contains(&q)
        });
    }

    // スロット。ear1/ring1 はそれぞれ ear2/ring2 も拾う。
    if !opts.slot.is_empty() {
        results.retain(|(_, item)| {
            item.slots.iter().any(|s| {
                s == &opts.slot
                    || (opts.slot == "ear1" && s == "ear2")
                    || (opts.slot == "ring1" && s == "ring2")
            })
        });
    }

    if !opts.job.is_empty() {
        results.retain(|(_, item)| item.jobs.contains(&opts.job));
    }

    // iLv119 フィルタ
    if opts.ilv119_only && !opts.ilv119_slots.is_empty() {
        if !opts.slot.is_empty() {
            results.retain(|(_, item)| item.item_level == Some(119));
        } else {
            results.retain(|(_, item)| {
                if item.slots.is_empty() {
                    return true;
                }
                let has = item.slots.iter().any(|s| opts.ilv119_slots.contains(s));
                if has {
                    item.item_level == Some(119)
                } else {
                    true
                }
            });
        }
    }

    for f in &opts.filters {
        if f.property.is_empty() || f.operator.is_empty() || f.value.is_empty() {
            continue;
        }
        results.retain(|(_, item)| apply_filter(item, f));
    }

    let total = results.len();

    // 並べ替え。JS の比較関数と同じ優先順位で行う。
    let asc = opts.sort_order != "desc";
    let priority_slot = PRIORITY_SLOTS.contains(&opts.slot.as_str());
    results.sort_by(|(_, a), (_, b)| {
        use std::cmp::Ordering;

        // 優先 1: 名前末尾の +N (降順)
        let ap = extract_plus_value(if a.ja.is_empty() { &a.en } else { &a.ja });
        let bp = extract_plus_value(if b.ja.is_empty() { &b.en } else { &b.ja });
        if ap != bp {
            return bp.cmp(&ap);
        }

        // 優先 2: 特定スロット選択時は iLv119 を先に
        if priority_slot {
            let a119 = a.item_level == Some(119);
            let b119 = b.item_level == Some(119);
            if a119 != b119 {
                return if b119 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
        }

        // 優先 3: 通常のソート
        if opts.sort_by == "desc_stat" && !opts.desc_stat.is_empty() {
            let av = extract_stat_from_description(
                a.description_ja
                    .as_deref()
                    .or(a.description_en.as_deref())
                    .unwrap_or(""),
                &opts.desc_stat,
            );
            let bv = extract_stat_from_description(
                b.description_ja
                    .as_deref()
                    .or(b.description_en.as_deref())
                    .unwrap_or(""),
                &opts.desc_stat,
            );
            return if asc { av.cmp(&bv) } else { bv.cmp(&av) };
        }

        // 文字列プロパティは JS の localeCompare に相当する比較を行う。
        let str_of = |it: &Item| -> Option<String> {
            match opts.sort_by.as_str() {
                "en" => Some(it.en.clone()),
                "ja" => Some(it.ja.clone()),
                "category" => Some(category_str(it.category).to_string()),
                _ => None,
            }
        };
        if let (Some(x), Some(y)) = (str_of(a), str_of(b)) {
            return if asc { x.cmp(&y) } else { y.cmp(&x) };
        }

        // 数値プロパティ。JS は undefined を asc で +Infinity、desc で -Infinity として扱う。
        let num_of = |it: &Item| -> Option<i32> {
            match opts.sort_by.as_str() {
                "id" => Some(it.id as i32),
                "level" => Some(it.level),
                "item_level" => it.item_level,
                "damage" => it.damage,
                "delay" => it.delay,
                "skill" => it.skill,
                _ => None,
            }
        };
        let sentinel = if asc { i32::MAX } else { i32::MIN };
        let av = num_of(a).unwrap_or(sentinel);
        let bv = num_of(b).unwrap_or(sentinel);
        if asc { av.cmp(&bv) } else { bv.cmp(&av) }
    });

    let items: Vec<&'static Item> = results
        .into_iter()
        .skip(opts.offset)
        .take(opts.limit)
        .map(|(_, it)| it)
        .collect();

    SearchResult {
        has_more: opts.offset + opts.limit < total,
        items,
        total,
        offset: opts.offset,
        limit: opts.limit,
    }
}

/// 出現するカテゴリの一覧 (ソート済み)。
pub fn get_categories() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = ITEMS
        .iter()
        .map(|i| category_str(i.category))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    v.sort_unstable();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_converts_katakana_to_hiragana() {
        assert_eq!(normalize_for_search("アビス"), "あびす");
    }

    #[test]
    fn normalize_converts_fullwidth_to_halfwidth() {
        assert_eq!(normalize_for_search("ＡＢＣ１２３"), "abc123");
    }

    #[test]
    fn normalize_lowercases_ascii() {
        assert_eq!(normalize_for_search("Sword"), "sword");
    }

    #[test]
    fn extract_plus_value_reads_trailing_suffix() {
        assert_eq!(extract_plus_value("エミネンスコロネット+2"), 2);
        assert_eq!(extract_plus_value("Hippo. Socks +1"), 1);
        assert_eq!(extract_plus_value("Sword"), 0);
        // 末尾でない +N は対象外
        assert_eq!(extract_plus_value("+2 の何か"), 0);
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

    #[test]
    fn search_by_query_finds_known_item() {
        let opts = SearchOptions {
            query: "ヒポメネソックス".to_string(),
            limit: 10,
            ..Default::default()
        };
        let r = search(&opts);
        assert!(r.total > 0, "検索結果が空");
        assert!(r.items.iter().any(|i| i.id == 27410));
    }

    #[test]
    fn search_filters_by_slot_and_job() {
        let opts = SearchOptions {
            slot: "main".to_string(),
            job: "WAR".to_string(),
            limit: 20,
            ..Default::default()
        };
        let r = search(&opts);
        assert!(r.total > 0);
        for it in &r.items {
            assert!(it.slots.contains(&"main".to_string()));
            assert!(it.jobs.contains(&"WAR".to_string()));
        }
    }

    #[test]
    fn search_ear1_also_matches_ear2() {
        let opts = SearchOptions {
            slot: "ear1".to_string(),
            limit: 5,
            ..Default::default()
        };
        let r = search(&opts);
        assert!(r.total > 0);
        assert!(
            r.items
                .iter()
                .all(|i| i.slots.iter().any(|s| s == "ear1" || s == "ear2"))
        );
    }

    #[test]
    fn search_pagination() {
        let base = SearchOptions {
            slot: "body".to_string(),
            limit: 10,
            ..Default::default()
        };
        let first = search(&base);
        let second = search(&SearchOptions {
            offset: 10,
            ..base.clone()
        });
        assert_eq!(first.total, second.total);
        assert_eq!(first.items.len(), 10);
        assert_ne!(first.items[0].id, second.items[0].id);
        assert_eq!(first.has_more, first.total > 10);
    }

    #[test]
    fn categories_are_weapon_and_armor() {
        assert_eq!(get_categories(), vec!["Armor", "Weapon"]);
    }
}
