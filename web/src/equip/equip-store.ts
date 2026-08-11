// 装備セットエディタの共有状態 (旧 web/js/equip-state.js を統合)。
//
// equipState はスロット UI・セット管理・ステータス表示・共有閲覧モードが
// 読み書きする可変オブジェクト。React 側はバージョン番号を購読し、
// notifyEquipState() (updateEquipEditStatus 経由で全変更点から呼ばれる)
// で再描画される。
import { EQUIPMENT_SLOTS } from '../constants';
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

export const equipState = {
    /** 選択中キャラクター名 */
    currentEquipChar: '',
    /** 選択中ジョブキー */
    currentEquipJob: '',
    /** 選択中サポートジョブキー */
    currentEquipSupportJob: '',
    /** slotKey → EquipSlotData | null */
    currentEquipSlots: {} as Record<string, EquipSlotData | null | undefined>,
    /** 編集中セット名 (新規なら null) */
    editingEquipSetName: null as string | null,
    /** "+" タブから新規作成中なら true */
    isNewEquipSet: false,
    /** 共有閲覧モード時のキャラクター snapshot (loadCharacters() に存在しないため)。null なら通常モード */
    sharedCharacterOverride: null as unknown,
};

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
