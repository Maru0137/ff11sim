// オーグメントデータの取得と参照 (旧 web/js/augments.js の TS 化)。
// データソースは web/public/data/augments.json (docs/adr/0004)。
// getAugmentText はオーグメント文の表示 (装備カード / 選択モーダル) 用。
// 解釈経路のオグメント文は Rust 側が同じ規則で解決する
// (rust/src/equip.rs の Equip::augment_text、docs/adr/0018)。
// スロット行のオーグメント UI は web/src/equip/EquipSlots.tsx。

export interface AugmentRank {
    rank: number;
    text: string;
}

export interface AugmentPath {
    type: string;
    ranks?: AugmentRank[];
}

export interface AugmentInfo {
    paths: AugmentPath[];
}

/** オーグメント選択状態を持つスロットデータ (equip-store の EquipSlotData 互換) */
interface AugmentSlotData {
    item_id: number;
    aug_path?: number | null;
    aug_rank?: number | null;
}

let augmentData: Record<string, AugmentInfo> = {};

export async function loadAugmentData() {
    try {
        const res = await fetch('./data/augments.json');
        const d = await res.json();
        augmentData = d.augments || {};
    } catch {
        augmentData = {};
    }
}

export function getItemAugments(itemId: number): AugmentInfo | null {
    return augmentData[String(itemId)] || null;
}

/**
 * 装備選択時に自動で入れるオーグメント。タイプが 1 つしか無い装備は
 * 選ぶ余地が無く、実用上ほぼ最高ランクなのでそれを既定値にする。
 * タイプが複数ある装備はユーザーに選ばせる (null を返す)。
 */
export function getDefaultAugmentSelection(
    augInfo: AugmentInfo | null | undefined
): { path: number; rank: number } | null {
    if (!augInfo || augInfo.paths.length !== 1) return null;
    const ranks = augInfo.paths[0].ranks;
    if (!ranks || ranks.length === 0) return null;
    const best = ranks.reduce((a, r) => (r.rank > a.rank ? r : a));
    return { path: 0, rank: best.rank };
}

export function getAugmentText(slotData: AugmentSlotData | null | undefined): string | null {
    if (!slotData || slotData.aug_path == null || slotData.aug_rank == null) return null;
    const augInfo = getItemAugments(slotData.item_id);
    if (!augInfo) return null;
    const path = augInfo.paths[slotData.aug_path];
    if (!path || !path.ranks) return null;
    const rankEntry = path.ranks.find((r) => r.rank === slotData.aug_rank);
    return rankEntry ? rankEntry.text : null;
}
