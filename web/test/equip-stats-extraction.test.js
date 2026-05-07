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
