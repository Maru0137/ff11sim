// モーダルの開閉状態ストア。開くきっかけはレガシー JS 側にもある
// (equip-slots の「?」ボタン、share-ui の共有/インポートボタン) ため、
// 開閉 API を React の外に置き、両側から呼べるようにする。
import { createStore } from '../store-utils';

export interface CharacterOption {
    name: string;
    race: string;
}

/** share.js#loadSharedEquipSet の返す共有装備セット (必要フィールドのみ) */
export interface SharedEquipSetData {
    name?: string;
    characterName?: string;
    job?: string;
    slots?: Record<string, unknown>;
}

export interface ImportSharePayload {
    characters: CharacterOption[];
    shared: SharedEquipSetData;
    /** モーダルを開くたびに増える。React 側でフォーム状態リセットの key に使う */
    openCount: number;
}

interface ModalsState {
    customAugHelpOpen: boolean;
    /** null = 非表示 */
    shareUrl: string | null;
    /** null = 非表示 */
    importShare: ImportSharePayload | null;
}

export const modalsStore = createStore<ModalsState>({
    customAugHelpOpen: false,
    shareUrl: null,
    importShare: null,
});

let importOpenCount = 0;

export function openCustomAugHelp() {
    modalsStore.set({ ...modalsStore.get(), customAugHelpOpen: true });
}

export function closeCustomAugHelp() {
    modalsStore.set({ ...modalsStore.get(), customAugHelpOpen: false });
}

export function openShareUrlModal(url: string) {
    modalsStore.set({ ...modalsStore.get(), shareUrl: url });
}

export function closeShareUrlModal() {
    modalsStore.set({ ...modalsStore.get(), shareUrl: null });
}

export function openImportShareModal(payload: Omit<ImportSharePayload, 'openCount'>) {
    modalsStore.set({
        ...modalsStore.get(),
        importShare: { ...payload, openCount: ++importOpenCount },
    });
}

export function closeImportShareModal() {
    modalsStore.set({ ...modalsStore.get(), importShare: null });
}
