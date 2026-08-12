// index.html のエントリポイント (旧 web/js/main.js)。初期化順序をここで管理する。
//
// 順序の要点 (元のインライン script の挙動を維持):
//   1. ページの React root マウントと全リスナ登録は同期的に行う
//      (WASM ロード完了を待たない)
//   2. WASM 初期化とオーグメントデータ取得を並行で待つ
//   3. ?share= 付きなら共有閲覧モードへ分岐し、通常初期化はスキップする
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { MantineProvider } from '@mantine/core';
// CSS は Mantine → 既存 index.css の順に読み、既存スタイルを後勝ちにする
import '@mantine/core/styles.css';
import '../styles/index.css';
import { theme, cssVariablesResolver } from './theme';
import { initWasmRuntime } from './wasm';
import { loadAugmentData } from './augments';
import { isShareMode, enterShareMode } from './share-ui';
import { onAuthChange } from './supabase-client';
import { initEquipSetPanel, refreshEquipSetPanel } from './equip/equip-sets-store';
import { reloadCharacterList } from './character/character-store';
import { loadPropsets } from './propsets/propsets-store';
import { updateEquipEditStatus } from './status/status-store';
import { App } from './App';
// ログイン時の localStorage → Supabase 同期 (import の副作用で登録)
import './sync';

export async function startApp() {
    // 共有閲覧モードの CSS (編集系 UI の非表示) は描画前に効かせる
    if (isShareMode()) {
        document.body.classList.add('share-mode');
    }
    // StrictMode は開発時のみ二重実行等の検査を行う (本番ビルドでは no-op)
    createRoot(document.getElementById('app-root')!).render(
        <StrictMode>
            <MantineProvider
                theme={theme}
                cssVariablesResolver={cssVariablesResolver}
                forceColorScheme="dark"
            >
                <App />
            </MantineProvider>
        </StrictMode>
    );

    await Promise.all([
        // items.json の fetch は廃止。WASM に埋め込まれている (docs/adr/0009)。
        initWasmRuntime(),
        loadAugmentData(),
    ]);

    // ===== 共有閲覧モード =====
    if (isShareMode()) {
        // 閲覧者自身のプロパティセットは共有画面でも使える
        await loadPropsets();
        await enterShareMode();
        return;
    }

    await loadPropsets();
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
        // repo が Local/Supabase で切り替わるため取り直し、ユーザー定義項目の値を再計算
        await loadPropsets();
        await updateEquipEditStatus();
    });

    // sync.js が localStorage → Supabase アップロード完了時に発火
    window.addEventListener('ff11sim:synced', async () => {
        await reloadCharacterList();
        await refreshEquipSetPanel();
        await loadPropsets();
        await updateEquipEditStatus();
    });
}
