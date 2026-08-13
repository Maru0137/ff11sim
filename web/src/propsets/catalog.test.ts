// カタログ定義の整合性テスト。
// format.ts 経由で constants (top-level await fetch を含む) に届くため、
// 必要な定数のみモックする (equip-bonuses.test.ts と同じ理由)。
import { describe, it, expect, vi } from 'vitest';

vi.mock('../constants', () => ({
    ALL_SKILL_KEYS: [],
}));

const {
    BUILTIN_PROPERTY_ITEMS,
    BUILTIN_ITEM_BY_ID,
    PROPERTY_CATEGORIES,
    TEMPLATE_ITEM_IDS,
    BREAKDOWN_UNSUPPORTED_IDS,
} = await import('./catalog');
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

    it('全項目が breakdown メタを持つか、非対応リストに明示されている', () => {
        // 新項目を追加したら内訳対応可否を必ず判断する (docs/adr/0016)。
        // 対応するなら breakdown メタを、非対応なら BREAKDOWN_UNSUPPORTED_IDS を更新。
        for (const item of BUILTIN_PROPERTY_ITEMS) {
            const supported = item.breakdown !== undefined;
            const declared = BREAKDOWN_UNSUPPORTED_IDS.includes(item.id);
            expect(
                supported !== declared,
                `${item.id}: breakdown メタと非対応リストのどちらか一方が必要`
            ).toBe(true);
        }
    });

    it('breakdown メタの状態異常レジストはデスのみ tenacity 非適用', () => {
        expect(BUILTIN_ITEM_BY_ID.get('resist_death')?.breakdown?.charKey).toBeUndefined();
        expect(BUILTIN_ITEM_BY_ID.get('resist_sleep')?.breakdown?.charKey).toBe('tenacity');
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

describe('魔命スキル/実効魔命 (macc_*)', () => {
    it('resolve が compute.ts の derived 合成値を表示する (0 は "-")', () => {
        const ctx = {
            equip: {},
            totalStats: {},
            derived: {
                magicAttackTotal: 0, magicAccuracyTotal: 0, magicDamageTotal: 0,
                wsDamagePct: 0, skillchainBonusTotal: 0, statusResists: {},
                mainMaccSkill: 228, subMaccSkill: 0,
                maccTotals: { main: 750, ranged: 610, enfeebling: 802, song: 0 },
                trueShotTotal: 0, recycleTotal: 0,
                doubleShotTotal: 0, tripleShotTotal: 0, conserveMpTotal: 0,
            },
        };
        expect(BUILTIN_ITEM_BY_ID.get('macc_skill_main')?.resolve(ctx)).toBe(228);
        expect(BUILTIN_ITEM_BY_ID.get('macc_skill_sub')?.resolve(ctx)).toBe('-');
        expect(BUILTIN_ITEM_BY_ID.get('macc_main')?.resolve(ctx)).toBe(750);
        expect(BUILTIN_ITEM_BY_ID.get('macc_ranged')?.resolve(ctx)).toBe(610);
        expect(BUILTIN_ITEM_BY_ID.get('macc_enfeebling')?.resolve(ctx)).toBe(802);
        expect(BUILTIN_ITEM_BY_ID.get('macc_song')?.resolve(ctx)).toBe('-');
    });

    it('実効魔命はメイン/レンジのみ (サブは存在しない)', () => {
        expect(BUILTIN_ITEM_BY_ID.has('macc_sub')).toBe(false);
    });

    it('魔法系テンプレート全てが魔命スキルと当該種別の実効魔命を含む', () => {
        const magicKinds = [
            'divine', 'healing', 'enhancing', 'enfeebling', 'elemental', 'dark',
            'summoning', 'ninjutsu', 'song', 'blue', 'geomancy',
        ];
        for (const kind of magicKinds) {
            const ids = TEMPLATE_ITEM_IDS[`subtab-magic-${kind}`];
            expect(ids, kind).toBeDefined();
            expect(ids).toContain('macc_skill_main');
            expect(ids).toContain(`macc_${kind}`);
        }
    });

    it('WS 系テンプレート全てが魔命スキルと実効魔命を含む', () => {
        const wsTemplates = [
            'subtab-melee-ws', 'subtab-ranged-ws', 'subtab-elemental-ws',
            'subtab-melee-elemental-ws', 'subtab-ranged-elemental-ws',
        ];
        for (const subtabId of wsTemplates) {
            const ids = TEMPLATE_ITEM_IDS[subtabId];
            expect(ids, subtabId).toBeDefined();
            expect(
                ids.some((id) => id.startsWith('macc_skill_')),
                `${subtabId}: 魔命スキル項目が必要`
            ).toBe(true);
            expect(
                ids.includes('macc_main') || ids.includes('macc_ranged'),
                `${subtabId}: 実効魔命項目が必要`
            ).toBe(true);
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
