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
  // web/ を静的配信する。-c-1 でキャッシュを無効化し、再ビルド後の取り違えを防ぐ。
  webServer: {
    command: 'npx http-server web -p 8000 -c-1 --silent',
    url: 'http://localhost:8000',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
