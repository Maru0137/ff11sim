-- ff11sim Supabase スキーマ
-- 適用方法: Supabase Dashboard → SQL Editor で本ファイルを貼り付けて Run
-- 冪等: drop は行わない。再実行時は CREATE 文がエラーになる場合があるが、
--       想定運用は初回のみ手動適用 + 以後 schema 変更時に追記用 SQL を作る。

-- ---------------------------------------------------------------------------
-- 1. profiles: auth.users と 1:1 で表示名を保持
-- ---------------------------------------------------------------------------
create table public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  display_name text,
  created_at timestamptz default now()
);

-- ---------------------------------------------------------------------------
-- 2. characters: ユーザーのキャラクターデータ
--    既存 localStorage の Character オブジェクトをそのまま data jsonb に格納
-- ---------------------------------------------------------------------------
create table public.characters (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  name text not null,
  data jsonb not null,
  created_at timestamptz default now(),
  updated_at timestamptz default now(),
  unique (user_id, name)
);
create index characters_user_id_idx on public.characters (user_id);

-- ---------------------------------------------------------------------------
-- 3. equipsets: 装備セット。character_name で characters と緩く紐付け
--    (FK 制約は付けない。キャラ名変更時の対応簡略化のため)
-- ---------------------------------------------------------------------------
create table public.equipsets (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  name text not null,
  character_name text,
  job text,
  position int not null default 0,
  data jsonb not null,
  created_at timestamptz default now(),
  updated_at timestamptz default now(),
  unique (user_id, name, character_name, job)
);
create index equipsets_user_idx on public.equipsets (user_id, character_name);

-- ---------------------------------------------------------------------------
-- 4. items: 全ユーザー共通の参照データ。SELECT のみ全員許可
--    INSERT/UPDATE/DELETE は service role のみ (CI でのインポート用)
-- ---------------------------------------------------------------------------
create table public.items (
  id integer primary key,
  data jsonb not null
);

-- ---------------------------------------------------------------------------
-- Row Level Security
-- ---------------------------------------------------------------------------
alter table public.profiles enable row level security;
alter table public.characters enable row level security;
alter table public.equipsets enable row level security;
alter table public.items enable row level security;

create policy "users can rw own profile" on public.profiles
  for all using (auth.uid() = id) with check (auth.uid() = id);

create policy "users can rw own characters" on public.characters
  for all using (auth.uid() = user_id) with check (auth.uid() = user_id);

create policy "users can rw own equipsets" on public.equipsets
  for all using (auth.uid() = user_id) with check (auth.uid() = user_id);

create policy "anyone can read items" on public.items
  for select using (true);

-- ---------------------------------------------------------------------------
-- updated_at 自動更新 trigger
-- ---------------------------------------------------------------------------
create or replace function public.set_updated_at()
returns trigger as $$
begin
  new.updated_at = now();
  return new;
end;
$$ language plpgsql;

create trigger characters_set_updated_at
  before update on public.characters
  for each row execute procedure public.set_updated_at();

create trigger equipsets_set_updated_at
  before update on public.equipsets
  for each row execute procedure public.set_updated_at();

-- ---------------------------------------------------------------------------
-- auth.users insert 時に profiles を自動作成
-- ---------------------------------------------------------------------------
create or replace function public.handle_new_user()
returns trigger as $$
begin
  insert into public.profiles (id, display_name)
  values (new.id, coalesce(new.raw_user_meta_data->>'name', new.email));
  return new;
end;
$$ language plpgsql security definer;

create trigger on_auth_user_created
  after insert on auth.users
  for each row execute procedure public.handle_new_user();
