// 装備グリッドのセルに出す 1 行要約 (EquipGrid.tsx)。
//
// description_ja の改行はリテラルの "\n" (rust/src/items.rs)、
// オーグメントテキスト (augments.json) は実改行を含むため、両方を
// 空白へ潰して 1 行に連結する。
// 注意: constants.ts (top-level await fetch) に依存させないこと。
// 依存すると Vitest がデータ取得できず落ちる。

export interface EquipSummaryParts {
    description?: string | null;
    augText?: string | null;
    custom?: string | null;
}

export function equipSummary({ description, augText, custom }: EquipSummaryParts): string {
    return [description, augText, custom]
        .map((s) => (s ?? '').replace(/\\n|\n/g, ' ').replace(/\s+/g, ' ').trim())
        .filter((s) => s !== '')
        .join(' ');
}
