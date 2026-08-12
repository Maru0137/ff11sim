import { describe, it, expect } from 'vitest';
import { equipSummary } from './equip-summary';

describe('equipSummary', () => {
    it('リテラル "\\n" (説明文) を空白にして 1 行にする', () => {
        expect(equipSummary({ description: 'D隔+126\\nSTR+18\\n命中+35' })).toBe(
            'D隔+126 STR+18 命中+35'
        );
    });

    it('実改行 (オーグメントテキスト) も空白にする', () => {
        expect(equipSummary({ description: 'D+92', augText: 'STR+15\n攻+30' })).toBe(
            'D+92 STR+15 攻+30'
        );
    });

    it('説明・Aug・カスタムを順に連結する', () => {
        expect(
            equipSummary({ description: 'D+126', augText: 'DEX+15', custom: 'ヘイスト+3%' })
        ).toBe('D+126 DEX+15 ヘイスト+3%');
    });

    it('空の要素は飛ばす', () => {
        expect(equipSummary({ description: '', augText: null, custom: 'STR+5' })).toBe('STR+5');
        expect(equipSummary({})).toBe('');
    });

    it('連続する空白を 1 つに潰す', () => {
        expect(equipSummary({ description: 'D+126  \\n  STR+18' })).toBe('D+126 STR+18');
    });
});
