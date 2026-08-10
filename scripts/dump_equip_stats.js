#!/usr/bin/env node
//
// JS 版 (web/js/equip-stats.js) の抽出結果を JSON で出力する。
//
// Rust への移植 (docs/adr/0010) で、JS をリファレンス実装として扱うためのハーネス。
// 期待値を思い込みで書くと移植の検証にならないため、実際に JS を動かした結果を
// テストの期待値として使う。
//
// 使い方:
//   node scripts/dump_equip_stats.js --id 27410 27151
//   node scripts/dump_equip_stats.js --text 'Attack+5\nAccuracy+3'
//   node scripts/dump_equip_stats.js --all > /tmp/js-stats.json
//
// 出力: { "<id or 'text'>": { name, description_en, stats, skills } }
// stats / skills はいずれも非ゼロのキーのみ (JS の挙動どおり)。
//
// 前提: scripts/build_web_data.sh を実行済みで web/data/items.json が存在すること。

const fs = require('fs');
const path = require('path');

const { extractAllStats, extractSkillBonuses } = require('../web/js/equip-stats.js');

const ITEMS_JSON = path.join(__dirname, '..', 'web', 'data', 'items.json');

function loadItems() {
    if (!fs.existsSync(ITEMS_JSON)) {
        console.error(`ERROR: ${ITEMS_JSON} が無い。先に scripts/build_web_data.sh を実行する。`);
        process.exit(1);
    }
    return JSON.parse(fs.readFileSync(ITEMS_JSON, 'utf8')).items;
}

function extract(descriptionEn) {
    return {
        stats: extractAllStats(descriptionEn),
        skills: extractSkillBonuses(descriptionEn),
    };
}

function main() {
    const argv = process.argv.slice(2);
    const mode = argv[0];
    const rest = argv.slice(1);
    const out = {};

    if (mode === '--text') {
        const text = rest.join(' ');
        out.text = { description_en: text, ...extract(text) };
    } else if (mode === '--id') {
        const items = loadItems();
        const byId = Object.fromEntries(items.map((it) => [String(it.id), it]));
        for (const id of rest) {
            const it = byId[id];
            if (!it) {
                console.error(`ERROR: id ${id} が items.json に無い`);
                process.exit(1);
            }
            out[id] = {
                name: it.en,
                name_ja: it.ja,
                description_en: it.description_en ?? null,
                ...extract(it.description_en),
            };
        }
    } else if (mode === '--all') {
        // 全件。Rust 実装との一括突き合わせ用。
        for (const it of loadItems()) {
            out[it.id] = extract(it.description_en);
        }
    } else {
        console.error('usage: dump_equip_stats.js (--id <id>... | --text <text> | --all)');
        process.exit(2);
    }

    process.stdout.write(JSON.stringify(out, null, 2) + '\n');
}

main();
