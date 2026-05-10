// extractAllStats の抽出ロジック検証。
//
// 主な対象:
//   1. Unity Ranking (例: "Unity Ranking: Evasion+15～20") の最大値が
//      ベースの同名ステに合算されること
//   2. Pet/Avatar/Wyvern/Automaton セクションの効果がキャラ側に混入しないこと
//
// 実行: node web/test/equip-stats-extraction.test.js

const fs = require('fs');
const path = require('path');
const assert = require('assert');

const { extractAllStats, extractSkillBonuses } = require('../js/equip-stats.js');

const items = JSON.parse(
    fs.readFileSync(path.join(__dirname, '..', 'data', 'items.json'), 'utf8')
).items;
const itemById = Object.fromEntries(items.map((it) => [it.id, it]));

let pass = 0;
let fail = 0;
function check(label, got, expected) {
    if (got === expected) {
        console.log(`  PASS  ${label}: ${got}`);
        pass++;
    } else {
        console.log(`  FAIL  ${label}: got ${got}, expected ${expected}`);
        fail++;
    }
}
function statsOf(id) {
    const desc = itemById[id]?.description_en;
    assert(desc, `id=${id} not found in items.json`);
    return extractAllStats(desc);
}

console.log('=== Unity Ranking ベース合算 ===');
{
    // ヒポメネソックス+1: Evasion+71 + Unity Ranking: Evasion+15～20 → 91
    const s = statsOf(27410);
    check('ヒポメネソックス+1 evasion (71+20)', s.evasion, 91);
}
{
    // ガズブレスレット+1: Accuracy+31 + Unity Ranking: Accuracy+10～15 → 46
    const s = statsOf(27151);
    check('ガズブレスレット+1 accuracy (31+15)', s.accuracy, 46);
}

console.log('\n=== Pet/Avatar/Wyvern/Automaton セクション除外 ===');
{
    // ナレスカフス: 装備 "Magic Atk. Bonus"+13、Avatar: "Magic Atk. Bonus"+8 → 13
    const s = statsOf(10533);
    check('ナレスカフス magic_attack (avatar 除外)', s.magic_attack, 13);
}
{
    // テチアンカフス+2: 装備 "Magic Atk. Bonus"+7、Avatar: +5 → 7
    const s = statsOf(10531);
    check('テチアンカフス+2 magic_attack (avatar 除外)', s.magic_attack, 7);
}
{
    // パンタンケープ: 装備 Attack+15、Automaton: Attack+15 → 15
    const s = statsOf(16245);
    check('パンタンケープ attack (automaton 除外)', s.attack, 15);
}
{
    // ＰＮトベ+2: 装備 Accuracy+15 Attack+15、Automaton: Accuracy+15 "Store TP"+10 → 15
    const s = statsOf(10687);
    check('ＰＮトベ+2 accuracy (automaton 除外)', s.accuracy, 15);
    check('ＰＮトベ+2 attack', s.attack, 15);
    // Store TP は Automaton 専用なので 0
    check('ＰＮトベ+2 store_tp (automaton のみ → 0)', s.store_tp ?? 0, 0);
}
{
    // モエパパストーン: 装備 Haste+5%、Pet: Haste+5% → 5
    const s = statsOf(10817);
    check('モエパパストーン haste_pct (pet 除外)', s.haste_pct, 5);
}
{
    // ＰＮダスタナ+2: 装備 Haste+4%、Automaton: Haste+4% "Subtle Blow"+5 → 4
    const s = statsOf(10707);
    check('ＰＮダスタナ+2 haste_pct (automaton 除外)', s.haste_pct, 4);
    check('ＰＮダスタナ+2 subtle_blow (automaton のみ → 0)', s.subtle_blow ?? 0, 0);
}
{
    // アスプロピアス (id=26119) ベース description:
    //   "Pet: Accuracy+15\nRanged Accuracy+15\nMagic Accuracy+15"
    // 3 行ともペット用 stats (Pet: 行が折り返しで続いている)。
    // キャラ側に魔命/飛命 +15 が乗るのは augment 経由 (本体には乗らない)。
    const s = statsOf(26119);
    check('アスプロピアス accuracy (Pet 専用 → 0)', s.accuracy ?? 0, 0);
    check('アスプロピアス ranged_accuracy (Pet 専用 → 0)', s.ranged_accuracy ?? 0, 0);
    check('アスプロピアス magic_accuracy (Pet 専用 → 0)', s.magic_accuracy ?? 0, 0);
    check('アスプロピアス hp (キャラ用)', s.hp, 100);
    check('アスプロピアス haste_pct (キャラ用)', s.haste_pct, 5);
}
{
    // メランリング (id=26234) も同様の構造 (Pet: が折り返し続行)
    const s = statsOf(26234);
    check('メランリング accuracy (Pet 専用 → 0)', s.accuracy ?? 0, 0);
    check('メランリング ranged_accuracy (Pet 専用 → 0)', s.ranged_accuracy ?? 0, 0);
    check('メランリング magic_accuracy (Pet 専用 → 0)', s.magic_accuracy ?? 0, 0);
    check('メランリング mp (キャラ用)', s.mp, 30);
    check('メランリング damage_taken_pct (キャラ用)', s.damage_taken_pct, -10);
}
{
    // ＰＩトベ+4 (id=23980) Automaton 折り返し続行の例:
    //   "Automaton: Accuracy+55 Attack+70 Ranged Accuracy+55 Ranged Attack+70
    //    Magic Accuracy+55 \"Store TP\"+15"
    // 2 行目もすべて Automaton 用なのでキャラ集計には混入しない。
    const s = statsOf(23980);
    check('ＰＩトベ+4 magic_accuracy (Automaton 続行除外)', s.magic_accuracy, 45);
    check('ＰＩトベ+4 store_tp (Automaton 専用 → 0)', s.store_tp ?? 0, 0);
}

console.log('\n=== 全魔法スキル一括加算 (Magic skills +N) ===');
{
    // インカンタートルク (id=26016): "Magic skills +10" → 14 種すべてに +10
    const sb = extractSkillBonuses(itemById[26016].description_en);
    const ALL_MAGIC = ['Divine','Healing','Enhancing','Enfeebling','Elemental','Dark',
                       'Summoning','Ninjutsu','Singing','StringInstrument','WindInstrument',
                       'BlueMagic','Geomancy','Handbell'];
    for (const k of ALL_MAGIC) check(`インカンタートルク ${k} (+10)`, sb[k] ?? 0, 10);
}
{
    // スティキニリング+1 (id=26184): "All magic skills +8" → 14 種すべてに +8
    const sb = extractSkillBonuses(itemById[26184].description_en);
    check('スティキニリング+1 Healing (+8)', sb.Healing ?? 0, 8);
    check('スティキニリング+1 Geomancy (+8)', sb.Geomancy ?? 0, 8);
    check('スティキニリング+1 BlueMagic (+8)', sb.BlueMagic ?? 0, 8);
}
{
    // ホクスニトルク (id=26043): "Combat skills +30 / Magic skills +30 / Slow+5%"
    // 戦闘スキル+30 は対象外 (魔法スキル+30 のみ 14 種に加算)
    const sb = extractSkillBonuses(itemById[26043].description_en);
    check('ホクスニトルク Divine (+30)', sb.Divine ?? 0, 30);
    check('ホクスニトルク Ninjutsu (+30)', sb.Ninjutsu ?? 0, 30);
    check('ホクスニトルク Sword (combat skills は未対応 → 0)', sb.Sword ?? 0, 0);
}

console.log('\n=== 属性耐性: 装備のアイコン (private-use Unicode) → "Fire Resistance" 等の正規化 ===');
{
    // フィーバーコラジン (id=10287): description_en に "+30" (耐火+30 を表す)
    const s = statsOf(10287);
    check('フィーバーコラジン resist_fire (アイコン正規化 → +30)', s.resist_fire ?? 0, 30);
}
{
    // キャリアーサッシュ (id=10832): 8 属性すべて +15
    const s = statsOf(10832);
    for (const e of ['fire','ice','wind','earth','lightning','water','light','dark']) {
        check(`キャリアーサッシュ resist_${e} (+15)`, s[`resist_${e}`] ?? 0, 15);
    }
}
{
    // ウェザリンシールド (id=10803): 全属性 -10
    const s = statsOf(10803);
    for (const e of ['fire','ice','wind','earth','lightning','water','light','dark']) {
        check(`ウェザリンシールド resist_${e} (-10)`, s[`resist_${e}`] ?? 0, -10);
    }
}
{
    // イリダルスタッフ (id=18632): "All elemental resistances +15" → 8 属性すべてに +15
    const s = statsOf(18632);
    for (const e of ['fire','ice','wind','earth','lightning','water','light','dark']) {
        check(`イリダルスタッフ resist_${e} (全属性 +15)`, s[`resist_${e}`] ?? 0, 15);
    }
}
{
    // 霊亀棍 (id=21152): "Resist all elements +20" → 8 属性すべてに +20
    const s = statsOf(21152);
    for (const e of ['fire','ice','wind','earth','lightning','water','light','dark']) {
        check(`霊亀棍 resist_${e} (全属性 +20)`, s[`resist_${e}`] ?? 0, 20);
    }
}

console.log('\n=== JA→EN 変換: 個別魔法スキル名の優先 (regression for #28) ===');
{
    // 「強化魔法スキル+N」「弱体魔法スキル+N」等の JA を convertAugmentJaToEn で
    // EN 化する際、'魔法スキル' → 'Magic skills' の汎用パターンが先に発火して
    // "Enhancing magic skill" を "強化Magic skills" に壊し、結果として 14 種すべての
    // 魔法スキルに +N 加算される regression を防ぐ。
    const fs = require('fs');
    const path = require('path');
    const constJs = fs.readFileSync(path.join(__dirname, '..', 'js', 'constants.js'), 'utf8');
    const m = constJs.match(/AUGMENT_JA_TO_EN\s*=\s*(\[[\s\S]*?\n\]);/);
    const AUG_JA_TO_EN = eval(m[1]);
    const jaToEn = (text) => {
        let out = text;
        for (const [ja, en] of AUG_JA_TO_EN) out = out.split(ja).join(en);
        return out;
    };

    // ゴストファイケープ (id=28621) custom: "弱体魔法スキル+10 強化魔法スキル+10 強化魔法効果時間+19%"
    const cust = '弱体魔法スキル+10 強化魔法スキル+10 強化魔法効果時間+19%';
    const en = jaToEn(cust);
    const sb = extractSkillBonuses(en);
    check('ゴストファイケープ custom Enhancing (+10)', sb.Enhancing ?? 0, 10);
    check('ゴストファイケープ custom Enfeebling (+10)', sb.Enfeebling ?? 0, 10);
    // 他の魔法スキルには加算されないこと (汎用 "Magic skills" として誤展開されない)
    check('ゴストファイケープ custom Healing (誤展開されない)', sb.Healing ?? 0, 0);
    check('ゴストファイケープ custom Geomancy (誤展開されない)', sb.Geomancy ?? 0, 0);
    check('ゴストファイケープ custom BlueMagic (誤展開されない)', sb.BlueMagic ?? 0, 0);
}
{
    // 個別魔法スキル名 9 種すべての JA→EN 変換を確認
    const fs = require('fs');
    const path = require('path');
    const constJs = fs.readFileSync(path.join(__dirname, '..', 'js', 'constants.js'), 'utf8');
    const mt = constJs.match(/AUGMENT_JA_TO_EN\s*=\s*(\[[\s\S]*?\n\]);/);
    const AUG_JA_TO_EN = eval(mt[1]);
    const jaToEn = (text) => {
        let out = text;
        for (const [ja, en] of AUG_JA_TO_EN) out = out.split(ja).join(en);
        return out;
    };
    const cases = [
        ['神聖魔法スキル+5', 'Divine'],
        ['回復魔法スキル+5', 'Healing'],
        ['強化魔法スキル+5', 'Enhancing'],
        ['弱体魔法スキル+5', 'Enfeebling'],
        ['精霊魔法スキル+5', 'Elemental'],
        ['暗黒魔法スキル+5', 'Dark'],
        ['召喚魔法スキル+5', 'Summoning'],
        ['青魔法スキル+5', 'BlueMagic'],
        ['風水魔法スキル+5', 'Geomancy'],
    ];
    for (const [ja, key] of cases) {
        const sb = extractSkillBonuses(jaToEn(ja));
        check(`JA→EN ${ja} → ${key} のみ +5`, sb[key] ?? 0, 5);
    }
}

console.log('\n=== 状態異常レジスト: 個別装備抽出 ===');
{
    // 実 items.json データ (id, ailment-key, expected-value)
    const cases = [
        [25788, 'sleep', 90],          // Udug Jacket "Resist Sleep"+90
        [20887, 'paralysis', 50],      // Dacnomania "Resist Paralyze"+50
        [26202, 'bind', 17],           // Shneddick Ring +1 "Resist Bind"+17
        [23759, 'silence', 9],         // Agwu's Cap "Resist Silence"+9
        [23313, 'gravity', 10],        // Pillager's Poulaines +2 "Resist Gravity"+10
        [21569, 'slow', 90],           // Chocobo Knife "Resist Slow"+90
        [20606, 'petrification', 15],  // Anathema Harpe "Resist Petrify"+15 (短縮形)
        [26029, 'stun', 10],           // Anu Torque "Resist Stun"+10
        [21787, 'poison', 10],         // Poison Axe +1 "Resist Poison"+10
        [26245, 'charm', 15],          // Solemnity Cape "Resist Charm"+15
        [21034, 'amnesia', 25],        // Kunimune "Resist Amnesia"+25
        [20754, 'death', 15],          // Malfeasance: Resistance against "Death" +15
        [26973, 'death', 15],          // Samnuha Coat: "Death" resistance +15
        [28330, 'terror', 30],         // Founder's Greaves: Terror resistance +30
    ];
    for (const [id, key, expected] of cases) {
        const s = statsOf(id);
        check(`id=${id} resist_${key} (+${expected})`, s[`resist_${key}`] ?? 0, expected);
    }
}
{
    // items.json に該当装備がない 3 種 (blind/curse/virus) は合成 description で確認
    const synth = '"Resist Blind"+5 "Resist Curse"+8\n"Resist Virus"+12';
    const s = extractAllStats(synth);
    check('合成 "Resist Blind"+5', s.resist_blind ?? 0, 5);
    check('合成 "Resist Curse"+8', s.resist_curse ?? 0, 8);
    check('合成 "Resist Virus"+12', s.resist_virus ?? 0, 12);
}
{
    // 引用符なし表記 (custom_description で生 augment 文字列を書いた場合のフォールバック)
    const s = extractAllStats('Resist Slow+45');
    check('引用符なし Resist Slow+45', s.resist_slow ?? 0, 45);
}

console.log('\n=== 状態異常レジスト: 全状態異常レジスト+N (デス除く 15 種) ===');
{
    // Staunch Tathlum +1 (id=22279): "Resistance to all status ailments +11"
    const s = statsOf(22279);
    const expected15 = ['sleep','paralysis','bind','silence','gravity','slow',
                        'petrification','stun','poison','charm','blind','curse',
                        'virus','amnesia','terror'];
    for (const st of expected15) {
        check(`Staunch Tathlum+1 resist_${st} (+11)`, s[`resist_${st}`] ?? 0, 11);
    }
    check('Staunch Tathlum+1 resist_death (除外 → 0)', s.resist_death ?? 0, 0);
}
{
    // Volte Jupon (id=23717): "Resistance to all status ailments +20" (装備に他のステも多数)
    const s = statsOf(23717);
    check('Volte Jupon resist_sleep (+20)', s.resist_sleep ?? 0, 20);
    check('Volte Jupon resist_terror (+20)', s.resist_terror ?? 0, 20);
    check('Volte Jupon resist_death (除外 → 0)', s.resist_death ?? 0, 0);
}
{
    // 別表記: "All status ailment resistance +5"
    const s = extractAllStats('All status ailment resistance +5');
    check('"All status ailment resistance +5" sleep', s.resist_sleep ?? 0, 5);
    check('"All status ailment resistance +5" death (除外)', s.resist_death ?? 0, 0);
}
{
    // 個別 + 全状態異常 が同居する場合は両方加算される (実装上 result[k] += allStatusResist)
    const s = extractAllStats('"Resist Sleep"+5 Resistance to all status ailments +10');
    check('個別+全 sleep (5+10=15)', s.resist_sleep ?? 0, 15);
    check('個別+全 paralysis (0+10=10)', s.resist_paralysis ?? 0, 10);
    check('個別+全 death (除外 → 0)', s.resist_death ?? 0, 0);
}

console.log('\n=== 状態異常レジスト: テナシティ合算 (status-display.combineStatusResist) ===');
{
    // status-display.js は ESM なのでファイルを読み込んで該当ロジックを再現
    const fs = require('fs');
    const path = require('path');
    const sd = fs.readFileSync(path.join(__dirname, '..', 'js', 'status-display.js'), 'utf8');
    // STATUS_RESIST_KEYS と combineStatusResist 関数を抽出
    const keysM = sd.match(/export const STATUS_RESIST_KEYS\s*=\s*(\[[\s\S]*?\]);/);
    const fnM   = sd.match(/export function combineStatusResist\(equipResists, tenacity\) \{([\s\S]*?)\n\}/);
    assert(keysM && fnM, 'status-display.js から combineStatusResist を抽出できない');
    const STATUS_RESIST_KEYS = eval(keysM[1]);
    // 関数本体内では STATUS_RESIST_KEYS を参照しているので、ローカル変数として再定義した上で eval する
    const combineStatusResist = new Function('equipResists', 'tenacity', `
        const STATUS_RESIST_KEYS = ${keysM[1]};
        ${fnM[1]}
    `);

    // (3) テナシティのみ (RUN Lv99 想定 +15)、装備抽出値 0
    const t = combineStatusResist({}, 15);
    for (const st of STATUS_RESIST_KEYS) {
        if (st === 'death') {
            check(`テナシティのみ resist_death (除外 → 0)`, t.death, 0);
        } else {
            check(`テナシティのみ resist_${st} (+15)`, t[st], 15);
        }
    }

    // (4) 装備個別 + 装備全状態異常 + テナシティ の 3 つ複合
    // 装備抽出: "Resist Sleep"+5 + "Resistance to all status ailments +10"
    const equip = extractAllStats('"Resist Sleep"+5 Resistance to all status ailments +10');
    const r = combineStatusResist(equip, 15);
    check('複合 sleep (5+10+15=30)', r.sleep, 30);
    check('複合 paralysis (0+10+15=25)', r.paralysis, 25);
    check('複合 terror (0+10+15=25)', r.terror, 25);
    check('複合 death (装備抽出 0、テナシティ除外 → 0)', r.death, 0);

    // テナシティ 0 (テナシティ未習得) の場合は装備値のみ
    const r0 = combineStatusResist(equip, 0);
    check('テナシティ 0 sleep (5+10=15)', r0.sleep, 15);
    check('テナシティ 0 paralysis (0+10=10)', r0.paralysis, 10);
    check('テナシティ 0 death (0)', r0.death, 0);
}

console.log('\n=== 既存挙動の維持 (回帰チェック) ===');
{
    // ニビルナイフ (id=20600): 既存の単一ステ抽出が壊れないこと
    const s = statsOf(20600);
    check('ニビルナイフ str (該当無し → undefined)', s.str ?? 0, 0);
    check('ニビルナイフ dex (+5)', s.dex, 5);
    check('ニビルナイフ agi (+5)', s.agi, 5);
    check('ニビルナイフ chr (+5)', s.chr, 5);
    check('ニビルナイフ evasion (+29)', s.evasion, 29);
    // 単一魔法スキル装備が誤って 14 種にばらまかれないこと
    const sb = extractSkillBonuses(itemById[20600].description_en);
    check('ニビルナイフ Dagger skill (+242 既存)', sb.Dagger ?? 0, 242);
    check('ニビルナイフ Healing (魔法スキル+対象外 → 0)', sb.Healing ?? 0, 0);
}

console.log('');
console.log(`${pass} passed, ${fail} failed`);
process.exit(fail > 0 ? 1 : 0);
