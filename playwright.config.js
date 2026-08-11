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
    baseURL: 'http://localhost:8000',
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  // 配信物 (vite build の成果物) に対して検証する。dev サーバではなく
  // preview を使うことで、バンドル起因の問題もここで検出できる。
  webServer: {
    command: 'npm run build && npm run preview',
    url: 'http://localhost:8000',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
