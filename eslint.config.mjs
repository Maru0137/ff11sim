// 未定義識別子の検出が主目的。
//
// 装備ロジックを Rust に移した際、削除した JS が持っていた WEAPON_SKILL_KEYS を
// index.html が参照したままになり、本番でステータスが全て 0 になる事故があった。
// この種の参照漏れはコードを動かさずに検出できる。
import globals from 'globals';
import html from 'eslint-plugin-html';

export default [
  {
    // 生成物と依存。整形も命名も生成側の都合なので検査しない。
    ignores: ['web/pkg/**', 'dist/**', 'node_modules/**', 'test-results/**', 'playwright-report/**'],
  },
  {
    // ブラウザで動くコード。インライン <script> を含むため HTML も対象にする。
    // ページ内スクリプトが 1600 行あり、ここを見ないと検査の意味がほぼ無い。
    files: ['web/**/*.js', 'web/**/*.html'],
    plugins: { html },
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: { ...globals.browser },
    },
    rules: {
      'no-undef': 'error',
    },
  },
  {
    // Node で動くコード (テスト・設定ファイル)。
    files: ['tests/**/*.js', '*.config.js', '*.mjs'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: { ...globals.node },
    },
    rules: {
      'no-undef': 'error',
    },
  },
];
