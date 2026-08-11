// 装備セットのステータス・スキルボーナス集計。DOM に依存しない純関数。
//
// 抽出・合算の実体は Rust 側 (docs/adr/0009, docs/adr/0010)。この層は
// アイテム説明文・オーグメント・カスタム説明の 3 入力を WASM の抽出に流し、
// スロット別バケツに振り分けるだけ。既知の互換挙動については
// docs/tech-debt/equip-stats-js-quirks.md を参照 (挙動を変えないこと)。
import {
    get_item_by_id, extract_all_stats, extract_skill_bonuses,
    sum_stats, empty_stats, isItemsLoaded,
} from './wasm.js';
import { SKILL_KEYS_WEAPON } from './constants.js';
import { convertAugmentJaToEn } from '../src/utils';
import { getAugmentText } from '../src/augments';

// 武器スキルのキー集合。以前は equip-stats.js が同じ一覧を自前で持っていたが、
// 二重管理になるので data/skills.json (category: Weapon) 由来の定義から作る。
const WEAPON_SKILL_KEYS = new Set(SKILL_KEYS_WEAPON.map(([key]) => key));

export function calculateEquipSetBonuses(equipSet) {
    if (!equipSet || !equipSet.slots || !isItemsLoaded()) {
        return empty_stats();
    }
    const slots = equipSet.slots;

    const statsArray = [];
    // 武器スロット (main/sub/range) ごとの装備合計を別途保持。
    // 一部の装備ステ (例: 魔命スキル) は装備中スロットに依存して扱いが変わるため、
    // UI で「メイン枠のみ表示」のような出し分けに使う。
    const slotStatsBuckets = { main: [], sub: [], ranged: [] };
    // スキルボーナスをスロット別に集計:
    // 武器スロット(main/sub/range)装備の「武器スキル」ボーナスはそのスロット専用。
    // それ以外（非武器スロット装備すべて、武器スロット装備の非武器スキル）は全スロット共通。
    const skillBonusBuckets = {
        main: {}, sub: {}, ranged: {}, global: {}
    };
    const addSkillBonuses = (slotKey, bonuses) => {
        if (!bonuses) return;
        const targetSlot = slotKey === 'range' ? 'ranged' : slotKey;
        const isWeaponSlot = (slotKey === 'main' || slotKey === 'sub' || slotKey === 'range');
        for (const [k, v] of Object.entries(bonuses)) {
            if (!v) continue;
            if (isWeaponSlot && WEAPON_SKILL_KEYS.has(k)) {
                skillBonusBuckets[targetSlot][k] = (skillBonusBuckets[targetSlot][k] || 0) + v;
            } else {
                skillBonusBuckets.global[k] = (skillBonusBuckets.global[k] || 0) + v;
            }
        }
    };

    for (const slotKey of Object.keys(slots)) {
        const slotData = slots[slotKey];
        if (!slotData) continue;

        const item = get_item_by_id(slotData.item_id);

        const targetWeaponBucket = slotKey === 'range' ? 'ranged' : slotKey;
        const isWeaponSlot = (slotKey === 'main' || slotKey === 'sub' || slotKey === 'range');

        if (item && item.description_en) {
            const stats = extract_all_stats(item.description_en);
            statsArray.push(stats);
            if (isWeaponSlot) slotStatsBuckets[targetWeaponBucket].push(stats);
            addSkillBonuses(slotKey, extract_skill_bonuses(item.description_en));
        }

        const augText = getAugmentText(slotData);
        if (augText) {
            const augEn = convertAugmentJaToEn(augText);
            const augStats = extract_all_stats(augEn);
            statsArray.push(augStats);
            if (isWeaponSlot) slotStatsBuckets[targetWeaponBucket].push(augStats);
            addSkillBonuses(slotKey, extract_skill_bonuses(augEn));
        }

        if (slotData.custom_description) {
            const customEn = convertAugmentJaToEn(slotData.custom_description);
            const customStats = extract_all_stats(customEn);
            statsArray.push(customStats);
            if (isWeaponSlot) slotStatsBuckets[targetWeaponBucket].push(customStats);
            addSkillBonuses(slotKey, extract_skill_bonuses(customEn));
        }
    }

    const result = sum_stats(statsArray);
    result.skill_bonus_main = skillBonusBuckets.main;
    result.skill_bonus_sub = skillBonusBuckets.sub;
    result.skill_bonus_ranged = skillBonusBuckets.ranged;
    result.skill_bonus_global = skillBonusBuckets.global;
    // 武器スロット別装備合計 (魔命スキルなどスロット依存表示用)
    result.slot_stats = {
        main: sum_stats(slotStatsBuckets.main),
        sub: sum_stats(slotStatsBuckets.sub),
        ranged: sum_stats(slotStatsBuckets.ranged),
    };
    return result;
}
