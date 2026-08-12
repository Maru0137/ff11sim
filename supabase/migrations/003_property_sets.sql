-- プロパティセット (docs/adr/0015) の保存先。
-- カスタムセットとユーザー定義項目をまとめた PropsetDoc を 1 ユーザー 1 行の
-- data jsonb に格納する。装備セットと違い共有対象外・並び順カラム不要のため、
-- コレクションを行に分解しない。
-- 適用方法: Supabase ダッシュボードの SQL Editor で実行 (他の migration と同様)。

create table public.property_sets (
  user_id uuid primary key references auth.users(id) on delete cascade,
  data jsonb not null default '{}'::jsonb,
  created_at timestamptz default now(),
  updated_at timestamptz default now()
);

alter table public.property_sets enable row level security;

create policy "users can rw own property_sets" on public.property_sets
  for all using (auth.uid() = user_id) with check (auth.uid() = user_id);

create trigger property_sets_set_updated_at
  before update on public.property_sets
  for each row execute procedure public.set_updated_at();
