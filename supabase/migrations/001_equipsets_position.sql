-- equipsets テーブルに position カラムを追加 (タブ並び順保存用)
-- 既存データは created_at 順で初期 position を埋める
--
-- 適用方法: Supabase Dashboard → SQL Editor で本ファイルを Run

alter table public.equipsets
  add column if not exists position int not null default 0;

-- 既存行の position を (user_id, character_name, job) 内で created_at 順に振り直す
with ranked as (
  select id,
         row_number() over (
           partition by user_id, character_name, job
           order by created_at, id
         ) - 1 as rn
  from public.equipsets
)
update public.equipsets e
set position = ranked.rn
from ranked
where e.id = ranked.id;
