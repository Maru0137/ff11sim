// テンプレート定義 (template-defs.ts) の整合性テスト。
// 宣言的データ化で JSX の型チェックが効かなくなった分をここで担保する。
// catalog は constants (top-level await fetch) に届くため必要な定数のみモックする。
import { describe, it, expect, vi } from 'vitest';

vi.mock('../constants', () => ({
    ALL_SKILL_KEYS: [],
}));

const { TEMPLATE_PROPSET_DEFS, TEMPLATE_PROPSET_GROUPS } = await import('./template-defs');
const { TEMPLATE_ITEM_IDS } = await import('../propsets/catalog');

describe('TEMPLATE_PROPSET_DEFS', () => {
    it('id が一意で、グループが定義済みである', () => {
        const ids = TEMPLATE_PROPSET_DEFS.map((d) => d.id);
        expect(new Set(ids).size).toBe(ids.length);
        const groups = new Set<string>(TEMPLATE_PROPSET_GROUPS);
        for (const def of TEMPLATE_PROPSET_DEFS) {
            expect(groups.has(def.group), def.id).toBe(true);
        }
    });

    it('全テーブルで各行のセル数が列数と一致する', () => {
        for (const def of TEMPLATE_PROPSET_DEFS) {
            for (const table of def.tables) {
                for (const row of table.rows) {
                    expect(row.cells.length, `${def.id}: ${table.columns.map((c) => c.label)}`)
                        .toBe(table.columns.length);
                }
            }
        }
    });

    it('valueId (DOM id) が全テンプレートを通して一意である', () => {
        const seen = new Set<string>();
        for (const def of TEMPLATE_PROPSET_DEFS) {
            for (const table of def.tables) {
                for (const row of table.rows) {
                    for (const id of row.cells) {
                        if (id === null) continue;
                        expect(seen.has(id), `duplicate valueId: ${id} (${def.id})`).toBe(false);
                        seen.add(id);
                    }
                }
            }
        }
    });

    it('TEMPLATE_ITEM_IDS (複製・内訳用の対応) と 1:1 で揃っている', () => {
        const defIds = new Set(TEMPLATE_PROPSET_DEFS.map((d) => d.id));
        for (const key of Object.keys(TEMPLATE_ITEM_IDS)) {
            expect(defIds.has(key), `TEMPLATE_ITEM_IDS の ${key} に対応するテンプレートがない`)
                .toBe(true);
        }
        for (const id of defIds) {
            expect(TEMPLATE_ITEM_IDS[id], `${id} の TEMPLATE_ITEM_IDS が未定義`).toBeDefined();
        }
    });
});
