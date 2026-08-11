// スモークテスト用の静的サーバ。dist/ を本番 (GitHub Pages のプロジェクト
// サイト) と同じ /ff11sim/ サブパス配下で配信する。
//
// vite preview はルート (/) 配信のため、アセット参照がルート絶対パス
// (/assets/...) になるバグを検出できない。実際に base 未設定で本番の全 JS が
// 404 になる障害が起きた (修正: vite.config.js の base: './')。
// サブパス配下で配信すればルート絶対パスの参照は 404 → コンソールエラーに
// なり、スモークの collectErrors が検出する。
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const PORT = 8000;
const PREFIX = '/ff11sim';
const ROOT = fileURLToPath(new URL('../dist', import.meta.url));

const TYPES = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'text/javascript',
    '.json': 'application/json',
    '.wasm': 'application/wasm',
    '.css': 'text/css',
    '.map': 'application/json',
};

createServer(async (req, res) => {
    const { pathname } = new URL(req.url, 'http://localhost');
    // プレフィックス外 (ルート絶対パス参照など) は本番同様に 404
    if (pathname !== PREFIX && pathname !== `${PREFIX}/` && !pathname.startsWith(`${PREFIX}/`)) {
        res.writeHead(404).end('not found');
        return;
    }
    let rel = decodeURIComponent(pathname.slice(PREFIX.length)).replace(/^\/+/, '');
    if (rel === '') rel = 'index.html';
    const file = normalize(join(ROOT, rel));
    if (!file.startsWith(ROOT + sep)) {
        res.writeHead(403).end('forbidden');
        return;
    }
    try {
        const data = await readFile(file);
        res.writeHead(200, {
            'content-type': TYPES[extname(file)] ?? 'application/octet-stream',
            // 再ビルド後の取り違え防止 (旧 http-server の -c-1 相当)
            'cache-control': 'no-store',
        });
        res.end(data);
    } catch {
        res.writeHead(404).end('not found');
    }
}).listen(PORT, () => {
    console.log(`serving dist/ at http://localhost:${PORT}${PREFIX}/`);
});
