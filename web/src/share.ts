// 装備セット共有機能 (旧 web/js/share.js の TS 化)。
//
// データモデル: shared_equipsets テーブル (DB schema 参照)
//   id (uuid PK), user_id (作成者, nullable), name, character_name, job, data (slots 等), created_at
//
// フロー:
//   1. createShare(equipSet) → shared_equipsets に INSERT → 共有 URL を返す
//   2. ?share=<id> 付き URL でアクセス → loadSharedEquipSet(id) で JSON 取得
//   3. インポート時は呼び出し側で character / job を選ばせて equipset-repo 経由で保存
//
// RLS:
//   - SELECT: 全員許可 (URL を知っていれば閲覧可)
//   - INSERT: 認証ユーザーのみ (auth.uid() = user_id)

import { supabase, getCurrentUser } from '../js/supabase-client.js';

const SHARE_PARAM = 'share';

interface ShareEquipSetInput {
    name?: string;
    character?: string;
    job?: string;
    support_job?: string | null;
    slots?: Record<string, unknown>;
}

export interface SharedEquipSetResult {
    name: string;
    characterName: string;
    job: string;
    support_job?: string | null;
    slots?: Record<string, unknown>;
    /** 共有時のキャラクタースナップショット (なければ null) */
    characterSnapshot: unknown;
    _shared_created_at: string;
}

/**
 * URL の `?share=<uuid>` を取得。なければ null。
 */
export function getShareIdFromUrl(): string | null {
    const params = new URLSearchParams(window.location.search);
    return params.get(SHARE_PARAM);
}

/**
 * 共有ページの URL を組み立てる (現在のサイト origin + path + ?share=id)。
 */
export function buildShareUrl(id: string): string {
    const base = window.location.origin + window.location.pathname;
    return `${base}?${SHARE_PARAM}=${encodeURIComponent(id)}`;
}

/**
 * 装備セットを共有テーブルに INSERT し、共有 URL を返す。
 * ログインユーザーのみ実行可能 (RLS 制約)。
 *
 * 引数の characterSnapshot は閲覧側でステータス計算 (素 + 装備) を再現するため
 * data.character フィールドに格納する。閲覧者は読み取り専用で復元計算のみに利用する。
 */
export async function createShare(
    equipSet: ShareEquipSetInput,
    characterSnapshot: unknown
): Promise<string> {
    const user = getCurrentUser();
    if (!user) throw new Error('not signed in');

    // name / character / job は別カラムに展開、それ以外 (slots 等) は data jsonb に格納
    const { name, character, job, ...rest } = equipSet;
    const dataPayload: Record<string, unknown> = { ...rest };
    if (characterSnapshot) dataPayload.character = characterSnapshot;

    const row = {
        user_id: user.id,
        name: name ?? '',
        character_name: character ?? '',
        job: job ?? '',
        data: dataPayload,
    };

    const { data, error } = await supabase
        .from('shared_equipsets')
        .insert(row)
        .select('id')
        .single();
    if (error) throw error;

    return buildShareUrl(data.id);
}

/**
 * 共有 ID から装備セットを復元。誰でも (未ログインでも) 取得可能。
 */
export async function loadSharedEquipSet(id: string): Promise<SharedEquipSetResult> {
    const { data, error } = await supabase
        .from('shared_equipsets')
        .select('name, character_name, job, data, created_at')
        .eq('id', id)
        .maybeSingle();
    if (error) throw error;
    if (!data) throw new Error('shared equipset not found');

    // data 内に character フィールドがあれば snapshot として展開する。
    // それ以外 (slots, support_job 等) はトップレベルに広げる。
    const { character: characterSnapshot, ...rest } = (data.data ?? {}) as Record<string, unknown>;

    return {
        name: data.name ?? '',
        characterName: data.character_name ?? '',
        job: data.job ?? '',
        ...rest,
        characterSnapshot: characterSnapshot ?? null,
        _shared_created_at: data.created_at,
    };
}
