// Supabase クライアントの初期化と認証ユーティリティ。
// SDK は CDN (esm.sh) 経由で読み込み、npm セットアップを不要にする。
//
// 公開 API:
//   - supabase: createClient のインスタンス (DB クエリに使用)
//   - getCurrentUser(): 現在のユーザー (未ログインなら null)
//   - signInWithGoogle(): OAuth リダイレクト開始
//   - signOut(): ログアウト
//   - onAuthChange(callback): 認証状態変化通知 (signed_in / signed_out)

import { createClient } from 'https://esm.sh/@supabase/supabase-js@2';
import { SUPABASE_URL, SUPABASE_ANON_KEY } from './config.js';

export const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
    auth: {
        persistSession: true,
        autoRefreshToken: true,
        detectSessionInUrl: true,
    },
});

let _currentUser = null;
const _listeners = new Set();

// 初期 session 復元 (リロード後も localStorage に保存された session で復元される)
const { data: { session: _initialSession } } = await supabase.auth.getSession();
_currentUser = _initialSession?.user ?? null;

supabase.auth.onAuthStateChange((event, session) => {
    const prev = _currentUser;
    _currentUser = session?.user ?? null;
    // SIGNED_IN は新規セッション、INITIAL_SESSION は startup 時のリプレイ
    // ユーザーが変わったときだけリスナを呼ぶ
    if (prev?.id !== _currentUser?.id) {
        for (const cb of _listeners) {
            try { cb(_currentUser, event); } catch (e) { console.error(e); }
        }
    }
});

export function getCurrentUser() {
    return _currentUser;
}

export function onAuthChange(callback) {
    _listeners.add(callback);
    // 登録直後に現状を 1 度通知
    try { callback(_currentUser, 'INITIAL'); } catch (e) { console.error(e); }
    return () => _listeners.delete(callback);
}

export async function signInWithGoogle() {
    const redirectTo = window.location.origin + window.location.pathname;
    const { error } = await supabase.auth.signInWithOAuth({
        provider: 'google',
        options: { redirectTo },
    });
    if (error) {
        console.error('signInWithGoogle failed:', error);
        alert('ログインに失敗しました: ' + error.message);
    }
}

export async function signOut() {
    const { error } = await supabase.auth.signOut();
    if (error) {
        console.error('signOut failed:', error);
        alert('ログアウトに失敗しました: ' + error.message);
    }
}
