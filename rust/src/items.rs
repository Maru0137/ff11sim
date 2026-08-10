//! 装備データ (`items.json` / `augments.json`) をバイナリに埋め込み、参照可能にする。
//!
//! 設計判断は docs/adr/0009 を参照。要点:
//! - `items.json` は上流から生成する派生物で git 管理外だが、`include_str!` は
//!   コンパイル時に存在すればよい。`scripts/build_web_data.sh` を `cargo build` より
//!   前に走らせることが前提になる。
//! - gzip 後の転送量は配信物として別に配る場合とほぼ変わらない (差 1KB 未満) ため、
//!   Rust 側にデータを寄せても利用者の負担は増えない。
//!
//! `items.json` は `build/` に生成する。`web/` の外に置くのは、ブラウザが
//! これを読まなくなったため (docs/adr/0010 手順 5 完了)。`web/` に置いたままだと
//! Pages に配信され、WASM に埋め込んだものと合わせて同じデータを二重に配ることになる。

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// items.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
}

/// 装備 1 件。`scripts/parse_lua_to_json.py` が生成する形に対応する。
///
/// `jobs` / `slots` / `races` を enum にしていないのは、いずれも既存の enum と
/// 綴りや粒度が一致しないため。`jobs` は `"WAR"` 形式で `Job::War` と綴りが違い、
/// `races` は `"Hum_M"` / `"Hum_F"` と性別まで分かれていて `Race` (5 種) に写せない。
/// 型付けは解釈の移植 (docs/adr/0010) で扱う。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub id: u32,
    /// 英語名
    pub en: String,
    /// 日本語名
    pub ja: String,
    /// 英語名 (小文字化済み)
    pub enl: String,
    /// 日本語名 (小文字化済み)
    pub jal: String,
    pub category: ItemCategory,
    #[serde(rename = "type")]
    pub item_type: i32,
    pub flags: i32,
    pub stack: i32,
    /// 装備可能レベル
    pub level: i32,
    /// 装備可能ジョブ (`"WAR"` 形式)
    pub jobs: Vec<String>,
    /// 装備可能スロット (`"body"` 形式)
    pub slots: Vec<String>,
    /// 装備可能種族 (`"Hum_M"` 形式、性別まで分かれる)
    pub races: Vec<String>,
    /// 説明文。全体の 1% 弱が欠落している。
    /// 改行はリテラルの `\n` (バックスラッシュ + n) として入っている点に注意。
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub description_ja: Option<String>,
    /// 以下は武器のみ (全体の約 1/3)
    #[serde(default)]
    pub damage: Option<i32>,
    #[serde(default)]
    pub delay: Option<i32>,
    /// 武器スキル ID。`skills::weapon_skill_from_item_id` で `SkillKind` に変換できる。
    #[serde(default)]
    pub skill: Option<i32>,
    #[serde(default)]
    pub item_level: Option<i32>,
    /// 盾のみ
    #[serde(default)]
    pub shield_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ItemsFile {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    item_count: usize,
    items: Vec<Item>,
}

pub static ITEMS: LazyLock<Vec<Item>> = LazyLock::new(|| {
    serde_json::from_str::<ItemsFile>(include_str!("../../build/items.json"))
        .expect("items.json parse failed")
        .items
});

static ITEMS_BY_ID: LazyLock<HashMap<u32, &'static Item>> =
    LazyLock::new(|| ITEMS.iter().map(|item| (item.id, item)).collect());

/// アイテム ID で引く。
pub fn item_by_id(id: u32) -> Option<&'static Item> {
    ITEMS_BY_ID.get(&id).copied()
}

// ---------------------------------------------------------------------------
// augments.json
// ---------------------------------------------------------------------------

/// オーグメントのランク 1 段階。`text` は日本語表記のまま保持する
/// (解釈は docs/adr/0010 の移植対象)。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AugmentRank {
    pub rank: i32,
    pub text: String,
}

/// オーグメントの経路。`path_type` は `"Default"` / `"Type:A"` など。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AugmentPath {
    #[serde(rename = "type")]
    pub path_type: String,
    pub ranks: Vec<AugmentRank>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemAugments {
    pub paths: Vec<AugmentPath>,
}

#[derive(Debug, Deserialize)]
struct AugmentsFile {
    #[allow(dead_code)]
    version: u32,
    augments: HashMap<u32, ItemAugments>,
}

/// `augments.json` はリポジトリ管理下なので、`items.json` と違いビルド順の制約を受けない。
pub static AUGMENTS: LazyLock<HashMap<u32, ItemAugments>> = LazyLock::new(|| {
    serde_json::from_str::<AugmentsFile>(include_str!("../../web/data/augments.json"))
        .expect("augments.json parse failed")
        .augments
});

/// アイテム ID に対応するオーグメント定義を引く。定義が無い装備は `None`。
pub fn augments_by_item_id(id: u32) -> Option<&'static ItemAugments> {
    AUGMENTS.get(&id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_are_loaded() {
        // 上流の更新で増減するため、具体的な件数ではなく下限だけを見る。
        assert!(ITEMS.len() > 14000, "item count: {}", ITEMS.len());
    }

    #[test]
    fn item_by_id_resolves_known_weapon() {
        // ニビルナイフ。wasm.rs の回避値テストが手書きしている装備の 1 つ。
        let item = item_by_id(20600).expect("id 20600 not found");
        assert_eq!(item.category, ItemCategory::Weapon);
        assert!(item.damage.is_some(), "weapon must have damage");
        assert!(item.delay.is_some(), "weapon must have delay");
        assert!(item.skill.is_some(), "weapon must have skill id");
        assert!(item.slots.contains(&"main".to_string()));
    }

    #[test]
    fn item_by_id_resolves_known_armor() {
        // 無の面。
        let item = item_by_id(24270).expect("id 24270 not found");
        assert_eq!(item.category, ItemCategory::Armor);
        assert!(item.damage.is_none(), "armor must not have damage");
        assert!(item.description_en.is_some());
    }

    #[test]
    fn item_by_id_returns_none_for_unknown() {
        assert!(item_by_id(0).is_none());
    }

    #[test]
    fn weapons_always_have_combat_fields() {
        for item in ITEMS.iter().filter(|i| i.category == ItemCategory::Weapon) {
            assert!(
                item.damage.is_some() && item.delay.is_some() && item.skill.is_some(),
                "weapon {} (id {}) lacks combat fields",
                item.en,
                item.id
            );
        }
    }

    #[test]
    fn augments_are_loaded() {
        assert!(!AUGMENTS.is_empty());
    }

    #[test]
    fn augments_by_item_id_resolves_known_item() {
        // アスプロピアス。Default 経路のランク別テキストを持つ。
        let aug = augments_by_item_id(23755).expect("id 23755 has no augments");
        let default_path = aug
            .paths
            .iter()
            .find(|p| p.path_type == "Default")
            .expect("no Default path");
        assert!(!default_path.ranks.is_empty());
        // ランクは昇順で、text は空でない。
        let ranks: Vec<i32> = default_path.ranks.iter().map(|r| r.rank).collect();
        let mut sorted = ranks.clone();
        sorted.sort_unstable();
        assert_eq!(ranks, sorted, "ranks must be ascending");
        assert!(default_path.ranks.iter().all(|r| !r.text.is_empty()));
    }

    #[test]
    fn augment_target_items_exist_in_items() {
        // オーグメント定義があるのに装備データに存在しない ID は、
        // augments.json が古い可能性を示す (docs/adr/0004 の既知の弱点)。
        let missing: Vec<u32> = AUGMENTS
            .keys()
            .copied()
            .filter(|id| item_by_id(*id).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "augments.json references {} unknown item ids: {:?}",
            missing.len(),
            &missing[..missing.len().min(10)]
        );
    }
}
