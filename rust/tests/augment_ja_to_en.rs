//! オーグメントの日本語→英語 変換が JS 実装と一致するかを検証する。
//!
//! `web/js/utils.js` の `convertAugmentJaToEn` と `web/js/constants.js` の
//! `AUGMENT_JA_TO_EN` を Rust へ移した (docs/adr/0010 手順 5)。
//!
//! 変換は単純な順次置換であり **並び順に意味がある** ため、表の順序が崩れると
//! 部分一致で壊れる。全オーグメントテキストで JS の出力と突き合わせることで
//! 順序込みの一致を保証する。
//!
//! 期待値の作り方:
//!   node で web/js/constants.js の表を読み、全オーグメントテキストを変換した
//!   結果を /tmp/aug-converted.json に出す。環境変数 AUG_CONVERTED で渡す。
//!   未設定ならスキップする (CI では JS を動かさないため)。

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
