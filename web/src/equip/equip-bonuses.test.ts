// calculateEquipSetBonuses の実 WASM 込みテスト。
// docs/tech-debt/inline-script-monolith.md の残課題 (WEAPON_SKILL_KEYS 事故の
// 現場だった集計ロジックにユニットテストが無い) への対応。
//
// - WASM は node では file URL を fetch できないため、.wasm のバイト列を
//   読んで initWasmRuntime に渡す (web/pkg のビルドが前提。CI では
//   Build WASM ステップの後に実行される)
// - constants は top-level await fetch を含むため必要な定数のみモックする
import { describe, it, expect, beforeAll, vi } from 'vitest';
import { readFileSync } from 'node:fs';

vi.mock('../constants', () => ({
    // equip-bonuses: 武器スキル判定用 (SkillKind 名は Rust 側と一致させる)
    SKILL_KEYS_WEAPON: [
        ['HandToHand', '格闘'], ['Dagger', '短剣'], ['Sword', '片手剣'],
        ['GreatSword', '両手剣'], ['Axe', '片手斧'], ['GreatAxe', '両手斧'],
        ['Scythe', '両手鎌'], ['Polearm', '両手槍'], ['Katana', '片手刀'],
        ['GreatKatana', '両手刀'], ['Club', '片手棍'], ['Staff', '両手棍'],
        ['Archery', '弓術'], ['Marksmanship', '射撃'], ['Throwing', '投てき'],
    ],
    // utils (convertAugmentJaToEn) 用: テストで使う変換のみ
    AUGMENT_JA_TO_EN: [
        ['命中', 'Accuracy'],
        ['攻', 'Attack'],
    ],
    JP_CATEGORY_COUNT: 10,
    JP_MAX_RANK: 20,
    JOB_MERIT_GROUP_SIZE: 8,
    JOB_MERIT_CATEGORIES: {},
    JOB_MERIT_PLACEHOLDER_RE: /^カテゴリ\s*\d+$/,
}));

const { calculateEquipSetBonuses } = await import('./equip-bonuses');

beforeAll(async () => {
    const { initWasmRuntime } = await import('../../js/wasm.js');
    const wasmBytes = readFileSync(new URL('../../pkg/ff11sim_bg.wasm', import.meta.url));
    await initWasmRuntime(wasmBytes);
});

describe('calculateEquipSetBonuses', () => {
    it('スロットなし・items 未ロードでなければ空ステータスを返す', () => {
        const result = calculateEquipSetBonuses(null);
        expect(result.str).toBe(0);
        expect(result.accuracy).toBe(0);
    });

    it('カスタム説明が JA→EN 変換されて抽出・合算される', () => {
        // item_id 0 は存在しない → アイテム説明の抽出はスキップされ、
        // カスタム説明のみが集計される
        const result = calculateEquipSetBonuses({
            slots: { body: { item_id: 0, custom_description: 'STR+5 命中+10' } },
        });
        expect(result.str).toBe(5);
        expect(result.accuracy).toBe(10);
    });

    it('複数スロットのカスタム説明が合算される', () => {
        const result = calculateEquipSetBonuses({
            slots: {
                body: { item_id: 0, custom_description: 'STR+5' },
                legs: { item_id: 0, custom_description: 'STR+3 攻+20' },
            },
        });
        expect(result.str).toBe(8);
        expect(result.attack).toBe(20);
    });

    it('武器スロット装備の武器スキルボーナスはそのスロット専用バケツに入る', () => {
        // 21071 = C. Palug Hammer。description_en に "Club skill +N" を含む
        // (スモークテストと同じ検証用アイテム)
        const result = calculateEquipSetBonuses({
            slots: { main: { item_id: 21071 } },
        });
        expect(result.skill_bonus_main.Club).toBeGreaterThan(0);
        // 非武器スロットに同じ装備を置いた場合は global 側に入る
        const resultBody = calculateEquipSetBonuses({
            slots: { neck: { item_id: 21071 } },
        });
        expect(resultBody.skill_bonus_main.Club).toBeUndefined();
        expect(resultBody.skill_bonus_global.Club).toBeGreaterThan(0);
    });

    it('武器スロット別の装備合計 (slot_stats) が返る', () => {
        const result = calculateEquipSetBonuses({
            slots: {
                main: { item_id: 0, custom_description: '命中+10' },
                body: { item_id: 0, custom_description: '命中+5' },
            },
        });
        // 全体合計にはどちらも入り、main バケツには main スロット分のみ入る
        expect(result.accuracy).toBe(15);
        expect(result.slot_stats.main.accuracy).toBe(10);
    });

    it('全スロット別の装備合計 (per_slot_stats) が返り、総和が全体と一致する', () => {
        const result = calculateEquipSetBonuses({
            slots: {
                main: { item_id: 0, custom_description: '命中+10' },
                body: { item_id: 0, custom_description: '命中+5 STR+3' },
                legs: null,
            },
        });
        // 生のスロットキーで各スロット分のみ持つ
        expect(result.per_slot_stats.main.accuracy).toBe(10);
        expect(result.per_slot_stats.body.accuracy).toBe(5);
        expect(result.per_slot_stats.body.str).toBe(3);
        // 装備のないスロットは含まない
        expect(result.per_slot_stats.legs).toBeUndefined();
        // スロット別の総和 == 全体合計
        const perSlot = result.per_slot_stats as Record<string, { accuracy?: number }>;
        const sum = Object.values(perSlot).reduce((acc, s) => acc + (s.accuracy || 0), 0);
        expect(sum).toBe(result.accuracy);
    });
});
