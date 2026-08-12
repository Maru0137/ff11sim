// augments.json のリンバス装束カバレッジテスト。
// scripts/scrape_augments.py の LIMBUS_SET_PAGES に NQ しか登録されておらず
// HQ1/HQ2 (例: 寵愛装束) が欠落していた問題の再発防止。
// items.json は CI 生成物でユニットテストから参照できないため (ADR 0003)、
// ゲーム内で不変のアイテム ID を直接記載する。
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';

interface AugmentRank {
    rank: number;
    text: string;
}

const augments: Record<string, { paths: { type: string; ranks?: AugmentRank[] }[] }> = JSON.parse(
    readFileSync(new URL('../public/data/augments.json', import.meta.url), 'utf-8')
).augments;

// 解呪後のリンバス装束 5 セット × NQ/HQ1/HQ2 × 5 部位 (ID は連続)
const LIMBUS_ARMOR_SETS: [string, number, number][] = [
    ['ホープ/パーフェク/レベレ', 24120, 24134],
    ['トラスト/プレステ/スオーン', 24135, 24149],
    ['ブレーブ/イントレ/インドム', 24150, 24164],
    ['ジャスト/マグニ/ドゥーテ', 24165, 24179],
    ['慈悲/慈愛/寵愛', 24180, 24194],
];

const LIMBUS_ACCESSORIES: [string, number][] = [
    ['アスプロピアス', 26119],
    ['メランリング', 26234],
    ['アスプロマント', 26275],
    ['メランマント', 26276],
];

describe('augments.json のリンバス装束カバレッジ', () => {
    for (const [label, from, to] of LIMBUS_ARMOR_SETS) {
        it(`${label} (${from}-${to}) の全アイテムに Rank 30 のオーグメントがある`, () => {
            for (let id = from; id <= to; id++) {
                const entry = augments[String(id)];
                expect(entry, `ID ${id} のエントリ`).toBeDefined();
                const rank30 = entry.paths[0]?.ranks?.find((r) => r.rank === 30);
                expect(rank30?.text, `ID ${id} の Rank 30 テキスト`).toBeTruthy();
            }
        });
    }

    for (const [name, id] of LIMBUS_ACCESSORIES) {
        it(`${name} (${id}) にオーグメントがある`, () => {
            const entry = augments[String(id)];
            expect(entry, `ID ${id} のエントリ`).toBeDefined();
            expect(entry.paths[0]?.ranks?.length, `ID ${id} の rank 数`).toBeGreaterThan(0);
        });
    }
});
