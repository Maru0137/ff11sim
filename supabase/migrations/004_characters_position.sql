-- characters テーブルに position カラムを追加 (一覧の並び順保存用、docs/adr/0022)
--
-- これまで SupabaseCharacterRepo.list() は ORDER BY 無しで SELECT していたため、
-- ログイン中はキャラクターの並び順がそもそも保存されていなかった。
-- 行を UPDATE すると Postgres の物理順が変わるので、キャラクターを編集して
-- 保存し直すたびに一覧の順番が入れ替わっていた。
--
-- 適用方法: Supabase Dashboard → SQL Editor で本ファイルを Run
-- 【重要】main へのマージは deploy.yml で本番反映されるため、先に本 SQL を適用すること。
--         カラムが無い状態で新しいコードが動くと .order('position') が失敗し、
--         ログイン中のキャラクター一覧が空になる。

alter table public.characters
  add column if not exists position int not null default 0;

-- 既存行の position を user_id 内で振り直す。
-- characters テーブルはリポジトリ管理外で作られており列構成を確定できないため、
-- 常に存在するシステム列 ctid で行を指し、created_at は有無を見てから使う。
do $$
begin
  if exists (
    select 1 from information_schema.columns
    where table_schema = 'public'
      and table_name = 'characters'
      and column_name = 'created_at'
  ) then
    -- 登録順を保つ
    execute $q$
      with ranked as (
        select ctid,
               row_number() over (partition by user_id order by created_at, name) - 1 as rn
        from public.characters
      )
      update public.characters c
      set position = ranked.rn
      from ranked
      where c.ctid = ranked.ctid
    $q$;
  else
    -- created_at が無い場合は名前順。元の並びは不定だったので、
    -- 決定的な順序に落ち着けば十分。
    execute $q$
      with ranked as (
        select ctid,
               row_number() over (partition by user_id order by name) - 1 as rn
        from public.characters
      )
      update public.characters c
      set position = ranked.rn
      from ranked
      where c.ctid = ranked.ctid
    $q$;
  end if;
end $$;
