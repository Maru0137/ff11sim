// SAM99/WAR50 ML50 JP マックス、童子切安綱A15 + ニャメ全身B30 ほか属性WS構成での
// 装備合計 WSダメ% (および周辺ステ) を検証するテスト。
//
// 実行: node web/test/ws-damage-sam-elemental.test.js
//
// 目的:
//   ・WSダメの期待値とUI表示の差分を究明
//   ・どの装備からいくつ拾われているか、フォシャ潜在 / 童子切照破ダメ 等の扱い
//     が想定どおりかを breakdown で確認
//
// 装備セット:
//   main:   童子切安綱  Default rank 15
//   sub:    ウトゥグリップ
//   ammo:   ノブキエリ
//   head:   ニャメヘルム  Type:B rank 30
//   neck:   フォシャゴルゲット
//   ear1:   胡蝶のイヤリング (custom: 魔攻+4 TPボーナス+250)
//   ear2:   フリオミシピアス
//   body:   ニャメメイル  Type:B rank 30
//   hands:  ニャメガントレ Type:B rank 30
//   ring1:  コーネリアリング
//   ring2:  エパミノダスリング
//   back:   スメルトリオマント (custom: STR+30 命中+20 攻+20 ウェポンスキルのダメージ+10% 被物理ダメージ-10%)
//   waist:  オルペウスサッシュ
//   legs:   ニャメフランチャ Type:B rank 30
//   feet:   ニャメソルレット Type:B rank 30

const fs = require('fs');
const path = require('path');
const assert = require('assert');

const { extractAllStats, getEmptyStats, sumStats } = require('../js/equip-stats.js');

const dataDir = path.join(__dirname, '..', 'data');
const items = JSON.parse(fs.readFileSync(path.join(dataDir, 'items.json'), 'utf8')).items;
const augments = JSON.parse(fs.readFileSync(path.join(dataDir, 'augments.json'), 'utf8')).augments;
const itemById = Object.fromEntries(items.map(it => [it.id, it]));

// index.html の AUGMENT_JA_TO_EN と同期（テスト用にコピー）。
// 元側の更新時はこちらも合わせること。
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
    for (const [ja, en] of AUGMENT_JA_TO_EN) {
        result = result.replaceAll(ja, en);
    }
    return result;
}

function getAugmentText(itemId, pathIdx, rank) {
    const info = augments[String(itemId)];
    if (!info) return null;
    const p = info.paths[pathIdx];
    if (!p) return null;
    const r = p.ranks.find(rr => rr.rank === rank);
    return r ? r.text : null;
}

// 装備セット定義
const equipSet = [
    { slot: 'main',  itemId: 21025, augPath: 0, augRank: 15 },                                                            // 童子切安綱 Default rank 15
    { slot: 'sub',   itemId: 22212 },                                                                                     // ウトゥグリップ
    { slot: 'ammo',  itemId: 22281 },                                                                                     // ノブキエリ
    { slot: 'head',  itemId: 23761, augPath: 1, augRank: 30 },                                                            // ニャメヘルム Type:B rank 30
    { slot: 'neck',  itemId: 27510 },                                                                                     // フォシャゴルゲット
    { slot: 'ear1',  itemId: 11697, custom: '魔攻+4 TPボーナス+250' },                                                    // 胡蝶のイヤリング
    { slot: 'ear2',  itemId: 28514 },                                                                                     // フリオミシピアス
    { slot: 'body',  itemId: 23768, augPath: 1, augRank: 30 },                                                            // ニャメメイル Type:B rank 30
    { slot: 'hands', itemId: 23775, augPath: 1, augRank: 30 },                                                            // ニャメガントレ Type:B rank 30
    { slot: 'ring1', itemId: 26227 },                                                                                     // コーネリアリング
    { slot: 'ring2', itemId: 26214 },                                                                                     // エパミノダスリング
    { slot: 'back',  itemId: 26257, custom: 'STR+30 命中+20 攻+20 ウェポンスキルのダメージ+10% 被物理ダメージ-10%' },     // スメルトリオマント (custom)
    { slot: 'waist', itemId: 26359 },                                                                                     // オルペウスサッシュ
    { slot: 'legs',  itemId: 23782, augPath: 1, augRank: 30 },                                                            // ニャメフランチャ Type:B rank 30
    { slot: 'feet',  itemId: 23789, augPath: 1, augRank: 30 },                                                            // ニャメソルレット Type:B rank 30
];

console.log('=== SAM99/WAR50 ML50 JP マックス 装備セット WSダメ breakdown ===\n');

const breakdown = [];
const allStatsArrays = [];

for (const slot of equipSet) {
    const item = itemById[slot.itemId];
    if (!item) {
        console.log(`[ERROR] item not found id=${slot.itemId}`);
        continue;
    }

    const baseStats = item.description_en ? extractAllStats(item.description_en) : getEmptyStats();
    let augStats = getEmptyStats();
    let augTextRaw = null;
    if (slot.augPath != null && slot.augRank != null) {
        augTextRaw = getAugmentText(slot.itemId, slot.augPath, slot.augRank);
        if (augTextRaw) augStats = extractAllStats(convertAugmentJaToEn(augTextRaw));
    }
    let customStats = getEmptyStats();
    if (slot.custom) {
        customStats = extractAllStats(convertAugmentJaToEn(slot.custom));
    }

    const baseWs = baseStats.weapon_skill_damage_pct || 0;
    const augWs = augStats.weapon_skill_damage_pct || 0;
    const customWs = customStats.weapon_skill_damage_pct || 0;

    breakdown.push({
        slot: slot.slot,
        name: item.ja,
        base: baseWs,
        aug: augWs,
        custom: customWs,
        total: baseWs + augWs + customWs,
        augText: augTextRaw,
        customText: slot.custom,
    });

    allStatsArrays.push(baseStats, augStats, customStats);
}

const grand = sumStats(allStatsArrays);

console.log('スロット   装備名                       base  aug  custom  合計  備考');
console.log('-------- ---------------------------- ----  ---- ------  ----  --------------');
for (const b of breakdown) {
    const note = [
        b.augText ? `aug=「${b.augText.replaceAll('\n', ' / ')}」` : '',
        b.customText ? `custom=「${b.customText}」` : '',
    ].filter(Boolean).join(', ');
    console.log(
        `${b.slot.padEnd(8)} ${b.name.padEnd(28)} ${String(b.base + '%').padStart(4)}  ` +
        `${String(b.aug + '%').padStart(4)} ${String(b.custom + '%').padStart(6)}  ${String(b.total + '%').padStart(4)}  ${note}`
    );
}

console.log('\n--- 合計 ---');
console.log(`weapon_skill_damage_pct: ${grand.weapon_skill_damage_pct}%`);
console.log(`tp_bonus:                ${grand.tp_bonus}`);
console.log(`store_tp:                ${grand.store_tp}`);
console.log(`magic_attack:            ${grand.magic_attack}`);
console.log(`magic_accuracy:          ${grand.magic_accuracy}`);
console.log(`magic_damage:            ${grand.magic_damage}`);
console.log(`attack:                  ${grand.attack}`);
console.log(`accuracy:                ${grand.accuracy}`);
console.log(`str:                     ${grand.str}`);
console.log(`double_attack_pct:       ${grand.double_attack_pct}`);
console.log(`physical_damage_taken_pct: ${grand.physical_damage_taken_pct}`);

// 期待値: 装備合計 WSダメ% は下記 9 装備のソースから合算される想定。
// フォシャゴルゲットの WSダメ +10% は "Latent effect:" 配下の潜在発動効果のため常時加算しない。
//   ノブキエリ                  +6%   (常時)
//   ニャメヘルム B30 aug        +11%
//   ニャメメイル B30 aug        +13%
//   ニャメガントレ B30 aug      +11%
//   コーネリアリング            +10%
//   エパミノダスリング          +5%
//   スメルトリオマント custom   +10%
//   ニャメフランチャ B30 aug    +12%
//   ニャメソルレット B30 aug    +11%
//                              ----
//                              89%
const EXPECTED_WS_DAMAGE_PCT = 89;

console.log(`\n期待値: ${EXPECTED_WS_DAMAGE_PCT}%`);
console.log(`実際値: ${grand.weapon_skill_damage_pct}%`);

assert.strictEqual(
    grand.weapon_skill_damage_pct,
    EXPECTED_WS_DAMAGE_PCT,
    `WSダメ% mismatch: expected ${EXPECTED_WS_DAMAGE_PCT}%, got ${grand.weapon_skill_damage_pct}%`,
);
console.log('PASS');
