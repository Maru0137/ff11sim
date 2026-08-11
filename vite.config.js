// Vite 設定 (docs/adr/0011)。
//
// - root は web/。index.html / search.html の 2 HTML がエントリ。
// - target: esnext — constants.js / supabase-client.js が top-level await を使う。
// - web/public/data/ は実行時 fetch されるデータ JSON (docs/adr/0002 の
//   シンボリックリンク群 + augments.json)。publicDir 経由でそのまま配信・コピーする。
// - web/pkg/ (wasm-pack --target web の生成物) は ES モジュールとして
//   バンドルされ、.wasm は new URL(..., import.meta.url) 経由でアセット化される。
import { defineConfig } from 'vite';
import { resolve } from 'node:path';
import react from '@vitejs/plugin-react';

export default defineConfig({
    root: 'web',
    // React コンポーネント (web/src/ の .tsx) の変換 (docs/adr/0012)。
    // 既存の web/js/*.js には作用しない。
    plugins: [react()],
    // GitHub Pages のプロジェクトサイトは /<repo>/ 配下で配信されるため、
    // アセット参照をルート絶対パス (デフォルト '/') にすると 404 になる。
    // 配信パスに依存しない相対参照にする。
    base: './',
    // 8000 は Supabase のリダイレクト許可リストに登録済みのポート
    // (web/README.md)。dev / preview とも同じポートに揃える。
    server: {
        port: 8000,
    },
    build: {
        target: 'esnext',
        outDir: '../dist',
        emptyOutDir: true,
        rollupOptions: {
            input: {
                index: resolve(import.meta.dirname, 'web/index.html'),
                search: resolve(import.meta.dirname, 'web/search.html'),
            },
        },
    },
    // Vitest (ユニットテスト)。root: 'web' 基準なので include も web/ 相対。
    // ブラウザ API に依存しない純関数のみを対象にする (ページ全体の検証は
    // Playwright スモークの責務)。
    test: {
        include: ['src/**/*.test.ts'],
        environment: 'node',
    },
});
