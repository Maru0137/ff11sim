// 装備セット共有の UI 層。共有 URL の発行 (作成側)・共有閲覧モード
// (?share=<uuid>)・インポート (閲覧側) のモーダルと配線を持つ。
// Supabase との読み書きは js/share.js (docs/adr/0008)。
import { loadCharacters } from './storage.js';
import { getCurrentUser } from './supabase-client.js';
import { createShare, loadSharedEquipSet, getShareIdFromUrl } from './share.js';
import { equipState } from './equip-state.js';
import {
    showSharedEquipSet, setShareHeader, showShareLoadError,
} from '../src/equip/equip-sets-store';
import { openShareUrlModal, openImportShareModal } from '../src/modals/modal-store';

// 共有閲覧モード判定 (?share=<uuid>)
const _sharedId = getShareIdFromUrl();
let _sharedEquipSet = null;

export function isShareMode() {
    return _sharedId != null;
}

// ===== 共有閲覧モード =====
// ?share=<id> 付きで開かれた場合に通常 UI の代わりに呼ばれ、共有データを読み込む
export async function enterShareMode() {
    try {
        _sharedEquipSet = await loadSharedEquipSet(_sharedId);
        // ヘッダー反映
        // 例: "Hum WAR99/SAM59 ML50"
        // ジョブキーは main が "War"、support が "war" と大小文字が混在しているので正規化する
        const normJobKey = (k) => k ? k.charAt(0).toUpperCase() + k.slice(1).toLowerCase() : '';
        const charSummary = (() => {
            const snap = _sharedEquipSet.characterSnapshot;
            if (!snap) return '';
            const race = snap.race || '?';
            const mainKey = normJobKey(_sharedEquipSet.job);
            const supKey = normJobKey(_sharedEquipSet.support_job);
            const jl = snap.job_levels || {};
            const mainJl = jl[mainKey] || { level: 0, master_lv: 0 };
            const supJl = supKey ? (jl[supKey] || { level: 0 }) : null;
            const mainStr = mainKey ? `${mainKey.toUpperCase()}${mainJl.level || ''}` : '';
            const supStr = supJl && supKey ? `/${supKey.toUpperCase()}${supJl.level || ''}` : '';
            const mlStr = mainJl.master_lv ? ` ML${mainJl.master_lv}` : '';
            return `${race} ${mainStr}${supStr}${mlStr}`.trim();
        })();
        setShareHeader(
            _sharedEquipSet.name || '(無名)',
            ` / 共有元: ${_sharedEquipSet.characterName || '?'}` +
                (charSummary ? ` / ${charSummary}` : '')
        );
        // ステータス再現用 character snapshot を override にセット
        equipState.sharedCharacterOverride = _sharedEquipSet.characterSnapshot || null;
        equipState.currentEquipChar = _sharedEquipSet.characterName || '(共有元)';
        equipState.currentEquipJob = _sharedEquipSet.job || '';
        equipState.currentEquipSupportJob = _sharedEquipSet.support_job || '';
        // 装備編集フォーム (React) に流し込んで描画。
        // インポートボタンの表示もこの中 (shareMode フラグ) で行われる
        showSharedEquipSet({
            name: _sharedEquipSet.name,
            slots: _sharedEquipSet.slots || {},
        });
    } catch (e) {
        console.error('failed to load shared equipset:', e);
        showShareLoadError(e.message || String(e));
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
    const charSnapshot = characters.find(c => c.name === equipState.currentEquipChar) || null;
    try {
        const url = await createShare(equipSet, charSnapshot);
        openShareUrlModal(url);
    } catch (e) {
        console.error('createShare failed:', e);
        alert('共有 URL の発行に失敗しました: ' + (e.message || e));
    }
}

// ===== インポート (閲覧側): 共有装備セットを自分のキャラ + ジョブにコピー =====
// モーダル本体 (キャラ + ジョブ + 名前の選択と確定処理) は React 側
// (web/src/modals/Modals.tsx)。ここは前提チェックとデータの受け渡しのみ。
export async function beginImportShare() {
    if (!_sharedEquipSet) return;
    if (!getCurrentUser()) {
        if (confirm('インポートにはログインが必要です。Google でログインしますか?')) {
            const { signInWithGoogle } = await import('./supabase-client.js');
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
