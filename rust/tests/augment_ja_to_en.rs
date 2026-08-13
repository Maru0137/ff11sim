//! オーグメントの日本語→英語 変換が JS 実装と一致するかを検証する。
//!
//! `web/js/utils.js` の `convertAugmentJaToEn` と `web/js/constants.js` の
//! `AUGMENT_JA_TO_EN` を Rust へ移した (docs/adr/0010 手順 5)。
//!
//! 変換は単純な順次置換であり **並び順に意味がある** ため、表の順序が崩れると
//! 部分一致で壊れる。全オーグメントテキストで JS の出力と突き合わせることで
//! 順序込みの一致を保証する。
//!
//! **移植元の JS 実装 (web/js/constants.js の AUGMENT_JA_TO_EN) は削除済みなので、
//! 現在この比較は実行できない。** 移植時に全 1,646 件で一致を確認した。
//! 期待値 JSON を別途用意すれば動く形で残してあり、変換表を大きく変えるときに
//! 変更前後を突き合わせる用途で使える。
//!
//! 期待値 JSON の形式: `[[入力, 期待する変換結果], ...]`
//! 環境変数 AUG_CONVERTED でパスを渡す。未設定ならスキップする。

use ff11sim::equip_stats::convert_augment_ja_to_en;

#[test]
fn conversion_matches_js_for_all_augment_texts() {
    let Ok(path) = std::env::var("AUG_CONVERTED") else {
        eprintln!("AUG_CONVERTED 未設定のためスキップ");
        return;
    };
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} を読めない: {e}"));
    let pairs: Vec<(String, String)> = serde_json::from_str(&raw).expect("JSON parse failed");

    let mut mismatches = Vec::new();
    for (input, expected) in &pairs {
        let got = convert_augment_ja_to_en(input);
        if &got != expected {
            mismatches.push(format!(
                "入力: {input:?}\n    js  : {expected:?}\n    rust: {got:?}"
            ));
        }
    }

    eprintln!("比較件数: {}, 不一致: {}", pairs.len(), mismatches.len());
    for m in mismatches.iter().take(10) {
        eprintln!("  {m}");
    }
    assert!(
        mismatches.is_empty(),
        "JS と {} 件の不一致がある",
        mismatches.len()
    );
}

/// 並び順が壊れると失敗するケース。表の順序を変えた際の検出用。
#[test]
fn longer_notations_are_replaced_first() {
    // 「魔法クリティカルヒットII」は「魔法クリティカルヒット率」より先に置く必要がある。
    assert_eq!(
        convert_augment_ja_to_en("魔法クリティカルヒットII+5"),
        "Magic Crit. Hit Rate II+5"
    );
    // 「攻」は最後。「魔攻」「飛攻」が先に処理されないと壊れる。
    assert_eq!(
        convert_augment_ja_to_en("魔攻+30"),
        "\"Magic Atk. Bonus\"+30"
    );
    assert_eq!(convert_augment_ja_to_en("攻+30"), "Attack+30");
}

/// マジックバースト系の表記揺れ。
/// 「マジックバースト+N」(ソーサラストール系) は MB ダメージとして合算する。
/// 「マジックバースト命中+N」は MB ダメージにも命中にも合算しない。
#[test]
fn converts_magic_burst_variants() {
    // ソーサラストール+2 Rank15 の実オーグメント
    let converted = convert_augment_ja_to_en("マジックバースト+10\nマジックバースト命中+25");
    assert_eq!(converted, "Magic burst damage+10\nMagic burst accuracy+25");
    let stats = ff11sim::equip_stats::extract_all_stats(&converted);
    assert_eq!(stats.magic_burst_damage, 10);
    assert_eq!(stats.magic_burst_damage_2, 0);
    // MB命中は追跡対象外。命中/魔命へ誤合算しない
    assert_eq!(stats.accuracy, 0);
    assert_eq!(stats.magic_accuracy, 0);

    // 「マジックバーストダメージII+N」(ニャメ系) は II として抽出される (回帰確認)
    let converted = convert_augment_ja_to_en("マジックバーストダメージII+7");
    let stats = ff11sim::equip_stats::extract_all_stats(&converted);
    assert_eq!(stats.magic_burst_damage_2, 7);
    assert_eq!(stats.magic_burst_damage, 0);
}

/// 実データで代表的な変換を確認する。
/// 期待値は node で JS 実装を動かした結果をそのまま使っている。
#[test]
fn converts_real_augment_text() {
    // ムパカキャップ Default rank 30
    let converted =
        convert_augment_ja_to_en("攻+30\n連携ダメージ+15%\n命中+15 魔命+15\nダブルアタック+5%");
    assert_eq!(
        converted,
        "Attack+30\n\"Skillchain Bonus\"+15%\nAccuracy+15 Magic Accuracy+15\n\"Double Attack\"+5%"
    );
    // 変換後の文字列がそのまま抽出に掛かることも確認する。
    let stats = ff11sim::equip_stats::extract_all_stats(&converted);
    assert_eq!(stats.attack, 30);
    assert_eq!(stats.skillchain_bonus, 15);
    assert_eq!(stats.accuracy, 15);
    assert_eq!(stats.magic_accuracy, 15);
    assert_eq!(stats.double_attack_pct, 5);
}
