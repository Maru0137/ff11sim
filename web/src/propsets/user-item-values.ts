// ユーザー定義プロパティ項目の値計算 (docs/adr/0015) の入口。
//
// 実体は Rust 側 (rust/src/equip.rs の EquipSet::property_values、docs/adr/0018)。
// 装備説明文 (日本語) + オーグメント文 + カスタム説明の 3 ソースから「term+N」を
// 抽出してスロット横断で合算する。ステータス計算の経路と違い、日本語のまま扱う:
// - アイテムは description_en ではなく description_ja を使う
// - オーグメント/カスタム文も JA→EN 変換を通さない (任意の日本語プロパティ名を
//   扱うため。変換表に無い語が消えてしまう)
//
// 抽出は各テキストにつき最初の一致 1 件のみ (合算は装備・ソース間のみ)。
// 「二刀流効果アップ」のような数値を伴わない表記は対象外 (将来
// augments.json のような補完データで対応する)。
// 「ペット:命中+3 モクシャ+3」のような条件ラベル配下 (ラベルから行末まで) は
// 本体に常時乗る値ではないため拾わない (docs/knowledge/items/description_labels.md)。
//
// この層が持つのは「プロパティ名 → 表示 id (`user:<term>`)」の対応だけ。
// これは UI 側の取り決めなので Rust には持たせていない。
import { equip_set_property_values, isItemsLoaded } from '../wasm';
import type { EquipSlotData } from '../equip/equip-store';
import type { UserPropertyItem } from './types';

/**
 * スロット別のユーザー定義項目値 (内訳モーダル用、docs/adr/0016)。
 * 寄与のあるスロットのみ含む。値 0 の項目はキーごと含まない (疎)。
 */
export function calculateUserPropertyValuesPerSlot(
    slots: Record<string, EquipSlotData | null | undefined> | undefined,
    userItems: UserPropertyItem[]
): Record<string, Record<string, number>> {
    if (!slots || userItems.length === 0 || !isItemsLoaded()) return {};

    // undefined を null に寄せる (equip-bonuses.ts と同じ理由)
    const normalized = Object.fromEntries(
        Object.entries(slots).map(([key, data]) => [key, data ?? null])
    );
    const byTerm: Record<string, Record<string, number>> = equip_set_property_values(
        { slots: normalized },
        userItems.map((item) => item.term)
    );

    // term キーを表示 id に付け替える。同じ term の項目は同じ値になる
    const perSlot: Record<string, Record<string, number>> = {};
    for (const [slotKey, values] of Object.entries(byTerm)) {
        const bucket: Record<string, number> = {};
        for (const item of userItems) {
            const v = values[item.term];
            if (v) bucket[item.id] = v;
        }
        perSlot[slotKey] = bucket;
    }
    return perSlot;
}

export function calculateUserPropertyValues(
    slots: Record<string, EquipSlotData | null | undefined> | undefined,
    userItems: UserPropertyItem[]
): Record<string, number> {
    const totals: Record<string, number> = {};
    for (const item of userItems) totals[item.id] = 0;
    for (const bucket of Object.values(calculateUserPropertyValuesPerSlot(slots, userItems))) {
        for (const [id, v] of Object.entries(bucket)) {
            totals[id] += v;
        }
    }
    return totals;
}
