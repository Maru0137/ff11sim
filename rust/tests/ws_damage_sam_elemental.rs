//! SAM99 属性 WS 構成での装備合計 WS ダメージ% 検証。
//!
//! `web/test/ws-damage-sam-elemental.test.js` からの移植 (docs/adr/0010 手順 5)。
//!
//! 装備セットは 3 つの経路からステータスを拾う。すべて合算されることを確認する。
//!   1. 装備の説明文 (`description_en`)
//!   2. オーグメント (`augments.json` の経路 × ランク、日本語表記なので JA→EN 変換を通す)
//!   3. カスタム入力 (ユーザーが自由記述する欄。日本語表記)
//!
//! 元の JS テストは `AUGMENT_JA_TO_EN` を 32 エントリだけコピーして持っており、
//! 実装の 76 エントリに対して 44 件欠けていた
//! (docs/tech-debt/mirrored-constants-in-tests.md の事例 2)。
//! 移植版は実装の `convert_augment_ja_to_en` を直接呼ぶため、この問題は起きない。

use ff11sim::equip_stats::{EquipStats, convert_augment_ja_to_en, extract_all_stats};
use ff11sim::items::{augments_by_item_id, item_by_id};

/// 装備セット 1 スロット分の指定。
struct Slot {
    slot: &'static str,
    item_id: u32,
    /// `augments.json` の `paths` 配列のインデックスと、そのランク。
    augment: Option<(usize, i32)>,
    /// カスタム入力欄 (日本語表記)。
    custom: Option<&'static str>,
}

const fn s(slot: &'static str, item_id: u32) -> Slot {
    Slot {
        slot,
        item_id,
        augment: None,
        custom: None,
    }
}
const fn s_aug(slot: &'static str, item_id: u32, path: usize, rank: i32) -> Slot {
    Slot {
        slot,
        item_id,
        augment: Some((path, rank)),
        custom: None,
    }
}
const fn s_custom(slot: &'static str, item_id: u32, custom: &'static str) -> Slot {
    Slot {
        slot,
        item_id,
        augment: None,
        custom: Some(custom),
    }
}

/// SAM99 属性 WS 構成。ID と装備名の対応は items.json と突き合わせ済み。
const EQUIP_SET: &[Slot] = &[
    s_aug("main", 21025, 0, 15), // 童子切安綱 Default rank 15
    s("sub", 22212),             // ウトゥグリップ
    s("ammo", 22281),            // ノブキエリ
    s_aug("head", 23761, 1, 30), // ニャメヘルム Type:B rank 30
    s("neck", 27510),            // フォシャゴルゲット
    s_custom("ear1", 11697, "魔攻+4 TPボーナス+250"), // 胡蝶のイヤリング
    s("ear2", 28514),            // フリオミシピアス
    s_aug("body", 23768, 1, 30), // ニャメメイル Type:B rank 30
    s_aug("hands", 23775, 1, 30), // ニャメガントレ Type:B rank 30
    s("ring1", 26227),           // コーネリアリング
    s("ring2", 26214),           // エパミノダスリング
    s_custom(
        "back",
        26257,
        "STR+30 命中+20 攻+20 ウェポンスキルのダメージ+10% 被物理ダメージ-10%",
    ), // スメルトリオマント
    s("waist", 26359),           // オルペウスサッシュ
    s_aug("legs", 23782, 1, 30), // ニャメフランチャ Type:B rank 30
    s_aug("feet", 23789, 1, 30), // ニャメソルレット Type:B rank 30
];

/// 元の JS テストが検証していた期待値。
const EXPECTED_WS_DAMAGE_PCT: i32 = 89;

fn augment_text(item_id: u32, path_idx: usize, rank: i32) -> String {
    let aug = augments_by_item_id(item_id)
        .unwrap_or_else(|| panic!("id {item_id} にオーグメント定義が無い"));
    let path = aug
        .paths
        .get(path_idx)
        .unwrap_or_else(|| panic!("id {item_id} に paths[{path_idx}] が無い"));
    path.ranks
        .iter()
        .find(|r| r.rank == rank)
        .unwrap_or_else(|| panic!("id {item_id} の paths[{path_idx}] に rank {rank} が無い"))
        .text
        .clone()
}

/// 装備セット全体のステータスを合算する。
/// 1 スロットにつき「説明文」「オーグメント」「カスタム入力」の 3 経路を足す。
fn sum_equip_set() -> EquipStats {
    let mut total = EquipStats::default();
    for slot in EQUIP_SET {
        let item = item_by_id(slot.item_id)
            .unwrap_or_else(|| panic!("id {} が items.json に無い", slot.item_id));
        total = add_stats(
            &total,
            &extract_all_stats(item.description_en.as_deref().unwrap_or("")),
        );

        if let Some((path, rank)) = slot.augment {
            let en = convert_augment_ja_to_en(&augment_text(slot.item_id, path, rank));
            total = add_stats(&total, &extract_all_stats(&en));
        }
        if let Some(custom) = slot.custom {
            let en = convert_augment_ja_to_en(custom);
            total = add_stats(&total, &extract_all_stats(&en));
        }
    }
    total
}

/// 2 つの `EquipStats` を項目ごとに足す。
/// JS の `sumStats` に対応する処理で、まだ Rust 側に実装が無いためここで持つ
/// (index.html がまだ JS の sumStats を使っている。docs/adr/0010 手順 5 の残作業)。
fn add_stats(a: &EquipStats, b: &EquipStats) -> EquipStats {
    let mut out = a.clone();
    out.hp += b.hp;
    out.mp += b.mp;
    out.str_ += b.str_;
    out.dex += b.dex;
    out.vit += b.vit;
    out.agi += b.agi;
    out.int += b.int;
    out.mnd += b.mnd;
    out.chr += b.chr;
    out.def += b.def;
    out.attack += b.attack;
    out.accuracy += b.accuracy;
    out.evasion += b.evasion;
    out.magic_attack += b.magic_attack;
    out.magic_accuracy += b.magic_accuracy;
    out.store_tp += b.store_tp;
    out.weapon_skill_damage_pct += b.weapon_skill_damage_pct;
    out.tp_bonus += b.tp_bonus;
    out.skillchain_bonus += b.skillchain_bonus;
    out.physical_damage_taken_pct += b.physical_damage_taken_pct;
    out.double_attack_pct += b.double_attack_pct;
    out.critical_hit_rate_pct += b.critical_hit_rate_pct;
    out
}

#[test]
fn equip_set_ids_resolve_to_expected_items() {
    // ID の取り違えを防ぐため、装備名まで確認する。
    let expected: &[(u32, &str)] = &[
        (21025, "童子切安綱"),
        (22212, "ウトゥグリップ"),
        (22281, "ノブキエリ"),
        (23761, "ニャメヘルム"),
        (27510, "フォシャゴルゲット"),
        (11697, "胡蝶のイヤリング"),
        (28514, "フリオミシピアス"),
        (23768, "ニャメメイル"),
        (23775, "ニャメガントレ"),
        (26227, "コーネリアリング"),
        (26214, "エパミノダスリング"),
        (26257, "スメルトリオマント"),
        (26359, "オルペウスサッシュ"),
        (23782, "ニャメフランチャ"),
        (23789, "ニャメソルレット"),
    ];
    for (id, name) in expected {
        let item = item_by_id(*id).unwrap_or_else(|| panic!("id {id} が無い"));
        assert!(
            item.ja.contains(name),
            "id {id} は {name} のはずが {} だった",
            item.ja
        );
    }
    assert_eq!(EQUIP_SET.len(), expected.len());

    // 各装備が指定スロットに装備可能であることも確認する。
    // ear1/ear2 と ring1/ring2 は items.json 上どちらか一方で表現される。
    for slot in EQUIP_SET {
        let item = item_by_id(slot.item_id).expect("id が無い");
        let acceptable: Vec<&str> = match slot.slot {
            "ear1" | "ear2" => vec!["ear1", "ear2"],
            "ring1" | "ring2" => vec!["ring1", "ring2"],
            other => vec![other],
        };
        assert!(
            item.slots.iter().any(|s| acceptable.contains(&s.as_str())),
            "{} ({}) は {} に装備できない: {:?}",
            item.ja,
            slot.item_id,
            slot.slot,
            item.slots
        );
    }
}

#[test]
fn weapon_skill_damage_sums_to_expected() {
    let total = sum_equip_set();
    assert_eq!(
        total.weapon_skill_damage_pct, EXPECTED_WS_DAMAGE_PCT,
        "装備合計の WS ダメージ%"
    );
}

#[test]
fn custom_and_augment_sources_are_included() {
    // カスタム入力とオーグメントが実際に合算に効いていることを確認する。
    // どちらかが無視されていると WS ダメージ% の合計だけでは気付けない。
    let back_custom = convert_augment_ja_to_en(
        "STR+30 命中+20 攻+20 ウェポンスキルのダメージ+10% 被物理ダメージ-10%",
    );
    let s = extract_all_stats(&back_custom);
    assert_eq!(s.str_, 30, "カスタム入力の STR");
    assert_eq!(s.accuracy, 20, "カスタム入力の命中");
    assert_eq!(s.weapon_skill_damage_pct, 10, "カスタム入力の WS ダメージ%");
    assert_eq!(
        s.physical_damage_taken_pct, -10,
        "カスタム入力の被物理ダメージ"
    );

    // 童子切安綱 Default rank 15 のオーグメント
    let main_aug = convert_augment_ja_to_en(&augment_text(21025, 0, 15));
    assert!(
        !main_aug.is_empty(),
        "オーグメントテキストが空: id 21025 paths[0] rank 15"
    );
}
