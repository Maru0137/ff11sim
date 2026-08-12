// ユーザー定義プロパティ項目の値計算 (docs/adr/0015)。
// 装備説明文 (日本語) + オーグメント文 + カスタム説明の 3 ソースから
// 「term+N」を extract_named_stat (WASM) で抽出してスロット横断で合算する。
// 3 ソースの組み立ては calculateEquipSetBonuses (equip-bonuses.ts) の
// ループと同じ構成だが、こちらは日本語テキストをそのまま渡す:
// - アイテムは description_en ではなく description_ja を使う
// - オーグメント/カスタム文は元々日本語なので convertAugmentJaToEn を通さない
//
// 抽出は各テキストにつき最初の一致 1 件のみ (合算は装備・ソース間のみ)。
// 「二刀流効果アップ」のような数値を伴わない表記は対象外 (将来
// augments.json のような補完データで対応する)。
import { get_item_by_id, extract_named_stat, isItemsLoaded } from '../wasm';
import { getAugmentText } from '../augments';
import type { EquipSlotData } from '../equip/equip-store';
import type { UserPropertyItem } from './types';

export function calculateUserPropertyValues(
    slots: Record<string, EquipSlotData | null | undefined> | undefined,
    userItems: UserPropertyItem[]
): Record<string, number> {
    const totals: Record<string, number> = {};
    for (const item of userItems) totals[item.id] = 0;
    if (!slots || userItems.length === 0 || !isItemsLoaded()) return totals;

    for (const slotKey of Object.keys(slots)) {
        const slotData = slots[slotKey];
        if (!slotData) continue;

        const item = get_item_by_id(slotData.item_id);
        const texts = [
            item?.description_ja,
            getAugmentText(slotData),
            slotData.custom_description,
        ];
        for (const text of texts) {
            if (!text) continue;
            for (const userItem of userItems) {
                totals[userItem.id] += extract_named_stat(text, userItem.term);
            }
        }
    }
    return totals;
}
