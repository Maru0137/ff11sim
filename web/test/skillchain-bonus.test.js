// 連携ボーナス (Skillchain Bonus) の 4 ソース合算検証。
//
// ソース:
//   1. 装備の "連携ダメージ" 表記オーグメント (例: ムパカキャップ Default rank 30 = +15%)
//   2. 装備の "連携ボーナス" 表記 (例: C. Palug Hammer = +7)
//   3. ジョブ特性 (MNK/NIN/SAM/BLU/DNC、wiki: https://wiki.ffo.jp/html/20337.html)
//   4. ジョブポイントギフト (SAM/DNC, 150/450/1125/2000 JP で +2%/段、最大 +8%)
//
// 実行: node web/test/skillchain-bonus.test.js

const fs = require('fs');
const path = require('path');
const assert = require('assert');

const { extractAllStats, sumStats } = require('../js/equip-stats.js');

const dataDir = path.join(__dirname, '..', 'data');
const items = JSON.parse(fs.readFileSync(path.join(dataDir, 'items.json'), 'utf8')).items;
const augments = JSON.parse(fs.readFileSync(path.join(dataDir, 'augments.json'), 'utf8')).augments;
const itemById = Object.fromEntries(items.map((it) => [it.id, it]));

// web/js/constants.js の AUGMENT_JA_TO_EN と同期 (Node から ES モジュールを直接読めないためコピー)
// 元側を更新したらこちらも追従すること。
const AUGMENT_JA_TO_EN = [
    ['ウェポンスキルのダメージ', 'Weapon skill damage'],
    ['マジックバーストダメージ', 'Magic burst damage'],
    ['魔法クリティカルヒットII', 'Magic Crit. Hit Rate II'],
    ['魔法クリティカルヒット率', 'Magic Critical hit rate'],
    ['物理ダメージ上限', 'Physical damage limit'],
    ['被物理ダメージ', 'Physical damage taken'],
    ['被魔法ダメージ', 'Magic damage taken'],
    ['クリティカルヒットダメージ', 'Critical hit damage'],
    ['クリティカルヒット率', 'Critical hit rate'],
    ['トリプルアタックダメージ', 'Triple Attack damage'],
    ['トリプルアタック', '"Triple Attack"'],
    ['ダブルアタックダメージ', 'Double Attack damage'],
    ['ダブルアタック', '"Double Attack"'],
    ['クワッドアタック', '"Quadruple Attack"'],
    ['モクシャII', '"Subtle Blow II"'],
    ['モクシャ', '"Subtle Blow"'],
    ['魔法ダメージ', 'Magic Damage'],
    ['被ダメージ', 'Damage taken'],
    ['ストアTP', '"Store TP"'],
    ['TPボーナス', '"TP Bonus"'],
    ['連携ボーナス', '"Skillchain Bonus"'],
    ['連携ダメージ', '"Skillchain Bonus"'],
    ['トゥルーショット', '"True Shot"'],
    ['アフィニティ', 'Affinity'],
    ['ヘイスト', 'Haste'],
    ['魔回避', 'Magic Evasion'],
    ['飛攻', 'Ranged Attack'],
    ['飛命', 'Ranged Accuracy'],
    ['魔命', 'Magic Accuracy'],
    ['魔攻', '"Magic Atk. Bonus"'],
    ['回避', 'Evasion'],
    ['命中', 'Accuracy'],
    ['攻', 'Attack'],
];

function convertAugmentJaToEn(text) {
    let result = text;
    for (const [ja, en] of AUGMENT_JA_TO_EN) result = result.replaceAll(ja, en);
    return result;
}

function getAugmentText(itemId, pathIdx, rank) {
    const info = augments[String(itemId)];
    if (!info) return null;
    const p = info.paths[pathIdx];
    if (!p) return null;
    const r = p.ranks.find((rr) => rr.rank === rank);
    return r ? r.text : null;
}

// ジョブ特性: Rust 側 (rust/src/job.rs) の SkillchainBonus 定義をミラー。
// 累積値 (rank 1 = +8, rank 2 = +12, rank 3 = +16, rank 4 = +20, rank 5 = +23)
//   MNK/NIN: Lv85, 95             (rank 1, 2)
//   SAM:    Lv78, 88, 98          (rank 1, 2, 3)
//   BLU:    Lv83, 96, 99, 99, 99  (rank 1〜5)
//   DNC:    Lv45, 58, 71, 84, 97  (rank 1〜5)
const SCB_TRAIT_CUMULATIVE = [8, 12, 16, 20, 23];
const SCB_TRAIT_LEVELS = {
    Mnk: [85, 95],
    Nin: [85, 95],
    Sam: [78, 88, 98],
    Blu: [83, 96, 99, 99, 99],
    Dnc: [45, 58, 71, 84, 97],
};
function skillchainBonusJobTrait(job, lv) {
    const levels = SCB_TRAIT_LEVELS[job] || [];
    const rank = levels.filter((req) => lv >= req).length;
    if (rank === 0) return 0;
    return SCB_TRAIT_CUMULATIVE[Math.min(rank, SCB_TRAIT_CUMULATIVE.length) - 1];
}

// ジョブポイントギフト: data/job_gifts.json の SkillchainBonus エントリをミラー。
// 累計 JP 閾値ごとに value を加算 (累積、最大 +8%)。
// SAM / DNC のみ実装。
const SCB_GIFT_TIERS = {
    Sam: [[150, 2], [450, 2], [1125, 2], [2000, 2]],
    Dnc: [[150, 2], [450, 2], [1125, 2], [2000, 2]],
};
function skillchainBonusGift(job, totalJp) {
    const tiers = SCB_GIFT_TIERS[job] || [];
    let sum = 0;
    for (const [thr, val] of tiers) {
        if (totalJp >= thr) sum += val;
        else break;
    }
    return sum;
}

let pass = 0;
let fail = 0;
function check(label, actual, expected) {
    try {
        assert.strictEqual(actual, expected, `${label}: expected ${expected}, got ${actual}`);
        console.log(`  PASS  ${label} = ${actual}`);
        pass++;
    } catch (e) {
        console.log(`  FAIL  ${label}: ${e.message}`);
        fail++;
    }
}

console.log('=== Source 1: 装備の "連携ダメージ" オーグメント (ムパカキャップ Default rank 30) ===');
{
    const itemId = 23758; // ムパカキャップ
    const item = itemById[itemId];
    assert.ok(item, 'ムパカキャップ (id 23758) が items.json に存在しない');
    const augText = getAugmentText(itemId, 0, 30);
    assert.ok(augText, 'ムパカキャップ Default rank 30 のオーグメントテキストが取得できない');
    const augStats = extractAllStats(convertAugmentJaToEn(augText));
    check('Mpaca Cap aug rank 30 → skillchain_bonus', augStats.skillchain_bonus || 0, 15);
}

console.log('\n=== Source 2: 装備の "Skillchain bonus +N" 表記 (C. Palug Hammer) ===');
{
    const palugHammer = items.find((it) => it.en === 'C. Palug Hammer');
    assert.ok(palugHammer, 'C. Palug Hammer が items.json に存在しない');
    const stats = extractAllStats(palugHammer.description_en);
    check('C. Palug Hammer → skillchain_bonus', stats.skillchain_bonus || 0, 7);
}

console.log('\n=== Source 3: ジョブ特性 (https://wiki.ffo.jp/html/20337.html) ===');
{
    // SAM
    check('SAM Lv77 → 0 (未習得)', skillchainBonusJobTrait('Sam', 77), 0);
    check('SAM Lv78 → +8 (rank 1)', skillchainBonusJobTrait('Sam', 78), 8);
    check('SAM Lv88 → +12 (rank 2)', skillchainBonusJobTrait('Sam', 88), 12);
    check('SAM Lv98 → +16 (rank 3)', skillchainBonusJobTrait('Sam', 98), 16);
    check('SAM Lv99 → +16 (rank 3)', skillchainBonusJobTrait('Sam', 99), 16);
    // MNK/NIN
    check('MNK Lv85 → +8',  skillchainBonusJobTrait('Mnk', 85), 8);
    check('MNK Lv95 → +12', skillchainBonusJobTrait('Mnk', 95), 12);
    check('NIN Lv95 → +12', skillchainBonusJobTrait('Nin', 95), 12);
    // BLU (Lv99 で 3 ランク同時習得)
    check('BLU Lv99 → +23 (rank 5)', skillchainBonusJobTrait('Blu', 99), 23);
    // DNC
    check('DNC Lv99 → +23 (rank 5)', skillchainBonusJobTrait('Dnc', 99), 23);
    // 習得しないジョブ
    check('WAR Lv99 → 0', skillchainBonusJobTrait('War', 99), 0);
    check('PLD Lv99 → 0', skillchainBonusJobTrait('Pld', 99), 0);
}

console.log('\n=== Source 4: ジョブポイントギフト ===');
{
    // SAM/DNC ギフト: 150/450/1125/2000 JP で +2% ずつ累積、最大 +8%
    check('SAM 0 JP → 0',         skillchainBonusGift('Sam', 0), 0);
    check('SAM 149 JP → 0',       skillchainBonusGift('Sam', 149), 0);
    check('SAM 150 JP → +2',      skillchainBonusGift('Sam', 150), 2);
    check('SAM 450 JP → +4',      skillchainBonusGift('Sam', 450), 4);
    check('SAM 1125 JP → +6',     skillchainBonusGift('Sam', 1125), 6);
    check('SAM 2000 JP → +8',     skillchainBonusGift('Sam', 2000), 8);
    check('SAM 2100 JP (全振り) → +8', skillchainBonusGift('Sam', 2100), 8);
    check('DNC 2100 JP → +8',     skillchainBonusGift('Dnc', 2100), 8);
    // ギフトを習得しないジョブ
    check('WAR 2100 JP → 0', skillchainBonusGift('War', 2100), 0);
}

console.log('\n=== 統合: 4 ソースが正しく合算される ===');
{
    // シナリオ: SAM Lv99 JP 全振り (2100) が C. Palug Hammer (装備 +7)
    //          + ムパカキャップ rank 30 (オーグ +15) を装備
    // 期待値: 装備 22 + 特性 16 + ギフト 8 = 46
    const palug = items.find((it) => it.en === 'C. Palug Hammer');
    const mpacaAug = getAugmentText(23758, 0, 30);
    const equipStatsArray = [
        extractAllStats(palug.description_en),
        extractAllStats(convertAugmentJaToEn(mpacaAug)),
    ];
    const equipTotal = sumStats(equipStatsArray);
    check('装備合計 = 7 + 15', equipTotal.skillchain_bonus, 22);

    const traitTotal = skillchainBonusJobTrait('Sam', 99);
    check('SAM Lv99 特性 = 16', traitTotal, 16);

    const giftTotal = skillchainBonusGift('Sam', 2100);
    check('SAM JP全振り ギフト = 8', giftTotal, 8);

    const grandTotal = (equipTotal.skillchain_bonus || 0) + traitTotal + giftTotal;
    check('総合 = 装備22 + 特性16 + ギフト8 = 46', grandTotal, 46);
}

console.log(`\n${pass} passed, ${fail} failed`);
if (fail > 0) process.exit(1);
