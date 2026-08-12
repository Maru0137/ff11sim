// プロパティセットの状態管理 (docs/adr/0015)。
// createStore + 命令的 async 関数の規約は equip-sets-store.ts に合わせる。
//
// 不変条件: このモジュールは status-store / compute を import しない
// (compute.ts がこちらを参照する向きのため)。ユーザー定義項目の変更後に
// ステータス再計算が必要な場合は、呼び出し側が updateEquipEditStatus() を呼ぶ。
import { createStore } from '../store-utils';
import { loadPropsetDoc, savePropsetDoc } from '../storage';
import {
    normalizePropsetDoc,
    stripItemFromSets,
    userItemId,
    type PropertySet,
    type PropsetDoc,
    type UserPropertyItem,
} from './types';

interface PropsetsState {
    sets: PropertySet[];
    userItems: UserPropertyItem[];
    loaded: boolean;
}

export const propsetsStore = createStore<PropsetsState>({
    sets: [],
    userItems: [],
    loaded: false,
});

function patch(p: Partial<PropsetsState>) {
    propsetsStore.set({ ...propsetsStore.get(), ...p });
}

function currentDoc(): PropsetDoc {
    const { sets, userItems } = propsetsStore.get();
    return { sets, userItems };
}

async function persist(doc: PropsetDoc) {
    await savePropsetDoc(doc);
    patch({ sets: doc.sets, userItems: doc.userItems });
}

export async function loadPropsets(): Promise<void> {
    const doc = normalizePropsetDoc(await loadPropsetDoc());
    patch({ sets: doc.sets, userItems: doc.userItems, loaded: true });
}

/** id で upsert する。新規作成側で id (crypto.randomUUID()) を採番すること。 */
export async function savePropertySet(set: PropertySet): Promise<void> {
    const doc = currentDoc();
    const idx = doc.sets.findIndex((s) => s.id === set.id);
    const sets = idx >= 0
        ? doc.sets.map((s) => (s.id === set.id ? set : s))
        : [...doc.sets, set];
    await persist({ ...doc, sets });
}

export async function deletePropertySet(id: string): Promise<void> {
    const doc = currentDoc();
    await persist({ ...doc, sets: doc.sets.filter((s) => s.id !== id) });
}

/**
 * ユーザー定義項目を追加する。既存 term なら既存項目を返す。
 * 空文字は呼び出し側で弾くこと (UI バリデーション)。
 */
export async function addUserItem(term: string): Promise<UserPropertyItem> {
    const trimmed = term.trim();
    if (!trimmed) throw new Error('empty term');
    const doc = currentDoc();
    const existing = doc.userItems.find((u) => u.term === trimmed);
    if (existing) return existing;
    const item: UserPropertyItem = { id: userItemId(trimmed), term: trimmed };
    await persist({ ...doc, userItems: [...doc.userItems, item] });
    return item;
}

/** ユーザー定義項目を削除し、全セットからも取り除く。 */
export async function deleteUserItem(id: string): Promise<void> {
    const doc = currentDoc();
    await persist({
        sets: stripItemFromSets(doc.sets, id),
        userItems: doc.userItems.filter((u) => u.id !== id),
    });
}
