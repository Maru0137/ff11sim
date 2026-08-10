---
status: accepted
date: 2026-08-02
decision-makers: Akira Maruoka
---

# 0007. Supabase anon key は CI で `config.js` を生成して注入し、リポジトリにコミットしない

## Context and Problem Statement

[ADR 0005](0005-localstorage-default-supabase-optin.md) で Supabase を使うと決めたが、
ff11sim はサーバーを持たない静的サイトである（[ADR 0001](0001-rust-wasm-static-site.md)）。
つまり Supabase の URL と anon key はどうやってもブラウザに露出する。
それを前提とした上で、鍵をリポジトリに置くのか、ユーザーデータの分離を何によって
保証するのかを決める必要がある。

## Decision Drivers

* ユーザー間でデータが混ざらないこと（他人のキャラクターが読めてはならない）
* 鍵のローテーションが現実的なコストで行えること
* パブリックリポジトリの履歴に秘匿情報を残さないこと
* ローカル開発で認証機能を試せること

## Considered Options

1. `web/js/config.js` に実値を書いてコミットする
2. CI が secrets から `config.js` を生成し、リポジトリには雛形だけを置く
3. 認証機能を使わない（[ADR 0005](0005-localstorage-default-supabase-optin.md) の見直し）

## Decision Outcome

選択: **2. CI が secrets から `config.js` を生成する**（採用）。
そのうえで、**データの安全性は Supabase の Row Level Security に全面的に依存させる**。

anon key は設計上公開される値であり、漏洩自体が直接の被害ではない。
しかしリポジトリにコミットすると git 履歴に永久に残り、ローテーション時に
過去の値が消せない。生成物として扱えば、secrets を差し替えるだけで切り替えられる。

- `web/js/config.js` は `.gitignore` に登録する。雛形として
  `web/js/config.example.js` を置き、ローカル開発ではこれをコピーして使う。
- `.github/workflows/deploy.yml` が `SUPABASE_URL` / `SUPABASE_ANON_KEY` の
  GitHub Secrets から `config.js` を生成する。
- Supabase の全テーブルで RLS を有効化する。ユーザー所有データ（`profiles` /
  `characters` / `equipsets`）は `auth.uid() = user_id`（`profiles` は `auth.uid() = id`）で
  自分の行のみに制限する。
- service role key はブラウザにもリポジトリにも置かない。
- anon key は「公開される値」として扱い、秘匿を前提にした設計をしない。

### Consequences

* Good: git 履歴に鍵が残らず、ローテーションが secrets の更新だけで済む。
* Good: ユーザー間の分離が、クライアントコードではなく DB 側で強制される。
  フロントエンドのバグでは他人のデータを読めない。
* Bad: 安全性が RLS ポリシーの正しさに完全に依存する。ポリシーを 1 つ書き間違えれば
  全ユーザーのデータが読める状態になりうる。
* Bad: `supabase/schema.sql` と `supabase/migrations/*.sql` は Supabase Dashboard の
  SQL Editor で **手動適用** する運用のため、リポジトリ上の定義と実 DB の状態が
  ずれても誰も気付けない。
* Bad: `config.js` を生成していないローカル環境では `supabase-client.js` の import が
  失敗し、それを import している画面が動かない。
* Neutral: anon key はデプロイされた `config.js` から誰でも読める。これは意図した状態。
* Neutral: Supabase SDK は CDN (`https://esm.sh/@supabase/supabase-js@2`) から読み込み、
  npm セットアップを不要にしている。バージョンは URL に固定されている。

### Confirmation

* `.gitignore` に `web/js/config.js` が登録されており、実値がコミットされない。
* `.github/workflows/deploy.yml` の "Generate Supabase config" ステップが
  GitHub Secrets から `config.js` を生成する。
* `supabase/schema.sql` が 5 テーブルすべてに `enable row level security` を適用し、
  `profiles` / `characters` / `equipsets` に
  `for all using (auth.uid() = user_id) with check (auth.uid() = user_id)`
  相当のポリシーを定義している。
* `web/js/config.example.js` が雛形として存在し、README にセットアップ手順がある。

検証されていないもの:

* リポジトリの SQL が実 DB に適用されているかを CI やテストで検証していない。
  スキーマ適用は手動であり、ドリフトを検出する仕組みがない。
* RLS ポリシーが意図通りに他ユーザーのデータを遮断するかを検証するテストがない。
* secrets が未設定のまま deploy した場合、`config.js` は空文字で生成され、
  ビルドは成功するが実行時に失敗する。これを止めるチェックはない。

## Pros and Cons of the Options

### 1. `web/js/config.js` に実値を書いてコミットする

* Good: セットアップが不要。チェックアウトすればそのまま動く。
* Good: CI に secrets を設定する手間がない。
* Bad: 鍵が git 履歴に永久に残る。ローテーションしても過去の値は消せない。
* Bad: 開発用と本番用で異なる Supabase プロジェクトを使いたくなったとき、切り替え手段がない。
* Bad: 一度コミットすると、後から「置かない」方針に戻しても履歴からは消えない。

### 2. CI が secrets から `config.js` を生成する（採用）

* Good: git 履歴に鍵が残らず、ローテーションが secrets の更新だけで済む。
* Good: 環境ごとに異なる値を注入できる。
* Bad: ローカル開発では `config.example.js` をコピーする手順が増える。
  生成しないと `supabase-client.js` の import が失敗する。
* Bad: secrets が未設定でもビルドは成功し、実行時に初めて失敗する。
* Bad: 生成された `config.js` の中身は結局配信されて公開されるため、
  秘匿の効果は「履歴汚染の回避」に限られる。安全性そのものは RLS 頼みで変わらない。

### 3. 認証機能を使わない

* Good: 鍵の管理という問題自体が消える。
  [ADR 0001](0001-rust-wasm-static-site.md) のサーバーレス志向とも最も整合する。
* Good: 攻撃面が存在しない。ユーザーデータの漏洩リスクがゼロになる。
* Bad: 端末間共有と装備セット共有 ([ADR 0008](0008-equipset-sharing.md)) が実現できず、
  [ADR 0005](0005-localstorage-default-supabase-optin.md) の判断をやり直すことになる。

## More Information

* 保存先の切り替え: [ADR 0005](0005-localstorage-default-supabase-optin.md)
* 公開 SELECT を許す例外テーブル: [ADR 0008](0008-equipset-sharing.md)
* セットアップ手順: [web/README.md](../../web/README.md)
