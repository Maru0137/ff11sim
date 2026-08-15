//! 装備 (装備セットの 1 枠) と装備セット。概念モデルと責務は docs/adr/0018 を参照。
//!
//! - `EquipSet` がエンティティ (集約ルート)、`Equip` は値オブジェクト。
//!   `Equip` は識別子を持たず、属性がすべて等しければ交換可能。
//! - `Equip` は `Item` を内包せず `item_id` で参照する。オグメントも実体ではなく
//!   選択 (`aug_path` / `aug_rank`) だけを持ち、実体は `items` 側のマスタから引く。
//! - 説明文 → 数値の解釈は `equip_stats` が持つ。このモジュールは
//!   「どのテキストを集めて、どう合算するか」だけを持つ。
//! - スロット横断の集計は単一の `Equip` では決められないため `EquipSet` の責務。

use std::collections::BTreeMap;

use serde::Deserialize;
use strum::IntoEnumIterator;

use crate::equip_stats::{self, EquipStats, LabelScopeMode};
use crate::items::{augments_by_item_id, item_by_id};
use crate::skills::SkillKind;

/// スキルキー (`SkillKind::key()`) → ボーナス値。
pub type SkillBonuses = BTreeMap<&'static str, i32>;

/// 武器スロット。装備中のスロットによって扱いが変わる値の振り分けに使う。
/// JS 側のスロットキー `range` に対応するのは `Ranged`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponSlot {
    Main,
    Sub,
    Ranged,
}

impl WeaponSlot {
    /// スロットキーから武器スロットを判定する。武器スロットでなければ `None`。
    pub fn from_slot_key(key: &str) -> Option<Self> {
        match key {
            "main" => Some(Self::Main),
            "sub" => Some(Self::Sub),
            "range" => Some(Self::Ranged),
            _ => None,
        }
    }
}

/// 装備 1 枠 (値オブジェクト)。
///
/// web の `EquipSlotData` のうち解釈に必要なフィールドだけを受け取る。
/// `name_en` / `name_ja` / `description_ja` / `skill` は `item_id` から引ける
/// マスタのコピーなので持たない (docs/adr/0018)。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Equip {
    pub item_id: u32,
    /// オグメント経路の選択 (`items::ItemAugments::paths` の添字)
    pub aug_path: Option<usize>,
    /// オグメントランクの選択
    pub aug_rank: Option<i32>,
    /// ユーザーが自由に書ける説明文。実行時にしか解釈できない (docs/adr/0010)
    pub custom_description: Option<String>,
}

impl Equip {
    /// 選択中のオグメント文 (日本語)。web の `getAugmentText` と同じ結果を返す。
    pub fn augment_text(&self) -> Option<&'static str> {
        let path_index = self.aug_path?;
        let rank = self.aug_rank?;
        let path = augments_by_item_id(self.item_id)?.paths.get(path_index)?;
        path.ranks
            .iter()
            .find(|r| r.rank == rank)
            .map(|r| r.text.as_str())
    }

    /// 解釈に使う 3 ソースを日本語のまま返す。順序は
    /// アイテム説明文 → オグメント → カスタム説明。空のソースは含めない。
    ///
    /// 3 つとも元が日本語なので、そのまま並べられる (docs/adr/0019)。
    /// 条件ラベル (`ペット:` `潜在能力:` など) の配下はここで落とすので、
    /// 任意名を読む経路 (docs/adr/0015 のユーザー定義プロパティ) と
    /// 固定項目を読む経路 (`normalized_texts`) が同じ規則で揃う。
    /// `コンビネーション:` だけは常時扱いで残る。
    ///
    /// ラベルの適用範囲はソースによって変える。アイテム説明文の改行はゲーム内表示の
    /// 折り返しなのでラベルは次のラベル/文末まで及ぶが、オグメント文の改行は
    /// wiki の `<br>` 由来で効果 1 件ごとの区切りなので行末まで
    /// (docs/knowledge/items/description_labels.md)。カスタム説明はユーザーが
    /// 行単位で書くのでオグメント文に揃える。
    pub fn texts_ja(&self) -> Vec<String> {
        let mut texts = Vec::new();
        if let Some(desc) = item_by_id(self.item_id).and_then(|i| i.description_ja.as_deref())
            && !desc.is_empty()
        {
            texts.push((desc, LabelScopeMode::Wrapped));
        }
        if let Some(aug) = self.augment_text() {
            texts.push((aug, LabelScopeMode::PerLine));
        }
        if let Some(custom) = self.custom_description.as_deref()
            && !custom.is_empty()
        {
            texts.push((custom, LabelScopeMode::PerLine));
        }
        texts
            .into_iter()
            .map(|(text, mode)| equip_stats::strip_conditional_labels(text, mode))
            .filter(|t| !t.trim().is_empty())
            .collect()
    }

    /// `texts_ja` を、固定項目の抽出器が読める正規表記に寄せて返す (docs/adr/0019)。
    ///
    /// 正規表記が英語なのは抽出器が英語説明文からの移植 (docs/adr/0010) だからで、
    /// 翻訳ではなく表記の正規化として扱う。表に無い表記は素通りするため、
    /// 正規化層の語彙が足りないぶんは取りこぼしになる。
    pub fn normalized_texts(&self) -> Vec<String> {
        self.texts_ja()
            .iter()
            .map(|t| equip_stats::convert_augment_ja_to_en(t))
            .collect()
    }
}

/// 装備セット (エンティティ / 集約ルート)。キーはスロットキー (`main` / `body` / `range` …)。
///
/// 値が `None` のスロットは「装備なし」。`EquipSet` の同一性は
/// `{character, job, name}` だが、それらは保存レコード側の属性で、
/// 解釈には要らないのでここでは持たない。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct EquipSet {
    pub slots: BTreeMap<String, Option<Equip>>,
}

/// 装備セットの合算結果。
///
/// スロット別の内訳を分けて持つのは、表示側がスロット依存の出し分け
/// (武器スロットのみの魔命スキル、内訳モーダル docs/adr/0016) に使うため。
#[derive(Debug, Clone, Default)]
pub struct EquipSetBonuses {
    /// 全スロット合計
    pub total: EquipStats,
    /// 武器スロット別合計 (装備のないスロットも 0 で持つ)
    pub main_stats: EquipStats,
    pub sub_stats: EquipStats,
    pub ranged_stats: EquipStats,
    /// スロット別合計。キーは生のスロットキー (`range` は `range` のまま)。
    /// 装備のあるスロットのみ含む。
    pub per_slot_stats: BTreeMap<String, EquipStats>,
    /// 武器スロット装備の「武器スキル」ボーナス。そのスロットでしか効かない。
    pub skill_bonus_main: SkillBonuses,
    pub skill_bonus_sub: SkillBonuses,
    pub skill_bonus_ranged: SkillBonuses,
    /// 上記以外のスキルボーナス (非武器スロット装備すべてと、武器スロット装備の非武器スキル)。
    pub skill_bonus_global: SkillBonuses,
    /// スロット別スキルボーナス。説明文を 1 つでも持つスロットのみ含む。
    pub per_slot_skill_bonuses: BTreeMap<String, SkillBonuses>,
}

fn add_bonuses(target: &mut SkillBonuses, key: &'static str, value: i32) {
    *target.entry(key).or_insert(0) += value;
}

/// スキルキーが武器スキルかどうか。
fn is_weapon_skill(key: &str) -> bool {
    SkillKind::iter().any(|s| s.is_weapon() && s.key() == key)
}

impl EquipSet {
    /// 装備セット全体のステータス・スキルボーナスを合算する。
    ///
    /// 各装備の 3 ソースを抽出して足し込むだけだが、スキルボーナスだけは
    /// 振り分け規則がある: 武器スロット (main/sub/range) 装備の **武器スキル** は
    /// そのスロット専用、それ以外 (非武器スロット装備のすべて、武器スロット装備の
    /// 非武器スキル) は全スロット共通として扱う。
    pub fn bonuses(&self) -> EquipSetBonuses {
        let mut out = EquipSetBonuses::default();

        for (slot_key, equip) in &self.slots {
            let Some(equip) = equip else { continue };
            let weapon_slot = WeaponSlot::from_slot_key(slot_key);
            // 装備のあるスロットは、抽出結果が空でもキーを持つ
            let per_slot = out.per_slot_stats.entry(slot_key.clone()).or_default();

            for text in equip.normalized_texts() {
                let stats = equip_stats::extract_all_stats(&text);
                out.total.add(&stats);
                per_slot.add(&stats);
                match weapon_slot {
                    Some(WeaponSlot::Main) => out.main_stats.add(&stats),
                    Some(WeaponSlot::Sub) => out.sub_stats.add(&stats),
                    Some(WeaponSlot::Ranged) => out.ranged_stats.add(&stats),
                    None => {}
                }

                let per_slot_skills = out
                    .per_slot_skill_bonuses
                    .entry(slot_key.clone())
                    .or_default();
                let skills = equip_stats::extract_skill_bonuses(&text);
                for (key, value) in &skills {
                    if *value == 0 {
                        continue;
                    }
                    add_bonuses(per_slot_skills, key, *value);
                    match weapon_slot {
                        Some(slot) if is_weapon_skill(key) => match slot {
                            WeaponSlot::Main => add_bonuses(&mut out.skill_bonus_main, key, *value),
                            WeaponSlot::Sub => add_bonuses(&mut out.skill_bonus_sub, key, *value),
                            WeaponSlot::Ranged => {
                                add_bonuses(&mut out.skill_bonus_ranged, key, *value)
                            }
                        },
                        _ => add_bonuses(&mut out.skill_bonus_global, key, *value),
                    }
                }
            }
        }
        out
    }

    /// ユーザー定義プロパティ (docs/adr/0015) のスロット別の値を返す。
    ///
    /// `terms` は抽出する日本語プロパティ名。戻り値は スロットキー → プロパティ名 → 値。
    /// 値 0 は持たず、値のないスロットはキーごと含まない (疎)。表示側が
    /// 「このスロットは寄与なし」を空で判定するため。
    ///
    /// `bonuses` と違い、テキストごとに **最初の一致 1 件だけ** を取る
    /// (`equip_stats::extract_stat_from_description`)。合算するのはスロット間と
    /// 3 ソース間のみ。
    pub fn property_values(&self, terms: &[String]) -> BTreeMap<String, BTreeMap<String, i32>> {
        let mut out: BTreeMap<String, BTreeMap<String, i32>> = BTreeMap::new();
        if terms.is_empty() {
            return out;
        }
        for (slot_key, equip) in &self.slots {
            let Some(equip) = equip else { continue };
            for text in equip.texts_ja() {
                for term in terms {
                    let value = equip_stats::extract_stat_from_description(&text, term);
                    if value == 0 {
                        continue;
                    }
                    *out.entry(slot_key.clone())
                        .or_default()
                        .entry(term.clone())
                        .or_insert(0) += value;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equip(item_id: u32) -> Equip {
        Equip {
            item_id,
            ..Default::default()
        }
    }

    fn custom(text: &str) -> Equip {
        Equip {
            custom_description: Some(text.to_string()),
            ..Default::default()
        }
    }

    fn set(slots: &[(&str, Equip)]) -> EquipSet {
        EquipSet {
            slots: slots
                .iter()
                .map(|(k, v)| (k.to_string(), Some(v.clone())))
                .collect(),
        }
    }

    // --- Equip: 値オブジェクトとしての性質 -------------------------------

    #[test]
    fn equips_with_same_attributes_are_equal() {
        assert_eq!(custom("命中+10"), custom("命中+10"));
        assert_ne!(custom("命中+10"), custom("命中+11"));
    }

    // --- Equip: 3 ソースの解決 -------------------------------------------

    #[test]
    fn normalized_texts_collects_item_description() {
        // ヒポメネソックス+1 (27410)。日本語説明文を正規化して読む (docs/adr/0019)
        let texts = equip(27410).normalized_texts();
        assert_eq!(texts.len(), 1, "アイテム説明文のみ: {texts:?}");
        assert!(texts[0].contains("HP+13"), "{}", texts[0]);
    }

    #[test]
    fn normalized_texts_normalizes_custom_description() {
        let texts = custom("命中+10 ダブルアタック+3%").normalized_texts();
        assert_eq!(texts.len(), 1);
        assert!(texts[0].contains("Accuracy+10"), "{}", texts[0]);
        assert!(texts[0].contains("\"Double Attack\"+3%"), "{}", texts[0]);
    }

    #[test]
    fn normalized_texts_skips_empty_sources() {
        // 存在しない item_id、オグメント未選択、カスタム説明なし
        assert!(equip(0).normalized_texts().is_empty());
    }

    #[test]
    fn augment_text_needs_both_path_and_rank() {
        let mut e = equip(27410);
        assert_eq!(e.augment_text(), None, "未選択なら None");
        e.aug_path = Some(0);
        assert_eq!(e.augment_text(), None, "ランク未選択なら None");
    }

    // --- EquipSet: 合算 ---------------------------------------------------

    #[test]
    fn bonuses_sums_stats_across_slots() {
        let b = set(&[("body", custom("命中+10")), ("hands", custom("命中+5"))]).bonuses();
        assert_eq!(b.total.accuracy, 15);
        assert_eq!(b.per_slot_stats["body"].accuracy, 10);
        assert_eq!(b.per_slot_stats["hands"].accuracy, 5);
    }

    #[test]
    fn bonuses_sums_all_three_sources_of_one_slot() {
        // アイテム説明文とカスタム説明の両方が足される
        let mut e = equip(27410); // ヒポメネソックス+1 は HP+13
        e.custom_description = Some("HP+7".to_string());
        let b = set(&[("feet", e)]).bonuses();
        assert_eq!(b.total.hp, 20);
    }

    // 以下のスキル振り分けテストはカスタム説明を英語で書く。JA→EN 変換表
    // (AUGMENT_JA_TO_EN) は魔法スキルしか持たず、「片手剣スキル」のような武器スキルは
    // 変換されないため。ここで検証したいのは変換ではなく振り分け規則。

    #[test]
    fn weapon_skill_of_weapon_slot_goes_to_that_slot_only() {
        let b = set(&[("main", custom("Sword skill+10"))]).bonuses();
        assert_eq!(b.skill_bonus_main.get("Sword"), Some(&10));
        assert!(
            b.skill_bonus_global.is_empty(),
            "{:?}",
            b.skill_bonus_global
        );
    }

    #[test]
    fn non_weapon_skill_of_weapon_slot_goes_to_global() {
        // 武器スロット装備でも、武器スキル以外は全スロット共通
        let b = set(&[("main", custom("Evasion skill+10"))]).bonuses();
        assert_eq!(b.skill_bonus_global.get("Evasion"), Some(&10));
        assert!(b.skill_bonus_main.is_empty(), "{:?}", b.skill_bonus_main);
    }

    #[test]
    fn weapon_skill_of_non_weapon_slot_goes_to_global() {
        let b = set(&[("body", custom("Sword skill+10"))]).bonuses();
        assert_eq!(b.skill_bonus_global.get("Sword"), Some(&10));
        assert!(b.skill_bonus_main.is_empty(), "{:?}", b.skill_bonus_main);
    }

    #[test]
    fn range_slot_maps_to_ranged_bucket() {
        let b = set(&[("range", custom("Archery skill+10"))]).bonuses();
        assert_eq!(b.skill_bonus_ranged.get("Archery"), Some(&10));
        // スロット別の内訳は生のスロットキー (`range`) で持つ
        assert!(b.per_slot_skill_bonuses.contains_key("range"));
    }

    #[test]
    fn weapon_slot_stats_are_kept_separately() {
        let b = set(&[("main", custom("命中+10")), ("body", custom("命中+5"))]).bonuses();
        assert_eq!(b.main_stats.accuracy, 10);
        assert_eq!(b.sub_stats.accuracy, 0);
        assert_eq!(b.total.accuracy, 15);
    }

    #[test]
    fn per_slot_stats_sum_to_total() {
        let b = set(&[
            ("main", custom("命中+10")),
            ("body", custom("命中+5")),
            ("hands", equip(27410)),
        ])
        .bonuses();
        let sum: i32 = b.per_slot_stats.values().map(|s| s.accuracy).sum();
        assert_eq!(sum, b.total.accuracy);
    }

    // --- EquipSet: ユーザー定義プロパティ (docs/adr/0015) ------------------

    fn terms(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn property_values_sums_across_slots_and_sources() {
        let mut body = custom("二刀流+3");
        body.item_id = 11555; // フェリンマント: DEX+4 CHR+4 命中+7 ペット:命中+10
        let v = set(&[("body", body), ("hands", custom("二刀流+2"))])
            .property_values(&terms(&["二刀流", "命中"]));
        assert_eq!(v["body"]["二刀流"], 3);
        assert_eq!(v["body"]["命中"], 7, "ペット行の命中+10 は拾わない");
        assert_eq!(v["hands"]["二刀流"], 2);
    }

    // --- 条件ラベルの規則を両経路で共有する (docs/adr/0019 手順 3) ---------

    #[test]
    fn conditional_labels_are_excluded_from_both_paths() {
        // イーガダブレット (11338): 防21 ペット:命中+3 モクシャ+3
        let set = set(&[("body", equip(11338))]);
        assert_eq!(set.bonuses().total.accuracy, 0, "ペット行は本体に乗らない");
        assert!(
            set.property_values(&terms(&["命中", "モクシャ"]))
                .is_empty(),
            "ユーザー定義プロパティ側も同じ"
        );
    }

    #[test]
    fn combination_is_counted_by_both_paths() {
        // マガバンフロック (10464): ... 魔攻+7 魔防+5\nコンビネーション:魔命+5
        // セット効果は常時扱いにする (docs/adr/0019)
        let set = set(&[("body", equip(10464))]);
        assert_eq!(set.bonuses().total.magic_accuracy, 5);
        assert_eq!(set.property_values(&terms(&["魔命"]))["body"]["魔命"], 5);
    }

    #[test]
    fn pet_label_with_fullwidth_colon_is_excluded() {
        // 寵愛総面 (24182): 本体 命中+42 魔命+42 飛命+42、Rank30 オグメントが
        // 「命中+30 飛命+30 魔命+30 / ペット：命中+30 魔命+30 / ...」。
        // 全角コロンをラベルと認識できず、ペットの命中/魔命が本体に乗っていた
        let mut e = equip(24182);
        e.aug_path = Some(0);
        e.aug_rank = Some(30);
        let b = set(&[("head", e)]).bonuses();
        assert_eq!(b.total.accuracy, 72);
        assert_eq!(b.total.magic_accuracy, 72);
        assert_eq!(b.total.ranged_accuracy, 72);
    }

    #[test]
    fn item_description_label_covers_wrapped_lines() {
        // ＦＯチュリダル+4 (24047): 本体 ヘイスト+6%、オートマトン行の折り返しに
        // 「ヘイスト+5% 被ダメージ-6%」がある。どちらもペットのもの
        let b = set(&[("legs", equip(24047))]).bonuses();
        assert_eq!(b.total.haste_pct, 6);
        assert_eq!(b.total.damage_taken_pct, 0);
    }

    #[test]
    fn augment_label_stops_at_the_line_end() {
        // ジャストクラウン (24165) の Rank30 オグメント:
        // 「命中+30 魔命+30 / ペット:命中+30 魔命+30 / ストアTP+7 / トリプルアタック+4%」。
        // オグメント文の改行は効果ごとの区切りなので、3・4 行目は本体に乗る
        let mut e = equip(24165);
        e.aug_path = Some(0);
        e.aug_rank = Some(30);
        let b = set(&[("head", e)]).bonuses();
        assert_eq!(b.total.store_tp, 7);
        assert_eq!(b.total.triple_attack_pct, 4);
    }

    #[test]
    fn both_paths_agree_on_the_same_item() {
        // フェリンマント (11555): DEX+4 CHR+4 命中+7 ペット:命中+10
        let set = set(&[("back", equip(11555))]);
        let stats = set.bonuses();
        let props = set.property_values(&terms(&["命中"]));
        assert_eq!(stats.total.accuracy, 7);
        assert_eq!(props["back"]["命中"], 7, "経路 A と B で同じ値になる");
    }

    #[test]
    fn property_values_are_not_converted_to_english() {
        // 「命中」は JA→EN 変換を通すと Accuracy になる語。日本語のまま渡している証拠
        let v = set(&[("body", custom("命中+10"))]).property_values(&terms(&["命中"]));
        assert_eq!(v["body"]["命中"], 10);
    }

    #[test]
    fn property_values_are_sparse() {
        let v = set(&[("body", custom("命中+10")), ("hands", custom("なにもない"))])
            .property_values(&terms(&["命中", "二刀流"]));
        assert!(!v["body"].contains_key("二刀流"), "値 0 は持たない");
        assert!(!v.contains_key("hands"), "寄与のないスロットは持たない");
    }

    #[test]
    fn property_values_takes_first_match_per_text() {
        // テキストごとに最初の 1 件のみ (docs/adr/0015)
        let v =
            set(&[("body", custom("二刀流+3 何か 二刀流+4"))]).property_values(&terms(&["二刀流"]));
        assert_eq!(v["body"]["二刀流"], 3);
    }

    #[test]
    fn empty_slot_is_skipped_but_equipped_slot_always_has_an_entry() {
        let mut s = set(&[("body", equip(0))]);
        s.slots.insert("hands".to_string(), None);
        let b = s.bonuses();
        // 抽出結果が空でも、装備のあるスロットはキーを持つ (内訳表示のため)
        assert!(b.per_slot_stats.contains_key("body"));
        assert!(!b.per_slot_stats.contains_key("hands"));
        // 説明文が 1 つも無いスロットはスキルボーナスのキーを持たない
        assert!(!b.per_slot_skill_bonuses.contains_key("body"));
    }
}
