// PropsetDoc の正規化・純関数ヘルパーのテスト。
import { describe, it, expect } from 'vitest';
import { normalizePropsetDoc, stripItemFromSets, userItemId } from './types';

describe('normalizePropsetDoc', () => {
    it('null や不正値は空ドキュメントに正規化される', () => {
        expect(normalizePropsetDoc(null)).toEqual({ sets: [], userItems: [] });
        expect(normalizePropsetDoc('junk')).toEqual({ sets: [], userItems: [] });
        expect(normalizePropsetDoc(42)).toEqual({ sets: [], userItems: [] });
        expect(normalizePropsetDoc({})).toEqual({ sets: [], userItems: [] });
    });

    it('欠損フィールドが補完される', () => {
        const doc = normalizePropsetDoc({
            sets: [{ id: 'a', name: 'セットA' }, { id: 'b', name: 'B', items: ['store_tp'] }],
        });
        expect(doc.sets).toEqual([
            { id: 'a', name: 'セットA', items: [] },
            { id: 'b', name: 'B', items: ['store_tp'] },
        ]);
        expect(doc.userItems).toEqual([]);
    });

    it('形の合わない要素は除外される', () => {
        const doc = normalizePropsetDoc({
            sets: [null, { name: 'idなし' }, { id: 'ok', name: 'OK', items: [] }],
            userItems: [null, { id: 'user:x' }, { id: 'user:二刀流', term: '二刀流' }],
        });
        expect(doc.sets.map((s) => s.id)).toEqual(['ok']);
        expect(doc.userItems).toEqual([{ id: 'user:二刀流', term: '二刀流' }]);
    });
});

describe('stripItemFromSets', () => {
    it('全セットから指定 id のみ取り除く', () => {
        const sets = [
            { id: 'a', name: 'A', items: ['store_tp', 'user:二刀流', 'attack'] },
            { id: 'b', name: 'B', items: ['attack'] },
        ];
        const result = stripItemFromSets(sets, 'user:二刀流');
        expect(result[0].items).toEqual(['store_tp', 'attack']);
        expect(result[1].items).toEqual(['attack']);
        // 含まないセットは同一参照のまま (無駄な再生成をしない)
        expect(result[1]).toBe(sets[1]);
    });
});

describe('userItemId', () => {
    it('term に user: プレフィクスを付ける', () => {
        expect(userItemId('二刀流')).toBe('user:二刀流');
    });
});
