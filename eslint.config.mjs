// 未定義識別子の検出が主目的。
//
// 装備ロジックを Rust に移した際、削除した JS が持っていた WEAPON_SKILL_KEYS を
// index.html が参照したままになり、本番でステータスが全て 0 になる事故があった。
// この種の参照漏れはコードを動かさずに検出できる。
import globals from 'globals';
import html from 'eslint-plugin-html';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';

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
    // React / TypeScript 層 (web/src/、docs/adr/0012)。
    // no-undef は付けない: TS では未定義識別子を tsc (npm run typecheck) が
    // 検出し、core の no-undef は誤検知源になる (typescript-eslint の推奨)。
    // 既存 JS 層の no-undef と同じ目的 (参照漏れ検出) は typecheck が担う。
    files: ['web/**/*.ts', 'web/**/*.tsx'],
    languageOptions: {
      parser: tseslint.parser,
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: { ...globals.browser },
    },
    plugins: { 'react-hooks': reactHooks },
    rules: {
      // 依存配列ミスは「状態と表示の不整合」という過去の本番障害と
      // 同型の不具合を生むため error にする。
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',
    },
  },
  {
    // Node で動くコード (テスト・設定ファイル)。
    files: ['tests/**/*.js', 'tests/**/*.mjs', '*.config.js', '*.mjs'],
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
