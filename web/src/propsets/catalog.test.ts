// カタログ定義の整合性テスト。
// format.ts 経由で constants (top-level await fetch を含む) に届くため、
// 必要な定数のみモックする (equip-bonuses.test.ts と同じ理由)。
import { describe, it, expect, vi } from 'vitest';

vi.mock('../constants', () => ({
    ALL_SKILL_KEYS: [],
}));

const { BUILTIN_PROPERTY_ITEMS, BUILTIN_ITEM_BY_ID, PROPERTY_CATEGORIES, TEMPLATE_ITEM_IDS } =
    await import('./catalog');
const { USER_ITEM_PREFIX } = await import('./types');

describe('BUILTIN_PROPERTY_ITEMS', () => {
    it('id が一意である', () => {
        const ids = BUILTIN_PROPERTY_ITEMS.map((i) => i.id);
        expect(new Set(ids).size).toBe(ids.length);
    });

    it('id が user:/template: プレフィクスと衝突しない', () => {
        for (const { id } of BUILTIN_PROPERTY_ITEMS) {
            expect(id.startsWith(USER_ITEM_PREFIX)).toBe(false);
            expect(id.startsWith('template:')).toBe(false);
        }
    });

    it('ラベルが空でなくカテゴリが定義済みである', () => {
        const categories = new Set<string>(PROPERTY_CATEGORIES);
        for (const item of BUILTIN_PROPERTY_ITEMS) {
            expect(item.label.length).toBeGreaterThan(0);
            expect(categories.has(item.category)).toBe(true);
        }
    });

    it('基本 9 項目と左テーブル既出項目を含まない', () => {
        const excluded = [
            'hp', 'mp', 'str', 'dex', 'vit', 'agi', 'int', 'mnd', 'chr',
            'def', 'evasion', 'mdef', 'magic_evasion', 'haste',
            'damage_taken', 'physical_damage_taken', 'magic_damage_taken',
        ];
        for (const id of excluded) {
            expect(BUILTIN_ITEM_BY_ID.has(id)).toBe(false);
        }
    });
});

describe('TEMPLATE_ITEM_IDS', () => {
    it('全エントリが実在するカタログ id を参照する', () => {
        for (const [subtabId, itemIds] of Object.entries(TEMPLATE_ITEM_IDS)) {
            for (const id of itemIds) {
                expect(BUILTIN_ITEM_BY_ID.has(id), `${subtabId} -> ${id}`).toBe(true);
            }
        }
    });

    it('エントリ内で id が重複しない', () => {
        for (const itemIds of Object.values(TEMPLATE_ITEM_IDS)) {
            expect(new Set(itemIds).size).toBe(itemIds.length);
        }
    });
});
