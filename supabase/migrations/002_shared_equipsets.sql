-- 装備セット共有機能用テーブル。
-- 共有 URL 形式: <site>/?share=<id>
--
-- ポリシー:
--   - 全員 (anon 含む) SELECT 可  → 共有 URL を知っていれば誰でも閲覧
--   - 認証ユーザーのみ INSERT 可  → スパム抑止
--   - 作成者のみ DELETE 可        → 将来の管理画面用
--
-- 適用方法: Supabase Dashboard → SQL Editor で本ファイルを Run

create table if not exists public.shared_equipsets (
  id uuid primary key default gen_random_uuid(),
  user_id uuid references auth.users(id) on delete set null,
  name text not null,
  character_name text,
  job text,
  data jsonb not null,
  created_at timestamptz default now()
);

create index if not exists shared_equipsets_user_id_idx
  on public.shared_equipsets (user_id);

alter table public.shared_equipsets enable row level security;

drop policy if exists "anyone can read shared equipsets" on public.shared_equipsets;
create policy "anyone can read shared equipsets" on public.shared_equipsets
  for select using (true);

drop policy if exists "authenticated users can create shared equipsets" on public.shared_equipsets;
create policy "authenticated users can create shared equipsets" on public.shared_equipsets
  for insert with check (auth.uid() = user_id);

drop policy if exists "owner can delete shared equipsets" on public.shared_equipsets;
create policy "owner can delete shared equipsets" on public.shared_equipsets
  for delete using (auth.uid() = user_id);
