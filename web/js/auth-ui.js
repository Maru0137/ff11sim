// ヘッダーのログイン UI。
// 未ログイン: 「Google でログイン」ボタン
// ログイン中: 表示名 + 「ログアウト」ボタン

import { getCurrentUser, onAuthChange, signInWithGoogle, signOut } from './supabase-client.js';

/**
 * 指定された container 要素にログイン UI を描画する。
 * 認証状態変化時に自動再描画。
 * @param {HTMLElement} container
 */
export function mountAuthUI(container) {
    function render() {
        const user = getCurrentUser();
        container.innerHTML = '';
        if (user) {
            const name = user.user_metadata?.name
                || user.user_metadata?.full_name
                || user.email
                || 'User';
            const span = document.createElement('span');
            span.className = 'auth-user';
            span.textContent = name;
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'auth-btn';
            btn.textContent = 'ログアウト';
            btn.addEventListener('click', () => signOut());
            container.appendChild(span);
            container.appendChild(btn);
        } else {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'auth-btn auth-btn-google';
            btn.textContent = 'Google でログイン';
            btn.addEventListener('click', () => signInWithGoogle());
            container.appendChild(btn);
        }
    }
    onAuthChange(render);
}
