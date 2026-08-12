import { describe, it, expect } from 'vitest';
import { parseHash, viewHash, VIEW_IDS } from './routing';

describe('parseHash', () => {
    it('各ビューの hash を解決する', () => {
        expect(parseHash('#/characters')).toBe('characters');
        expect(parseHash('#/equipsets')).toBe('equipsets');
        expect(parseHash('#/search')).toBe('search');
    });

    it('スラッシュ無しの形式も受け付ける', () => {
        expect(parseHash('#search')).toBe('search');
    });

    it('空・不明な hash は null', () => {
        expect(parseHash('')).toBeNull();
        expect(parseHash('#')).toBeNull();
        expect(parseHash('#/')).toBeNull();
        expect(parseHash('#/unknown')).toBeNull();
        expect(parseHash('#/searching')).toBeNull();
    });

    it('viewHash と往復できる', () => {
        for (const view of VIEW_IDS) {
            expect(parseHash(viewHash(view))).toBe(view);
        }
    });
});
