// 装備セット共有の UI 層ロジック (旧 web/js/share-ui.js の TS 化。DOM 操作なし)。
// 共有 URL の発行 (作成側)・共有閲覧モード (?share=<uuid>)・インポート (閲覧側)。
// Supabase との読み書きは share.ts (docs/adr/0008)、表示は React 側
// (App.tsx の共有ヘッダ / modals / EquipSetToolbar)。
import { loadCharacters } from './storage';
import { getCurrentUser } from './supabase-client';
import { createShare, loadSharedEquipSet, getShareIdFromUrl } from './share';
import type { SharedEquipSetResult } from './share';
import { equipState } from './equip/equip-store';
import type { EquipSlotData } from './equip/equip-store';
import {
    showSharedEquipSet, setShareHeader, showShareLoadError,
} from './equip/equip-sets-store';
import { openShareUrlModal, openImportShareModal } from './modals/modal-store';

// 共有閲覧モード判定 (?share=<uuid>)
const _sharedId = getShareIdFromUrl();
let _sharedEquipSet: SharedEquipSetResult | null = null;

export function isShareMode(): boolean {
    return _sharedId != null;
}

interface CharacterSnapshot {
    race?: string;
    job_levels?: Record<string, { level?: number; master_lv?: number }>;
}

// ===== 共有閲覧モード =====
// ?share=<id> 付きで開かれた場合に通常 UI の代わりに呼ばれ、共有データを読み込む
export async function enterShareMode() {
    try {
        _sharedEquipSet = await loadSharedEquipSet(_sharedId!);
        const shared = _sharedEquipSet;
        // ヘッダー反映
        // 例: "Hum WAR99/SAM59 ML50"
        // ジョブキーは main が "War"、support が "war" と大小文字が混在しているので正規化する
        const normJobKey = (k: string | null | undefined) =>
            k ? k.charAt(0).toUpperCase() + k.slice(1).toLowerCase() : '';
        const charSummary = (() => {
            const snap = shared.characterSnapshot as CharacterSnapshot | null;
            if (!snap) return '';
            const race = snap.race || '?';
            const mainKey = normJobKey(shared.job);
            const supKey = normJobKey(shared.support_job);
            const jl = snap.job_levels || {};
            const mainJl = jl[mainKey] || { level: 0, master_lv: 0 };
            const supJl = supKey ? (jl[supKey] || { level: 0 }) : null;
            const mainStr = mainKey ? `${mainKey.toUpperCase()}${mainJl.level || ''}` : '';
            const supStr = supJl && supKey ? `/${supKey.toUpperCase()}${supJl.level || ''}` : '';
            const mlStr = mainJl.master_lv ? ` ML${mainJl.master_lv}` : '';
            return `${race} ${mainStr}${supStr}${mlStr}`.trim();
        })();
        setShareHeader(
            shared.name || '(無名)',
            ` / 共有元: ${shared.characterName || '?'}` +
                (charSummary ? ` / ${charSummary}` : '')
        );
        // ステータス再現用 character snapshot を override にセット
        equipState.sharedCharacterOverride = shared.characterSnapshot || null;
        equipState.currentEquipChar = shared.characterName || '(共有元)';
        equipState.currentEquipJob = shared.job || '';
        equipState.currentEquipSupportJob = shared.support_job || '';
        // 装備編集フォーム (React) に流し込んで描画。
        // インポートボタンの表示もこの中 (shareMode フラグ) で行われる
        showSharedEquipSet({
            name: shared.name,
            slots: (shared.slots || {}) as Record<string, EquipSlotData | null | undefined>,
        });
    } catch (e) {
        console.error('failed to load shared equipset:', e);
        showShareLoadError(e instanceof Error ? e.message : String(e));
    }
}

// ===== 共有 (作成側): 編集中の装備セットを shared_equipsets テーブルに INSERT =====
// ツールバーの「共有」ボタン (React、web/src/equip/EquipSetToolbar.tsx) から呼ばれる。
export async function shareCurrentEquipSet() {
    if (!getCurrentUser()) {
        alert('共有するにはログインが必要です。');
        return;
    }
    if (!equipState.editingEquipSetName) {
        alert('共有する装備セットを開いてから実行してください。');
        return;
    }
    const equipSet = {
        name: equipState.editingEquipSetName,
        character: equipState.currentEquipChar,
        job: equipState.currentEquipJob,
        support_job: equipState.currentEquipSupportJob || null,
        slots: { ...equipState.currentEquipSlots },
    };
    // 閲覧側でステータス再現するためキャラクター snapshot を同梱
    const characters = await loadCharacters();
    const charSnapshot =
        characters.find((c: { name: string }) => c.name === equipState.currentEquipChar) || null;
    try {
        const url = await createShare(equipSet, charSnapshot);
        openShareUrlModal(url);
    } catch (e) {
        console.error('createShare failed:', e);
        alert('共有 URL の発行に失敗しました: ' + (e instanceof Error ? e.message : e));
    }
}

// ===== インポート (閲覧側): 共有装備セットを自分のキャラ + ジョブにコピー =====
// モーダル本体 (キャラ + ジョブ + 名前の選択と確定処理) は React 側
// (web/src/modals/Modals.tsx)。ここは前提チェックとデータの受け渡しのみ。
export async function beginImportShare() {
    if (!_sharedEquipSet) return;
    if (!getCurrentUser()) {
        if (confirm('インポートにはログインが必要です。Google でログインしますか?')) {
            const { signInWithGoogle } = await import('./supabase-client');
            await signInWithGoogle();
        }
        return;
    }
    const characters = await loadCharacters();
    if (characters.length === 0) {
        alert('先に「キャラクター管理」タブからキャラクターを作成してください。');
        return;
    }
    openImportShareModal({ characters, shared: _sharedEquipSet });
}
