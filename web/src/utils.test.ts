import { describe, it, expect, vi } from 'vitest';

// constants.js は top-level await で data/*.json を fetch するため
// node 環境ではそのまま import できない。utils が参照する定数のみモックする。
vi.mock('../js/constants.js', () => ({
    JP_CATEGORY_COUNT: 10,
    JP_MAX_RANK: 20,
    JOB_MERIT_GROUP_SIZE: 5,
    JOB_MERIT_CATEGORIES: {
        Sam: {
            group1: ['ストアTP効果アップ', '(空き)', 'その他'],
            group2: [null, 'カテゴリ B'],
        },
    },
    JOB_MERIT_PLACEHOLDER_RE: /^\(空き\)$/,
    AUGMENT_JA_TO_EN: [
        ['飛攻', 'Rng.Atk.'],
        ['命中', 'Accuracy'],
    ],
}));

const {
    jpCategoryCost, jpJobTotal, jpDefaultRanks,
    jobMeritDefaultRanks, jobMeritCategoryName, isJobMeritPlaceholder, samStoreTpIndex,
    formatBonus, formatPctBonus, convertAugmentJaToEn,
} = await import('./utils');

describe('ジョブポイント (JP)', () => {
    it('jpCategoryCost はランク r までの三角数', () => {
        expect(jpCategoryCost(0)).toBe(0);
        expect(jpCategoryCost(1)).toBe(1);
        expect(jpCategoryCost(20)).toBe(210);
    });

    it('jpJobTotal はカテゴリごとのコスト合計', () => {
        expect(jpJobTotal([0, 1, 20])).toBe(0 + 1 + 210);
    });

    it('jpDefaultRanks は全カテゴリ最大ランク', () => {
        expect(jpDefaultRanks()).toEqual(new Array(10).fill(20));
    });
});

describe('ジョブ別メリットポイント', () => {
    it('jobMeritDefaultRanks は全項目 0', () => {
        expect(jobMeritDefaultRanks()).toEqual([0, 0, 0, 0, 0]);
    });

    it('カテゴリ名を返し、未定義はフォールバック名', () => {
        expect(jobMeritCategoryName('Sam', 'group1', 0)).toBe('ストアTP効果アップ');
        expect(jobMeritCategoryName('War', 'group1', 2)).toBe('カテゴリ 3');
    });

    it('プレースホルダ項目を判定する', () => {
        expect(isJobMeritPlaceholder('Sam', 'group1', 1)).toBe(true);
        expect(isJobMeritPlaceholder('Sam', 'group1', 0)).toBe(false);
    });

    it('samStoreTpIndex は「ストアTP」で始まる項目の位置', () => {
        expect(samStoreTpIndex()).toBe(0);
    });
});

describe('数値フォーマッタ', () => {
    it('formatBonus は符号付き、0 は "-"', () => {
        expect(formatBonus(5)).toBe('+5');
        expect(formatBonus(-3)).toBe('-3');
        expect(formatBonus(0)).toBe('-');
        expect(formatBonus(null)).toBe('-');
    });

    it('formatPctBonus は % 付き', () => {
        expect(formatPctBonus(5)).toBe('+5%');
        expect(formatPctBonus(-3)).toBe('-3%');
        expect(formatPctBonus(0)).toBe('-');
    });
});

describe('convertAugmentJaToEn', () => {
    it('変換テーブルの全エントリを置換する', () => {
        expect(convertAugmentJaToEn('飛攻+25 命中+10')).toBe('Rng.Atk.+25 Accuracy+10');
    });

    it('該当なしなら原文のまま', () => {
        expect(convertAugmentJaToEn('STR+5')).toBe('STR+5');
    });
});
