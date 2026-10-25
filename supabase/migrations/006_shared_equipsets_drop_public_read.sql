-- 共有の読み出しを「共有 ID を知っている 1 件だけ」に絞る (2/2)。docs/internal/adr/0008
--
-- 誰でも全件を読めていた公開 SELECT ポリシーを外す。以後、作成者以外は
-- public.get_shared_equipset(share_id) (005) を通してしか共有を読めない。
--
-- 適用方法: Supabase Dashboard → SQL Editor で本ファイルを Run
-- 【重要】005 を適用し、それを使うクライアントが本番に反映された後に適用すること。
--         先に適用すると、旧クライアントでは共有 URL を開けなくなる。

drop policy if exists "anyone can read shared equipsets" on public.shared_equipsets;
