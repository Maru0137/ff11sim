// Supabase 接続設定のテンプレ。
// ローカル開発:
//   このファイルを `config.js` にコピーして実際の値を埋める (config.js は .gitignore)
// CI (GitHub Actions):
//   Repository Secrets の SUPABASE_URL / SUPABASE_ANON_KEY から
//   `web/js/config.js` を生成する (.github/workflows/deploy.yml 参照)
//
// anon key は公開しても問題ない (RLS でユーザー間のデータが分離されるため)。
// ただし git 履歴に残ると key ローテーションが面倒なので config.js は commit しない。

export const SUPABASE_URL = 'https://YOUR_PROJECT.supabase.co';
export const SUPABASE_ANON_KEY = 'YOUR_ANON_KEY';
