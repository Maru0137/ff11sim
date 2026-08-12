// calculateUserPropertyValues の実 WASM 込みテスト。
// WASM の読み込み方は equip-bonuses.test.ts と同じ (node は file URL を
// fetch できないため .wasm のバイト列を渡す。web/pkg のビルドが前提)。
import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync } from 'node:fs';
import {
    calculateUserPropertyValues,
    calculateUserPropertyValuesPerSlot,
} from './user-item-values';

beforeAll(async () => {
    const { initWasmRuntime } = await import('../../js/wasm.js');
    const wasmBytes = readFileSync(new URL('../../pkg/ff11sim_bg.wasm', import.meta.url));
    await initWasmRuntime(wasmBytes);
});

const userItem = (term: string) => ({ id: `user:${term}`, term });

describe('calculateUserPropertyValues', () => {
    it('スロット未設定なら全項目 0', () => {
        const totals = calculateUserPropertyValues(undefined, [userItem('二刀流')]);
        expect(totals).toEqual({ 'user:二刀流': 0 });
    });

    it('複数スロットのカスタム説明から抽出して合算する', () => {
        // item_id 0 は存在しない → アイテム説明はスキップ、カスタム説明のみ
        const totals = calculateUserPropertyValues(
            {
                body: { item_id: 0, custom_description: '二刀流+5' },
                hands: { item_id: 0, custom_description: '二刀流+3 ストアTP+10' },
            },
            [userItem('二刀流'), userItem('ストアTP')]
        );
        expect(totals['user:二刀流']).toBe(8);
        expect(totals['user:ストアTP']).toBe(10);
    });

    it('一致しない term は 0 のまま', () => {
        const totals = calculateUserPropertyValues(
            { body: { item_id: 0, custom_description: 'ストアTP+5' } },
            [userItem('二刀流')]
        );
        expect(totals['user:二刀流']).toBe(0);
    });

    it('スロット別の値が取れ、総和が合算関数と一致する', () => {
        const slots = {
            body: { item_id: 0, custom_description: '二刀流+5' },
            hands: { item_id: 0, custom_description: '二刀流+3 ストアTP+10' },
        };
        const items = [userItem('二刀流'), userItem('ストアTP')];
        const perSlot = calculateUserPropertyValuesPerSlot(slots, items);
        expect(perSlot.body['user:二刀流']).toBe(5);
        expect(perSlot.hands['user:二刀流']).toBe(3);
        expect(perSlot.hands['user:ストアTP']).toBe(10);
        // 値 0 の項目はキーごと含まない (疎)
        expect(perSlot.body['user:ストアTP']).toBeUndefined();

        const totals = calculateUserPropertyValues(slots, items);
        expect(totals['user:二刀流']).toBe(8);
        expect(totals['user:ストアTP']).toBe(10);
    });

    it('日本語テキストを JA→EN 変換せずそのまま抽出する', () => {
        // 「命中」は convertAugmentJaToEn を通すと Accuracy に変換されて
        // しまう語。変換されていれば 0 になるので、10 が取れることが
        // 生の日本語テキストに対して抽出している証拠になる。
        const totals = calculateUserPropertyValues(
            { body: { item_id: 0, custom_description: '命中+10' } },
            [userItem('命中')]
        );
        expect(totals['user:命中']).toBe(10);
    });
});
