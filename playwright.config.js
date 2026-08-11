// スモークテストの設定。
//
// 目的は「値が正しいか」ではなく「ページが機能しているか」の確認。
// Rust 側のユニットテストが全て通っていても、JS の参照漏れや WASM の
// 初期化失敗でページ全体が動かないことがある (実際に起きた)。
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  // 落ちたときに原因が分かるよう、失敗時のみトレースを残す。
  use: {
    baseURL: 'http://localhost:8000/ff11sim/',
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  // 配信物 (vite build の成果物) を、本番 (GitHub Pages) と同じ
  // /ff11sim/ サブパス配下で配信して検証する (tests/serve-dist.mjs)。
  // ルート配信 (vite preview) ではルート絶対パスのアセット参照が
  // 通ってしまい、本番だけ 404 になるバグを検出できない (実際に起きた)。
  webServer: {
    command: 'npm run build && node tests/serve-dist.mjs',
    url: 'http://localhost:8000/ff11sim/',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
