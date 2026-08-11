import { describe, it, expect } from 'vitest';
import { jobsToKanji, slotsToJa, highlightStat } from './format';

describe('jobsToKanji', () => {
    it('空・未指定は "-"', () => {
        expect(jobsToKanji([])).toBe('-');
        expect(jobsToKanji(undefined)).toBe('-');
    });

    it('22 ジョブは All Jobs', () => {
        const all = [
            'WAR', 'MNK', 'WHM', 'BLM', 'RDM', 'THF', 'PLD', 'DRK', 'BST', 'BRD', 'RNG',
            'SAM', 'NIN', 'DRG', 'SMN', 'BLU', 'COR', 'PUP', 'DNC', 'SCH', 'GEO', 'RUN',
        ];
        expect(jobsToKanji(all)).toBe('All Jobs');
    });

    it('略記へ変換し、未知のキーはそのまま', () => {
        expect(jobsToKanji(['WAR', 'SAM'])).toBe('戦侍');
        expect(jobsToKanji(['XXX'])).toBe('XXX');
    });
});

describe('slotsToJa', () => {
    it('空・未指定は "-"', () => {
        expect(slotsToJa([])).toBe('-');
        expect(slotsToJa(undefined)).toBe('-');
    });

    it('同じ表示名 (耳/指) は重複排除される', () => {
        expect(slotsToJa(['ear1', 'ear2'])).toBe('耳');
        expect(slotsToJa(['ring1', 'ring2', 'back'])).toBe('指 背');
    });
});

describe('highlightStat', () => {
    it('"STR+5" 形式をハイライトする', () => {
        expect(highlightStat('STR+5 DEX+3', 'STR')).toBe(
            '<span class="stat-highlight">STR+5</span> DEX+3'
        );
    });

    it('"DEF:77" 形式 (コロン区切り) をハイライトする', () => {
        expect(highlightStat('防 DEF:77', 'DEF')).toBe(
            '防 <span class="stat-highlight">DEF:77</span>'
        );
    });

    it('全角英数を正規化してマッチし、元の表記のまま囲む', () => {
        // 全角 "ＳＴＲ＋５" にも STR で一致し、囲む文字列は原文
        expect(highlightStat('ＳＴＲ＋５', 'STR')).toBe(
            '<span class="stat-highlight">ＳＴＲ＋５</span>'
        );
    });

    it('日本語ステータス名 (攻) にも一致する', () => {
        expect(highlightStat('攻+20 命中+10', '攻')).toBe(
            '<span class="stat-highlight">攻+20</span> 命中+10'
        );
    });

    it('一致しなければ原文のまま', () => {
        expect(highlightStat('回避+10', 'STR')).toBe('回避+10');
    });

    it('説明文または対象が空なら原文 (または空文字) を返す', () => {
        expect(highlightStat('', 'STR')).toBe('');
        expect(highlightStat('STR+5', '')).toBe('STR+5');
    });
});
