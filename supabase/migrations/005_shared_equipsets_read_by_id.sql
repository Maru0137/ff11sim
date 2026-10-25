-- 共有の読み出しを「共有 ID を知っている 1 件だけ」に絞る (1/2)。docs/adr/0008
--
-- これまでの SELECT ポリシー `using (true)` は、ID で絞らない select も許していた。
-- そのため anon キー (クライアントに埋め込まれていて誰でも取り出せる) で
-- `select *` を送ると、URL を知らない人でも全員の共有を user_id ごと取得できた。
-- 閲覧は ID を引数に取る関数だけを公開し、関数は user_id を返さない。
--
-- 適用方法: Supabase Dashboard → SQL Editor で本ファイルを Run
-- 【重要】順序: 本ファイルを適用 → このブランチを main にマージ (deploy.yml で本番反映)
--         → 006_shared_equipsets_drop_public_read.sql を適用。
--         本ファイルは旧クライアントを壊さない (公開 SELECT ポリシーはまだ残る)。
--         新しいクライアントは本ファイルの関数が無いと共有を開けない。

create or replace function public.get_shared_equipset(share_id uuid)
returns table (
  name text,
  character_name text,
  job text,
  data jsonb,
  created_at timestamptz
)
language sql
stable
security definer
set search_path = ''
as $$
  select s.name, s.character_name, s.job, s.data, s.created_at
  from public.shared_equipsets s
  where s.id = share_id
$$;

revoke all on function public.get_shared_equipset(uuid) from public;
grant execute on function public.get_shared_equipset(uuid) to anon, authenticated;

-- 作成時の insert ... returning id と、作成者による delete は、行を読む権限を要する。
-- 公開 SELECT ポリシーを外した後もこれらが動くように、作成者には自分の行だけを読ませる。
drop policy if exists "owner can read own shared equipsets" on public.shared_equipsets;
create policy "owner can read own shared equipsets" on public.shared_equipsets
  for select using (auth.uid() = user_id);
