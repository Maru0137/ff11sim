import { describe, it, expect } from 'vitest';
import { clampToMax, clampWithinGroup } from './limits';

describe('clampToMax', () => {
    it('範囲内はそのまま返す', () => {
        expect(clampToMax(7, 15)).toBe(7);
        expect(clampToMax(15, 15)).toBe(15);
    });

    it('上限を超えたら上限に丸める', () => {
        expect(clampToMax(999, 433)).toBe(433);
    });

    it('負値と NaN は 0 にする', () => {
        expect(clampToMax(-3, 15)).toBe(0);
        expect(clampToMax(NaN, 15)).toBe(0);
    });

    it('上限 0 の項目は 0 しか通さない', () => {
        // 習得ジョブに該当スキルがないとキャップが 0 になる (skills.rs default_skill_value)
        expect(clampToMax(200, 0)).toBe(0);
    });
});

describe('clampWithinGroup', () => {
    const PER = 5;
    const TOTAL = 10;

    it('グループ計に余裕があれば項目上限まで通す', () => {
        expect(clampWithinGroup([0, 0, 0, 0, 0], 0, 5, PER, TOTAL)).toBe(5);
    });

    it('項目上限を超える入力は項目上限で止まる', () => {
        expect(clampWithinGroup([0, 0, 0, 0, 0], 0, 99, PER, TOTAL)).toBe(5);
    });

    it('他項目の合計を引いた残りで頭打ちになる', () => {
        // 他が 5+3 = 8 なので残りは 2
        expect(clampWithinGroup([0, 5, 3, 0, 0], 0, 5, PER, TOTAL)).toBe(2);
    });

    it('残りが無ければ 0 になる', () => {
        expect(clampWithinGroup([0, 5, 5, 0, 0], 0, 4, PER, TOTAL)).toBe(0);
    });

    it('自分自身の現在値は残りの計算に含めない', () => {
        // ranks[0] が既に 5 でも、他項目 (計 5) を引いた残り 5 まで入れ直せる
        expect(clampWithinGroup([5, 5, 0, 0, 0], 0, 5, PER, TOTAL)).toBe(5);
    });

    it('負値は 0 にする', () => {
        expect(clampWithinGroup([0, 0, 0, 0, 0], 0, -1, PER, TOTAL)).toBe(0);
    });
});
