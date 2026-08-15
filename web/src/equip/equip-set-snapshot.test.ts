// 装備セットの未保存判定 (docs/adr/0020) のスナップショット比較を検証する。
import { describe, it, expect } from 'vitest';
import { equipSetSnapshot } from './equip-set-snapshot';
import type { EquipSlotData } from './equip-store';

const SLOT_KEYS = ['main', 'sub', 'head'];

function item(overrides: Partial<EquipSlotData> = {}): EquipSlotData {
    return { item_id: 100, name_ja: '金剛草牙', skill: 2, ...overrides };
}

function snap(
    slots: Record<string, EquipSlotData | null | undefined>,
    name = 'WS用',
    propsetSelection: string | null = null
) {
    return equipSetSnapshot({ name, slots, propsetSelection }, SLOT_KEYS);
}

describe('equipSetSnapshot', () => {
    it('スロットのキー順が違っても同じ内容なら一致する', () => {
        // 保存レコードの展開順と createEmptySlots の生成順は一致しない
        const a = snap({ main: item(), head: item({ item_id: 200 }) });
        const b = snap({ head: item({ item_id: 200 }), main: item() });
        expect(a).toBe(b);
    });

    it('未設定スロットは null / undefined / 欠落を区別しない', () => {
        expect(snap({ main: item(), sub: null })).toBe(snap({ main: item(), sub: undefined }));
        expect(snap({ main: item(), sub: null })).toBe(snap({ main: item() }));
    });

    it('表示用フィールドの違いは変更とみなさない', () => {
        // 同じ item_id なら name_ja / description_ja は同じ装備の別表現でしかない
        const a = snap({ main: item({ name_ja: '金剛草牙', description_ja: 'DMG:100' }) });
        const b = snap({ main: item({ name_ja: undefined, description_ja: undefined }) });
        expect(a).toBe(b);
    });

    it('装備・オーグメント・カスタム説明の変更を検出する', () => {
        const base = snap({ main: item() });
        expect(snap({ main: item({ item_id: 999 }) })).not.toBe(base);
        expect(snap({ main: item({ aug_path: 0, aug_rank: 3 }) })).not.toBe(base);
        expect(snap({ main: item({ custom_description: 'STR+5' }) })).not.toBe(base);
        expect(snap({})).not.toBe(base);
    });

    it('名前とプロパティセット選択の変更を検出する', () => {
        const base = snap({ main: item() });
        expect(snap({ main: item() }, 'TP用')).not.toBe(base);
        expect(snap({ main: item() }, 'WS用', 'template:subtab-defense')).not.toBe(base);
    });

    it('変更を元に戻せば元のスナップショットに一致する', () => {
        const base = snap({ main: item() });
        const edited = snap({ main: item({ custom_description: '命中+10' }) });
        expect(edited).not.toBe(base);
        expect(snap({ main: item({ custom_description: '' }) })).toBe(base);
    });

    it('slotKeys に無いスロットは比較対象にならない', () => {
        // 未知のキーが混ざっても保存対象のスロットだけで判定する
        expect(snap({ main: item(), unknown: item({ item_id: 1 }) })).toBe(snap({ main: item() }));
    });
});
