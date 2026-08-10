//! 連携ボーナス (Skillchain Bonus) の 4 ソース合算検証。
//!
//! `web/test/skillchain-bonus.test.js` からの移植 (docs/adr/0010 手順 5)。
//! 元のテストは JS の装備解釈と、ジョブ特性・ギフトを JS 側で再実装したものを
//! 使っていた。装備解釈が Rust に移り (docs/adr/0010)、特性とギフトは元々
//! Rust にあるため、突き合わせる意味が無くなったので Rust 側へ移した。
//!
//! ソース:
//!   1. 装備の「連携ダメージ」表記オーグメント (例: ムパカキャップ Default rank 30)
//!   2. 装備の "Skillchain bonus +N" 表記 (例: C. Palug Hammer)
//!   3. ジョブ特性 (wiki: https://wiki.ffo.jp/html/20337.html)
//!   4. ジョブポイントギフト (SAM / DNC)
//!
//! # 移植時に見つかった食い違い
//!
//! 元の JS テストはギフトの 2 番目の閾値を 450 JP としていたが、Rust 実装は
//! 500 JP である。`gift.rs` のギフト全系統が例外なく 150/500/1125/2000 の
//! パターンを持つため、Rust 側の 500 を正とした。JS 側の 450 は、`gift.rs` へ
//! 移行する前の古い `data/job_gifts.json` をミラーした際の誤りと思われる
//! (同ファイルは現存しない)。
//!
//! 元のテストは中間閾値を 450 で検証していたため、この誤りが検出されていなかった。
//! 移植にあたり中間閾値の検証を残しつつ値を修正した。

use ff11sim::equip_stats::extract_all_stats;
use ff11sim::gift::Gift;
use ff11sim::items::{augments_by_item_id, item_by_id};
use ff11sim::job::{Job, JobTrait};

/// 指定アイテムの、指定経路・ランクのオーグメントテキストを得る。
fn augment_text(item_id: u32, path_type: &str, rank: i32) -> String {
    let aug = augments_by_item_id(item_id)
        .unwrap_or_else(|| panic!("id {item_id} にオーグメント定義が無い"));
    let path = aug
        .paths
        .iter()
        .find(|p| p.path_type == path_type)
        .unwrap_or_else(|| panic!("id {item_id} に経路 {path_type} が無い"));
    path.ranks
        .iter()
        .find(|r| r.rank == rank)
        .unwrap_or_else(|| panic!("id {item_id} の {path_type} に rank {rank} が無い"))
        .text
        .clone()
}

// ---------------------------------------------------------------------------
// ソース 1: 装備のオーグメント
// ---------------------------------------------------------------------------

#[test]
fn source1_augment_skillchain_damage() {
    // ムパカキャップ Default rank 30。「連携ダメージ+15%」相当。
    // オーグメントは日本語表記なので、JA→EN 変換を経ずに英語表記へ至る経路が
    // 現状は無い。ここでは英語表記に直したテキストで抽出を確認する
    // (JA→EN 変換自体の移植は docs/adr/0010 手順 5 の残作業)。
    let text = augment_text(23758, "Default", 30);
    assert!(text.contains("連携ダメージ"), "想定した表記が無い: {text}");

    let en = "Skillchain bonus+15%";
    assert_eq!(extract_all_stats(en).skillchain_bonus, 15);
}

// ---------------------------------------------------------------------------
// ソース 2: 装備の英語表記
// ---------------------------------------------------------------------------

#[test]
fn source2_equipment_skillchain_bonus_text() {
    // C. Palug Hammer (パルーグハンマー, id 21071)。説明文に "Skillchain bonus +7" を持つ。
    let item = item_by_id(21071).expect("id 21071 が無い");
    let desc = item.description_en.as_deref().unwrap_or("");
    let stats = extract_all_stats(desc);
    assert_eq!(stats.skillchain_bonus, 7, "{} の連携ボーナス", item.en);
}

// ---------------------------------------------------------------------------
// ソース 3: ジョブ特性
// ---------------------------------------------------------------------------

#[test]
fn source3_job_trait_by_level() {
    let scb = |job: Job, lv: i32| job.trait_bonus(JobTrait::SkillchainBonus, lv);

    // SAM: 78 / 88 / 98 で rank 1..3、累積値は [8, 12, 16, 20, 23]
    assert_eq!(scb(Job::Sam, 77), 0, "未習得");
    assert_eq!(scb(Job::Sam, 78), 8);
    assert_eq!(scb(Job::Sam, 88), 12);
    assert_eq!(scb(Job::Sam, 98), 16);
    assert_eq!(scb(Job::Sam, 99), 16);

    // MNK / NIN: 85 / 95
    assert_eq!(scb(Job::Mnk, 84), 0);
    assert_eq!(scb(Job::Mnk, 85), 8);
    assert_eq!(scb(Job::Mnk, 95), 12);
    assert_eq!(scb(Job::Nin, 95), 12);

    // DNC: 45 / 58 / 71 / 84 / 97 の 5 段階
    assert_eq!(scb(Job::Dnc, 99), 23);

    // 習得しないジョブ
    assert_eq!(scb(Job::War, 99), 0);
}

// ---------------------------------------------------------------------------
// ソース 4: ジョブポイントギフト
// ---------------------------------------------------------------------------

#[test]
fn source4_job_point_gift_thresholds() {
    let g = |job: Job, jp: i32| job.gift_value(Gift::SkillchainBonus, jp);

    // 閾値は 150 / 500 / 1125 / 2000 で累積 2 / 4 / 6 / 8。
    // 元の JS テストは 2 番目を 450 としていたが誤り (モジュール冒頭の注記を参照)。
    assert_eq!(g(Job::Sam, 0), 0);
    assert_eq!(g(Job::Sam, 149), 0);
    assert_eq!(g(Job::Sam, 150), 2);
    assert_eq!(g(Job::Sam, 499), 2, "500 未満は 1 段階目のまま");
    assert_eq!(g(Job::Sam, 500), 4);
    assert_eq!(g(Job::Sam, 1124), 4);
    assert_eq!(g(Job::Sam, 1125), 6);
    assert_eq!(g(Job::Sam, 1999), 6);
    assert_eq!(g(Job::Sam, 2000), 8);
    assert_eq!(g(Job::Sam, 2100), 8, "全振りでも上限は 8");

    assert_eq!(g(Job::Dnc, 2100), 8);

    // ギフトを持たないジョブ
    assert_eq!(g(Job::War, 2100), 0);
}

// ---------------------------------------------------------------------------
// 統合: 4 ソースの合算
// ---------------------------------------------------------------------------

#[test]
fn all_sources_sum_up() {
    // SAM99 / JP 全振り (2100) / C. Palug Hammer 装備という構成での合計。
    let job = Job::Sam;
    let lv = 99;
    let total_jp = 2100;

    let from_trait = job.trait_bonus(JobTrait::SkillchainBonus, lv);
    let from_gift = job.gift_value(Gift::SkillchainBonus, total_jp);
    let from_equip = extract_all_stats(
        item_by_id(21071)
            .expect("id 21071 が無い")
            .description_en
            .as_deref()
            .unwrap_or(""),
    )
    .skillchain_bonus;
    let from_augment = extract_all_stats("Skillchain bonus+15%").skillchain_bonus;

    assert_eq!(from_trait, 16, "SAM99 のジョブ特性");
    assert_eq!(from_gift, 8, "JP 全振りのギフト");
    assert_eq!(from_equip, 7, "装備からの連携ボーナス");
    assert_eq!(from_augment, 15, "オーグメントからの連携ボーナス");

    // 16 (特性) + 8 (ギフト) + 7 (装備) + 15 (オーグメント) = 46
    let total = from_trait + from_gift + from_equip + from_augment;
    assert_eq!(total, 46, "4 ソースが単純合算される");
}
