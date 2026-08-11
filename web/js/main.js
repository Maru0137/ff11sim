// index.html のエントリポイント。各モジュールの初期化順序だけをここで管理する。
//
// 順序の要点 (元のインライン script の挙動を維持):
//   1. 認証 UI のマウントと全リスナ登録は同期的に行う (WASM ロード完了を待たない)
//   2. WASM 初期化とオーグメントデータ取得を並行で待つ
//   3. ?share= 付きなら共有閲覧モードへ分岐し、通常初期化はスキップする
import { initWasmRuntime } from './wasm.js';
import { mountAuthUI } from '../src/auth-ui';
import { initTabs } from './tabs.js';
import { loadAugmentData } from './augments.js';
import { mountModals } from '../src/modals/Modals';
import { mountStatusPanel } from '../src/status/StatusPanel';
import { mountEquipSlots } from '../src/equip/EquipSlots';
import { mountEquipSetControls } from '../src/equip/EquipSetControls';
import { mountEquipSetToolbar } from '../src/equip/EquipSetToolbar';
import { initEquipSetPanel, refreshEquipSetPanel } from '../src/equip/equip-sets-store';
import { mountCharacterTab } from '../src/character/CharacterTab';
import { reloadCharacterList } from '../src/character/character-store';
import { isShareMode, enterShareMode } from './share-ui.js';
import { onAuthChange } from './supabase-client.js';
import './sync.js';

export async function startApp() {
    mountAuthUI(document.getElementById('auth-ui'));

    mountModals(document.getElementById('modals-root'));
    mountStatusPanel(document.getElementById('status-root'));
    mountEquipSlots(document.getElementById('equipSlotsContainer'));
    mountEquipSetControls(document.getElementById('equipset-controls-root'));
    mountEquipSetToolbar(document.getElementById('equipset-toolbar-root'));
    mountCharacterTab(document.getElementById('characters-root'));
    initTabs();

    await Promise.all([
        // items.json の fetch は廃止。WASM に埋め込まれている (docs/adr/0009)。
        initWasmRuntime(),
        loadAugmentData(),
    ]);

    // ===== 共有閲覧モード =====
    if (isShareMode()) {
        await enterShareMode();
        return;
    }

    await initEquipSetPanel();
    await reloadCharacterList();

    // 認証状態が変わったら表示中のリストを再描画
    // (INITIAL は startApp 完了直後に呼ばれて二重実行になるのでスキップ)
    let initialAuthSeen = false;
    onAuthChange(async (_user, event) => {
        if (event === 'INITIAL' && !initialAuthSeen) {
            initialAuthSeen = true;
            return;
        }
        await reloadCharacterList();
        await refreshEquipSetPanel();
    });

    // sync.js が localStorage → Supabase アップロード完了時に発火
    window.addEventListener('ff11sim:synced', async () => {
        await reloadCharacterList();
        await refreshEquipSetPanel();
    });
}
