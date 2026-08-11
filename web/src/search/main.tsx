// search.html のエントリポイント。ページ全体を React でマウントする
// (docs/adr/0012 の段階的移行で最初にページ単位移行した箇所)。
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { mountAuthUI } from '../auth-ui';
import { SearchPage } from './SearchPage';
// ログイン時の localStorage → Supabase 同期 (import の副作用で登録)
import '../sync';

export function startSearchPage() {
    mountAuthUI(document.getElementById('auth-ui')!);
    createRoot(document.getElementById('search-root')!).render(
        <StrictMode>
            <SearchPage />
        </StrictMode>
    );
}
