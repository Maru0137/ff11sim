import { describe, it, expect } from 'vitest';
import { moveItem } from './reorder';

describe('moveItem', () => {
    const base = ['a', 'b', 'c', 'd'];

    it('後ろへ動かすと、落とした先の位置に入る', () => {
        expect(moveItem(base, 0, 2)).toEqual(['b', 'c', 'a', 'd']);
    });

    it('前へ動かすと、落とした先を押し下げる', () => {
        expect(moveItem(base, 3, 1)).toEqual(['a', 'd', 'b', 'c']);
    });

    it('隣へ 1 つ動かせる', () => {
        expect(moveItem(base, 1, 2)).toEqual(['a', 'c', 'b', 'd']);
    });

    it('末尾へ動かせる', () => {
        expect(moveItem(base, 0, 3)).toEqual(['b', 'c', 'd', 'a']);
    });

    it('同じ位置なら元の配列をそのまま返す', () => {
        expect(moveItem(base, 2, 2)).toBe(base);
    });

    it('範囲外の添字なら元の配列をそのまま返す', () => {
        expect(moveItem(base, -1, 2)).toBe(base);
        expect(moveItem(base, 0, 9)).toBe(base);
        expect(moveItem(base, 9, 0)).toBe(base);
    });

    it('元の配列を書き換えない', () => {
        const src = [...base];
        moveItem(src, 0, 3);
        expect(src).toEqual(base);
    });

    it('要素が 1 つでも壊れない', () => {
        expect(moveItem(['a'], 0, 0)).toEqual(['a']);
    });
});
