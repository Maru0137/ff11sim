// index.html のエントリポイント (旧 web/js/main.js)。初期化順序をここで管理する。
//
// 順序の要点 (元のインライン script の挙動を維持):
//   1. ページの React root マウントと全リスナ登録は同期的に行う
//      (WASM ロード完了を待たない)
//   2. WASM 初期化とオーグメントデータ取得を並行で待つ
//   3. ?share= 付きなら共有閲覧モードへ分岐し、通常初期化はスキップする
import { createRoot } from 'react-dom/client';
import { initWasmRuntime } from '../js/wasm.js';
import { loadAugmentData } from '../js/augments.js';
import { isShareMode, enterShareMode } from '../js/share-ui.js';
import { onAuthChange } from '../js/supabase-client.js';
import { initEquipSetPanel, refreshEquipSetPanel } from './equip/equip-sets-store';
import { reloadCharacterList } from './character/character-store';
import { App } from './App';
// ログイン時の localStorage → Supabase 同期 (import の副作用で登録)
import '../js/sync.js';

export async function startApp() {
    // 共有閲覧モードの CSS (編集系 UI の非表示) は描画前に効かせる
    if (isShareMode()) {
        document.body.classList.add('share-mode');
    }
    createRoot(document.getElementById('app-root')!).render(<App />);

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
    onAuthChange(async (_user: unknown, event: string) => {
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
