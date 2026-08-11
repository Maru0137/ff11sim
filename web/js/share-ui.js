// 装備セット共有の UI 層。共有 URL の発行 (作成側)・共有閲覧モード
// (?share=<uuid>)・インポート (閲覧側) のモーダルと配線を持つ。
// Supabase との読み書きは js/share.js (docs/adr/0008)。
import { JOBS, RACE_NAMES } from './constants.js';
import { loadCharacters, loadEquipSets, saveEquipSets } from './storage.js';
import { getCurrentUser } from './supabase-client.js';
import { createShare, loadSharedEquipSet, getShareIdFromUrl } from './share.js';
import { equipState } from './equip-state.js';
import { showEquipSetEditForm } from './equip-sets.js';

// 共有閲覧モード判定 (?share=<uuid>)
const _sharedId = getShareIdFromUrl();
let _sharedEquipSet = null;

export function isShareMode() {
    return _sharedId != null;
}

// ===== 共有閲覧モード =====
// ?share=<id> 付きで開かれた場合に通常 UI の代わりに呼ばれ、共有データを読み込む
export async function enterShareMode() {
    document.body.classList.add('share-mode');
    // 装備セットタブを active に切り替え
    document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));
    document.getElementById('tab-equipsets').classList.add('active');
    try {
        _sharedEquipSet = await loadSharedEquipSet(_sharedId);
        // ヘッダー反映
        document.getElementById('sharedHeader').style.display = '';
        document.getElementById('sharedSetName').textContent = _sharedEquipSet.name || '(無名)';
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
        document.getElementById('sharedSetMeta').textContent =
            ` / 共有元: ${_sharedEquipSet.characterName || '?'}` +
            (charSummary ? ` / ${charSummary}` : '');
        // インポートボタンを表示
        document.getElementById('btnImportShare').style.display = '';
        // ステータス再現用 character snapshot を override にセット
        equipState.sharedCharacterOverride = _sharedEquipSet.characterSnapshot || null;
        // 装備編集フォームに流し込んで描画
        equipState.currentEquipChar = _sharedEquipSet.characterName || '(共有元)';
        equipState.currentEquipJob = _sharedEquipSet.job || '';
        equipState.currentEquipSupportJob = _sharedEquipSet.support_job || '';
        // showEquipSetEditForm は equipSet.slots を読むので合わせて name/slots を持たせる
        showEquipSetEditForm({
            name: _sharedEquipSet.name,
            slots: _sharedEquipSet.slots || {},
        });
    } catch (e) {
        console.error('failed to load shared equipset:', e);
        document.getElementById('equipEditSection').classList.remove('hidden');
        document.getElementById('sharedHeader').style.display = '';
        document.getElementById('sharedSetName').textContent = '読み込みに失敗しました';
        document.getElementById('sharedSetMeta').textContent =
            ` / ${e.message || e}`;
    }
}

export function initShareUI() {
    // ===== 共有 (作成側): 編集中の装備セットを shared_equipsets テーブルに INSERT =====
    document.getElementById('btnShareEquipSet').addEventListener('click', async () => {
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
            document.getElementById('shareUrlInput').value = url;
            document.getElementById('shareUrlModal').classList.remove('hidden');
        } catch (e) {
            console.error('createShare failed:', e);
            alert('共有 URL の発行に失敗しました: ' + (e.message || e));
        }
    });

    document.getElementById('btnCopyShareUrl').addEventListener('click', async () => {
        const input = document.getElementById('shareUrlInput');
        try {
            await navigator.clipboard.writeText(input.value);
            const btn = document.getElementById('btnCopyShareUrl');
            const orig = btn.textContent;
            btn.textContent = 'コピーしました!';
            setTimeout(() => { btn.textContent = orig; }, 1500);
        } catch {
            // フォールバック: 選択するだけ
            input.select();
        }
    });

    document.getElementById('btnCloseShareUrlModal').addEventListener('click', () => {
        document.getElementById('shareUrlModal').classList.add('hidden');
    });

    // ===== インポート (閲覧側): 共有装備セットを自分のキャラ + ジョブにコピー =====
    document.getElementById('btnImportShare').addEventListener('click', async () => {
        if (!_sharedEquipSet) return;
        if (!getCurrentUser()) {
            if (confirm('インポートにはログインが必要です。Google でログインしますか?')) {
                const { signInWithGoogle } = await import('./supabase-client.js');
                await signInWithGoogle();
            }
            return;
        }
        // モーダル: キャラ + ジョブ + 名前 を選択
        const charSel = document.getElementById('importShareCharSelect');
        const jobSel = document.getElementById('importShareJobSelect');
        const nameInput = document.getElementById('importShareNameInput');
        const desc = document.getElementById('importShareDescription');

        const characters = await loadCharacters();
        charSel.innerHTML = '';
        if (characters.length === 0) {
            alert('先に「キャラクター管理」タブからキャラクターを作成してください。');
            return;
        }
        characters.forEach(ch => {
            const opt = document.createElement('option');
            opt.value = ch.name;
            opt.textContent = `${ch.name} (${RACE_NAMES[ch.race] || ch.race})`;
            charSel.appendChild(opt);
        });

        jobSel.innerHTML = '';
        JOBS.forEach(j => {
            const opt = document.createElement('option');
            opt.value = j.key;
            opt.textContent = j.name;
            jobSel.appendChild(opt);
        });
        // 共有元のジョブをデフォルト選択
        if (_sharedEquipSet.job) jobSel.value = _sharedEquipSet.job;

        nameInput.value = _sharedEquipSet.name || '共有装備セット';
        desc.textContent = `共有元: ${_sharedEquipSet.characterName || '(未設定)'} / ${_sharedEquipSet.job || '(未設定)'} / ${_sharedEquipSet.name}`;

        document.getElementById('importShareModal').classList.remove('hidden');
    });

    document.getElementById('btnCancelImportShare').addEventListener('click', () => {
        document.getElementById('importShareModal').classList.add('hidden');
    });

    document.getElementById('btnConfirmImportShare').addEventListener('click', async () => {
        const character = document.getElementById('importShareCharSelect').value;
        const job = document.getElementById('importShareJobSelect').value;
        const name = document.getElementById('importShareNameInput').value.trim();
        if (!character || !job || !name) {
            alert('キャラクター・ジョブ・名前を全て指定してください。');
            return;
        }
        const sets = await loadEquipSets();
        // 重複名は (2), (3) ... を付与して回避
        const used = new Set(
            sets.filter(s => s.character === character && s.job === job).map(s => s.name)
        );
        let finalName = name;
        let i = 2;
        while (used.has(finalName)) finalName = `${name} (${i++})`;

        sets.push({
            name: finalName,
            character,
            job,
            slots: { ...(_sharedEquipSet.slots || {}) },
        });
        try {
            await saveEquipSets(sets);
            alert(`「${finalName}」としてインポートしました。`);
            document.getElementById('importShareModal').classList.add('hidden');
            // share=URL を外して通常モードに戻る
            window.location.href = window.location.origin + window.location.pathname;
        } catch (e) {
            console.error('import failed:', e);
            alert('インポートに失敗しました: ' + (e.message || e));
        }
    });
}
