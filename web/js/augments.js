// オーグメントデータの取得と参照。
// データソースは web/public/data/augments.json (docs/adr/0004)。
// AUGMENT_JA_TO_EN は constants.js、convertAugmentJaToEn は utils.js に定義。
// スロット行のオーグメント UI は React 側 (web/src/equip/EquipSlots.tsx)。

let augmentData = {};

export async function loadAugmentData() {
    try {
        const res = await fetch('./data/augments.json');
        const d = await res.json();
        augmentData = d.augments || {};
    } catch {
        augmentData = {};
    }
}

export function getItemAugments(itemId) {
    return augmentData[String(itemId)] || null;
}

export function getAugmentText(slotData) {
    if (!slotData || slotData.aug_path == null || slotData.aug_rank == null) return null;
    const augInfo = getItemAugments(slotData.item_id);
    if (!augInfo) return null;
    const path = augInfo.paths[slotData.aug_path];
    if (!path) return null;
    const rankEntry = path.ranks.find(r => r.rank === slotData.aug_rank);
    return rankEntry ? rankEntry.text : null;
}
