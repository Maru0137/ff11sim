// オーグメントデータの取得と、スロット行のオーグメント UI 更新。
// データソースは web/data/augments.json (docs/adr/0004)。
// AUGMENT_JA_TO_EN は constants.js、convertAugmentJaToEn は utils.js に定義。
import { equipState } from './equip-state.js';

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

export function updateAugPathOptions(slotKey) {
    const augContainer = document.querySelector(`.equip-slot-aug-container[data-slot="${slotKey}"]`);
    const pathSelect = document.querySelector(`.equip-slot-aug-path[data-slot="${slotKey}"]`);
    if (!augContainer || !pathSelect) return;

    const slotData = equipState.currentEquipSlots[slotKey];
    const augInfo = slotData ? getItemAugments(slotData.item_id) : null;
    const hasAug = !!(augInfo && augInfo.paths && augInfo.paths.length > 0);

    // オーグメント候補が無い (装備未選択 or 該当装備にオーグなし) 場合は disabled に
    pathSelect.disabled = !hasAug;

    if (!hasAug) {
        pathSelect.innerHTML = '<option value="">-- オーグメント --</option>';
        updateAugTextDisplay(slotKey);
        return;
    }

    pathSelect.innerHTML = '<option value="">-- オーグメント --</option>';
    augInfo.paths.forEach((path, idx) => {
        if (!path.ranks) return;
        path.ranks.forEach(r => {
            const opt = document.createElement('option');
            opt.value = `${idx}-${r.rank}`;
            opt.textContent = `${path.type} Rank ${r.rank}`;
            pathSelect.appendChild(opt);
        });
    });

    if (slotData.aug_path != null && slotData.aug_rank != null) {
        pathSelect.value = `${slotData.aug_path}-${slotData.aug_rank}`;
    }
    updateAugTextDisplay(slotKey);
}

export function updateAugTextDisplay(slotKey) {
    const augTextDiv = document.querySelector(`.equip-slot-aug-text[data-slot="${slotKey}"]`);
    if (!augTextDiv) return;
    const slotData = equipState.currentEquipSlots[slotKey];
    const text = getAugmentText(slotData);
    augTextDiv.textContent = text || '';
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
