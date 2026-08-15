// 装備選択時の既定オーグメント選択 (EquipSelectModal.selectItem が使う)。
import { describe, it, expect } from 'vitest';
import { getDefaultAugmentSelection } from './augments';

describe('getDefaultAugmentSelection', () => {
    it('タイプが 1 つなら最高ランクを選ぶ', () => {
        const result = getDefaultAugmentSelection({
            paths: [
                {
                    type: 'Default',
                    ranks: [
                        { rank: 15, text: 'a' },
                        { rank: 30, text: 'b' },
                        { rank: 25, text: 'c' },
                    ],
                },
            ],
        });
        expect(result).toEqual({ path: 0, rank: 30 });
    });

    it('タイプが複数ある場合は選ばない', () => {
        const result = getDefaultAugmentSelection({
            paths: [
                { type: 'A', ranks: [{ rank: 30, text: 'a' }] },
                { type: 'B', ranks: [{ rank: 30, text: 'b' }] },
            ],
        });
        expect(result).toBeNull();
    });

    it('ランクが無い / オーグメント自体が無い場合は選ばない', () => {
        expect(getDefaultAugmentSelection({ paths: [{ type: 'Default' }] })).toBeNull();
        expect(getDefaultAugmentSelection({ paths: [{ type: 'Default', ranks: [] }] })).toBeNull();
        expect(getDefaultAugmentSelection({ paths: [] })).toBeNull();
        expect(getDefaultAugmentSelection(null)).toBeNull();
    });
});
