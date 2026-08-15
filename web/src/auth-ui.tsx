// ログイン UI (docs/adr/0012)。
// 未ログイン: 「Google でログイン」ボタン
// ログイン中: 表示名 + 「ログアウト」ボタン
//
// 認証状態は supabase-client.js の既存ストア (onAuthChange / getCurrentUser)
// に useSyncExternalStore で接続する。onAuthChange は購読直後に 'INITIAL'
// イベントで一度発火するが、余分な通知として無害。
import { useSyncExternalStore } from 'react';
import type { User } from '@supabase/supabase-js';
import {
    getCurrentUser,
    onAuthChange,
    signInWithGoogle,
    signOut,
} from './supabase-client';
import { guard } from './dirty-guard';

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
            {/* ログアウトは保存先が Supabase → ローカルに切り替わり、一覧と
                編集フォームが読み直されるので未保存の変更は失われる
                (docs/adr/0020)。ログイン側は OAuth リダイレクトでページを
                離れるため beforeunload が受け持つ。 */}
            <button
                type="button"
                className="auth-btn"
                onClick={() => guard('ログアウト', () => void signOut())}
            >
                ログアウト
            </button>
        </>
    );
}
