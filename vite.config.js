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

export default defineConfig({
    root: 'web',
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
});
