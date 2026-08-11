// ヘッダーのログイン UI (React 島、docs/adr/0012)。
// 未ログイン: 「Google でログイン」ボタン
// ログイン中: 表示名 + 「ログアウト」ボタン
//
// 認証状態は supabase-client.js の既存ストア (onAuthChange / getCurrentUser)
// に useSyncExternalStore で接続する。onAuthChange は購読直後に 'INITIAL'
// イベントで一度発火するが、余分な通知として無害。
import { useSyncExternalStore } from 'react';
import { createRoot } from 'react-dom/client';
import type { User } from '@supabase/supabase-js';
import {
    getCurrentUser,
    onAuthChange,
    signInWithGoogle,
    signOut,
} from '../js/supabase-client.js';

const subscribe = (callback: () => void): (() => void) => onAuthChange(callback);
const getSnapshot = (): User | null => getCurrentUser();

export function AuthWidget() {
    const user = useSyncExternalStore(subscribe, getSnapshot);
    if (!user) {
        return (
            <button
                type="button"
                className="auth-btn auth-btn-google"
                onClick={() => signInWithGoogle()}
            >
                Google でログイン
            </button>
        );
    }
    const name =
        user.user_metadata?.name || user.user_metadata?.full_name || user.email || 'User';
    return (
        <>
            <span className="auth-user">{name}</span>
            <button type="button" className="auth-btn" onClick={() => signOut()}>
                ログアウト
            </button>
        </>
    );
}

/**
 * 指定された container 要素にログイン UI を描画する。
 * 認証状態変化時に自動再描画。
 */
export function mountAuthUI(container: HTMLElement) {
    createRoot(container).render(<AuthWidget />);
}
