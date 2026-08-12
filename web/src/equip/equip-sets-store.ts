// 装備セット管理パネルの状態と操作 (旧 web/js/equip-sets.js の移植)。
//
// セレクタ (キャラ/ジョブ/サポ) とタブバーは EquipSetControls、
// 名前入力とボタン群は EquipSetToolbar が描画する。2 つの React root に
// またがるため状態はこのストアに置く。
// equipState (選択キャラ・ジョブ・スロット) への書き込みは従来どおりで、
// compute.ts / share-ui.js などの読み手には影響しない。
import { get_item_by_id } from '../wasm';
import { loadCharacters, loadEquipSets, saveEquipSets } from '../storage';
import { equipState, createEmptySlots, notifySlotsLoaded, notifyEquipState } from './equip-store';
import type { EquipSlotData } from './equip-store';
import { updateEquipEditStatus } from '../status/status-store';
import { createStore } from '../store-utils';
import {
    equipSetPrefKey,
    migrateSelectionKey,
    removeSelectedPropsetId,
} from '../propsets/selection-prefs';

export interface CharacterSummary {
    name: string;
    race: string;
}

export interface EquipSet {
    name: string;
    character: string;
    job: string;
    slots: Record<string, EquipSlotData | null | undefined>;
}

export interface EquipSetsPanelState {
    characters: CharacterSummary[];
    /** 現在の (キャラ, ジョブ) のセット一覧 (タブ表示順) */
    sets: EquipSet[];
    /** アクティブタブのセット名 (新規フォーム中や未選択は null) */
    selectedName: string | null;
    /** 編集フォーム (#equipEditSection) の表示 */
    editVisible: boolean;
    /** 削除ボタンの表示 (既存セット編集中のみ) */
    deleteVisible: boolean;
    /** 名前入力欄の値 */
    nameInput: string;
    /** 共有閲覧モード (インポートボタン表示) */
    shareMode: boolean;
}

export const equipSetsStore = createStore<EquipSetsPanelState>({
    characters: [],
    sets: [],
    selectedName: null,
    editVisible: false,
    deleteVisible: false,
    nameInput: '',
    shareMode: false,
});

function patch(p: Partial<EquipSetsPanelState>) {
    equipSetsStore.set({ ...equipSetsStore.get(), ...p });
}

export function setNameInput(v: string) {
    patch({ nameInput: v });
}

// ===== 読み込み =====

/** キャラクターセレクタの選択肢を保存済みキャラクター一覧から再構築 */
export async function reloadCharacters() {
    const characters = await loadCharacters();
    patch({ characters });
}

async function loadScopedSets(): Promise<EquipSet[]> {
    const all: EquipSet[] = await loadEquipSets();
    return all.filter(
        (s) => s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob
    );
}

/** 旧 renderEquipSetTabs: タブ一覧を再構築し、先頭タブを自動選択する */
export async function refreshTabs() {
    if (!equipState.currentEquipChar || !equipState.currentEquipJob) {
        patch({ sets: [] });
        hideEditForm();
        return;
    }
    const sets = await loadScopedSets();
    patch({ sets });
    if (sets.length > 0) {
        await selectSet(sets[0].name);
    } else {
        hideEditForm();
    }
}

/** 起動時初期化: キャラ一覧の読み込み (旧 updateEquipCharSelector 相当) */
export async function initEquipSetPanel() {
    await reloadCharacters();
}

/** 認証状態変化・同期完了時の再読込 */
export async function refreshEquipSetPanel() {
    await reloadCharacters();
    await refreshTabs();
}

// ===== セット選択・編集フォーム =====

/** equipSet の内容を equipState と編集フォームに反映する (旧 showEquipSetEditForm) */
function applyEquipSet(equipSet: EquipSet | null) {
    if (equipSet) {
        equipState.editingEquipSetName = equipSet.name;
        equipState.currentEquipSlots = { ...equipSet.slots };
        // 旧データは skill フィールドを持たないので item_id から補完
        const slots: Record<string, EquipSlotData | null | undefined> =
            equipState.currentEquipSlots;
        Object.keys(slots).forEach((k) => {
            const d = slots[k];
            if (d && d.skill == null && d.item_id != null) {
                const item = get_item_by_id(d.item_id);
                if (item && item.skill != null) d.skill = item.skill;
            }
        });
        patch({ editVisible: true, deleteVisible: true, nameInput: equipSet.name });
    } else {
        equipState.editingEquipSetName = null;
        equipState.currentEquipSlots = createEmptySlots();
        patch({ editVisible: true, deleteVisible: false, nameInput: '' });
    }
    // スロット行 (React) に読み込みを通知し、検索テキスト等を選択名へリセット
    notifySlotsLoaded();
    updateEquipEditStatus();
}

export async function selectSet(name: string) {
    equipState.isNewEquipSet = false;
    patch({ selectedName: name });
    const sets = await loadEquipSets();
    const es = (sets as EquipSet[]).find(
        (s) =>
            s.character === equipState.currentEquipChar &&
            s.job === equipState.currentEquipJob &&
            s.name === name
    );
    if (es) {
        applyEquipSet(es);
    }
}

export function newSet() {
    equipState.isNewEquipSet = true;
    patch({ selectedName: null });
    applyEquipSet(null);
}

export function hideEditForm() {
    patch({ editVisible: false, selectedName: null });
    equipState.editingEquipSetName = null;
    equipState.isNewEquipSet = false;
}

/** 共有閲覧モード: 共有された装備セットを編集フォームに流し込む (share-ui.js から呼ばれる) */
export function showSharedEquipSet(equipSet: { name?: string; slots: EquipSet['slots'] }) {
    patch({ shareMode: true });
    applyEquipSet({
        name: equipSet.name || '',
        character: equipState.currentEquipChar,
        job: equipState.currentEquipJob,
        slots: equipSet.slots || {},
    });
}

// ===== 共有ヘッダ (#sharedHeader) =====
// 共有閲覧モードで表示する「共有された装備セット」バナー。
// テキストは share-ui.js#enterShareMode が組み立てる。

export interface ShareHeaderState {
    visible: boolean;
    name: string;
    meta: string;
}

export const shareHeaderStore = createStore<ShareHeaderState>({
    visible: false,
    name: '-',
    meta: '',
});

export function setShareHeader(name: string, meta: string) {
    shareHeaderStore.set({ visible: true, name, meta });
}

/** 共有データの読み込み失敗時: ヘッダにエラーを出しつつ編集フォームの枠だけ開く */
export function showShareLoadError(message: string) {
    shareHeaderStore.set({ visible: true, name: '読み込みに失敗しました', meta: ` / ${message}` });
    patch({ editVisible: true });
}

// ===== 保存・複製・削除・並べ替え =====

export async function saveEquipSet() {
    const name = equipSetsStore.get().nameInput.trim();
    if (!name) {
        alert('装備セット名を入力してください。');
        return;
    }

    const sets: EquipSet[] = await loadEquipSets();
    const character = equipState.currentEquipChar;
    const job = equipState.currentEquipJob;

    if (equipState.editingEquipSetName) {
        // 既存セットの編集
        const idx = sets.findIndex(
            (s) => s.character === character && s.job === job && s.name === equipState.editingEquipSetName
        );
        if (idx >= 0) {
            // 改名時は名前の重複を検査
            if (name !== equipState.editingEquipSetName) {
                if (sets.some((s) => s.character === character && s.job === job && s.name === name)) {
                    alert(`装備セット「${name}」は既に存在します。`);
                    return;
                }
                // 識別キーが変わるため、プロパティセット選択記憶を移し替える
                migrateSelectionKey(
                    equipSetPrefKey(character, job, equipState.editingEquipSetName),
                    equipSetPrefKey(character, job, name)
                );
            }
            sets[idx] = { name, job, character, slots: { ...equipState.currentEquipSlots } };
        }
    } else {
        // 新規作成
        if (sets.some((s) => s.character === character && s.job === job && s.name === name)) {
            alert(`装備セット「${name}」は既に存在します。`);
            return;
        }
        sets.push({ name, job, character, slots: { ...equipState.currentEquipSlots } });
    }

    await saveEquipSets(sets);
    await refreshTabs();
    await selectSet(name);
}

export async function copyEquipSet() {
    if (!equipState.currentEquipChar || !equipState.currentEquipJob) return;
    // 編集中のセット名をベースに、未使用の名前を生成
    const baseName = equipState.editingEquipSetName || equipSetsStore.get().nameInput.trim();
    if (!baseName) {
        alert('複製元の装備セットがありません。');
        return;
    }
    const sets: EquipSet[] = await loadEquipSets();
    const inScope = sets.filter(
        (s) => s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob
    );
    const usedNames = new Set(inScope.map((s) => s.name));
    let newName = `${baseName} のコピー`;
    let i = 2;
    while (usedNames.has(newName)) {
        newName = `${baseName} のコピー ${i++}`;
    }
    sets.push({
        name: newName,
        job: equipState.currentEquipJob,
        character: equipState.currentEquipChar,
        slots: { ...equipState.currentEquipSlots },
    });
    await saveEquipSets(sets);
    await refreshTabs();
    await selectSet(newName);
}

export async function deleteEquipSet() {
    if (!equipState.editingEquipSetName) return;
    if (!confirm(`装備セット「${equipState.editingEquipSetName}」を削除しますか？`)) return;

    const sets = ((await loadEquipSets()) as EquipSet[]).filter(
        (s) =>
            !(
                s.character === equipState.currentEquipChar &&
                s.job === equipState.currentEquipJob &&
                s.name === equipState.editingEquipSetName
            )
    );
    await saveEquipSets(sets);
    removeSelectedPropsetId(
        equipSetPrefKey(
            equipState.currentEquipChar,
            equipState.currentEquipJob,
            equipState.editingEquipSetName
        )
    );
    equipState.editingEquipSetName = null;
    await refreshTabs();
}

export async function reorderEquipSetTabs(fromName: string, toName: string) {
    // 現在の (character, job) 内で fromName を toName の位置に移動。
    // 他の (character, job) のセットには触れずに save。
    const allSets: EquipSet[] = await loadEquipSets();
    const inScope = (s: EquipSet) =>
        s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob;
    const tabs = allSets.filter(inScope);
    const others = allSets.filter((s) => !inScope(s));

    const fromIdx = tabs.findIndex((s) => s.name === fromName);
    const toIdx = tabs.findIndex((s) => s.name === toName);
    if (fromIdx < 0 || toIdx < 0) return;

    const [moved] = tabs.splice(fromIdx, 1);
    tabs.splice(toIdx, 0, moved);

    // 順序統合 (other は元の位置を維持できないので末尾に集める。
    //  Local 側は順序を厳密維持しないため許容、Supabase 側は (char, job) 単位で position 管理)
    await saveEquipSets([...others, ...tabs]);
    await refreshTabs();
    await selectSet(moved.name);
}

// ===== セレクタ =====

// セレクタは equipState を値とする controlled select なので、
// 変更を同期通知しないと React が直前の描画値へ巻き戻してしまう。

export function selectChar(value: string) {
    equipState.currentEquipChar = value;
    if (!value) {
        equipState.currentEquipJob = '';
        equipState.currentEquipSupportJob = '';
    }
    notifyEquipState();
    refreshTabs();
}

export function selectJob(value: string) {
    equipState.currentEquipJob = value;
    // ジョブ変更時はサポートジョブをリセット (旧 updateSupportJobOptions)
    equipState.currentEquipSupportJob = '';
    notifyEquipState();
    refreshTabs();
}

export function selectSupportJob(value: string) {
    equipState.currentEquipSupportJob = value;
    updateEquipEditStatus();
}
