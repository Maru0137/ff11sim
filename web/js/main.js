// index.html のエントリポイント。各モジュールの初期化順序だけをここで管理する。
//
// 順序の要点 (元のインライン script の挙動を維持):
//   1. 認証 UI のマウントと全リスナ登録は同期的に行う (WASM ロード完了を待たない)
//   2. WASM 初期化とオーグメントデータ取得を並行で待つ
//   3. ?share= 付きなら共有閲覧モードへ分岐し、通常初期化はスキップする
import { initWasmRuntime } from './wasm.js';
import { JOBS } from './constants.js';
import { buildJobLevelTable, renderCharList, initCharacterTab } from './character-list.js';
import { mountAuthUI } from '../src/auth-ui';
import { initTabs } from './tabs.js';
import { loadAugmentData } from './augments.js';
import { buildEquipSlotsUI, initCustomAugHelpModal } from './equip-slots.js';
import {
    renderEquipSetTabs, updateEquipCharSelector, initEquipSetControls,
} from './equip-sets.js';
import { isShareMode, enterShareMode, initShareUI } from './share-ui.js';
import { onAuthChange } from './supabase-client.js';
import './sync.js';

export async function startApp() {
    mountAuthUI(document.getElementById('auth-ui'));

    initTabs();
    initCharacterTab();
    initEquipSetControls();
    initShareUI();
    initCustomAugHelpModal();

    await Promise.all([
        // items.json の fetch は廃止。WASM に埋め込まれている (docs/adr/0009)。
        initWasmRuntime(),
        loadAugmentData(),
    ]);
    buildJobLevelTable();
    buildEquipSlotsUI();

    // Populate equipment set job selector
    const equipJobSel = document.getElementById('equipSelectJob');
    JOBS.forEach(job => {
        const opt = document.createElement('option');
        opt.value = job.key;
        opt.textContent = job.name;
        equipJobSel.appendChild(opt);
    });

    // ===== 共有閲覧モード =====
    if (isShareMode()) {
        await enterShareMode();
        return;
    }

    await updateEquipCharSelector();
    await renderCharList();

    // 認証状態が変わったら表示中のリストを再描画
    // (INITIAL は startApp 完了直後に呼ばれて二重実行になるのでスキップ)
    let initialAuthSeen = false;
    onAuthChange(async (_user, event) => {
        if (event === 'INITIAL' && !initialAuthSeen) {
            initialAuthSeen = true;
            return;
        }
        await renderCharList();
        await renderEquipSetTabs();
    });

    // sync.js が localStorage → Supabase アップロード完了時に発火
    window.addEventListener('ff11sim:synced', async () => {
        await renderCharList();
        await renderEquipSetTabs();
    });
}
