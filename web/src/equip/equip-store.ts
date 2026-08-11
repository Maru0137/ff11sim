// 装備セットエディタ共有状態 (web/js/equip-state.js) と React の橋渡し。
//
// equipState 本体は従来どおり複数のレガシーモジュールが直接読み書きする
// 可変オブジェクトのまま。React 側はバージョン番号を購読し、
// notifyEquipState() (updateEquipEditStatus 経由で全変更点から呼ばれる)
// で再描画される。equipState 自体の React 移行は Phase 4 以降。
import { equipState } from '../../js/equip-state.js';
import { EQUIPMENT_SLOTS } from '../../js/constants.js';
import { createStore } from '../store-utils';

/** 装備スロット 1 枠の保存データ (equipState.currentEquipSlots の値) */
export interface EquipSlotData {
    item_id: number;
    name_en?: string;
    name_ja?: string;
    description_ja?: string;
    skill?: number | null;
    aug_path?: number | null;
    aug_rank?: number | null;
    custom_description?: string;
}

export interface SlotDef {
    key: string;
    label: string;
}

export const SLOT_DEFS: SlotDef[] = EQUIPMENT_SLOTS;

export function getSlots(): Record<string, EquipSlotData | null | undefined> {
    return equipState.currentEquipSlots;
}

export function createEmptySlots(): Record<string, null> {
    const slots: Record<string, null> = {};
    SLOT_DEFS.forEach((s) => {
        slots[s.key] = null;
    });
    return slots;
}

// equipState の変更通知 (毎回インクリメントされるバージョン番号)
const versionStore = createStore(0);
export const subscribeEquipState = versionStore.subscribe;
export const getEquipStateVersion = versionStore.get;
export function notifyEquipState() {
    versionStore.set(versionStore.get() + 1);
}

// 装備セットの読み込み (セット切替・新規フォーム) の通知。
// スロット行の検索テキスト等のローカル状態を破棄して選択名に戻すため、
// 通常の変更通知とは別に世代番号で管理し、行の key に使う。
const generationStore = createStore(0);
export const subscribeSlotsGeneration = generationStore.subscribe;
export const getSlotsGeneration = generationStore.get;
export function notifySlotsLoaded() {
    generationStore.set(generationStore.get() + 1);
}

export { equipState };
