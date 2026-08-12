// ログイン時に localStorage の characters / equipsets を Supabase へマージ。
// (旧 web/js/sync.js の TS 化)
//
// 競合解決: Supabase に同名行が既にあればスキップ (Supabase 優先)。
// 同期完了後 `ff11sim_synced_<user.id>` フラグを localStorage に立て、
// 同 user で再度同期しないようにする (別ユーザーログインで再 sync 可能)。
//
// 同期完了後は `window.dispatchEvent(new Event('ff11sim:synced'))` を発火。
// index.html / search.html はこれを listen して画面を再描画する。

import type { User } from '@supabase/supabase-js';
import { STORAGE_KEY, EQUIP_STORAGE_KEY, PROPSET_STORAGE_KEY } from './constants';
import { supabase, onAuthChange } from './supabase-client';
import { normalizePropsetDoc } from './propsets/types';

const syncFlagKey = (userId: string) => `ff11sim_synced_${userId}`;
// プロパティセットは後から追加された同期対象のため専用フラグ。
// 既存フラグに相乗りすると、機能追加前に同期済みのユーザーの
// ローカル doc が永久にアップロードされない。
const propsetSyncFlagKey = (userId: string) => `ff11sim_synced_propsets_${userId}`;

interface LocalCharacter {
    name: string;
    [key: string]: unknown;
}

interface LocalEquipSet {
    name: string;
    character?: string;
    job?: string;
    [key: string]: unknown;
}

function readLocalCharacters(): LocalCharacter[] {
    try {
        return JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]');
    } catch {
        return [];
    }
}

function readLocalEquipSets(): LocalEquipSet[] {
    try {
        const sets = JSON.parse(localStorage.getItem(EQUIP_STORAGE_KEY) || '[]');
        return sets.map((s: LocalEquipSet) => ({ job: '', character: '', ...s }));
    } catch {
        return [];
    }
}

async function syncLocalToSupabase(user: User): Promise<{ uploaded: number }> {
    const flagKey = syncFlagKey(user.id);
    if (localStorage.getItem(flagKey)) return { uploaded: 0 };

    const localChars = readLocalCharacters();
    const localSets = readLocalEquipSets();

    if (localChars.length === 0 && localSets.length === 0) {
        localStorage.setItem(flagKey, '1');
        return { uploaded: 0 };
    }

    // Supabase 既存データの key 一覧を取得
    const { data: existingChars, error: e1 } = await supabase
        .from('characters')
        .select('name')
        .eq('user_id', user.id);
    if (e1) throw e1;
    const supaCharNames = new Set((existingChars ?? []).map((r) => r.name));

    const { data: existingSets, error: e2 } = await supabase
        .from('equipsets')
        .select('name, character_name, job')
        .eq('user_id', user.id);
    if (e2) throw e2;
    const supaSetKeys = new Set(
        (existingSets ?? []).map((r) => `${r.character_name ?? ''}|${r.job ?? ''}|${r.name}`),
    );

    // 衝突しない characters を insert
    const charsToUpload = localChars.filter((c) => !supaCharNames.has(c.name));
    if (charsToUpload.length > 0) {
        const rows = charsToUpload.map((c) => {
            const { name, ...rest } = c;
            return { user_id: user.id, name, data: rest };
        });
        const { error } = await supabase.from('characters').insert(rows);
        if (error) throw error;
    }

    // 衝突しない equipsets を insert
    // position は (character, job) 内で「Supabase 既存件数からの続き」を割り当て
    const supaPositionByGroup = new Map<string, number>();
    for (const r of existingSets ?? []) {
        const k = `${r.character_name ?? ''}|${r.job ?? ''}`;
        supaPositionByGroup.set(k, (supaPositionByGroup.get(k) ?? 0) + 1);
    }
    const setsToUpload = localSets.filter(
        (s) => !supaSetKeys.has(`${s.character ?? ''}|${s.job ?? ''}|${s.name}`),
    );
    if (setsToUpload.length > 0) {
        const rows = setsToUpload.map((s) => {
            const { name, character, job, ...rest } = s;
            const k = `${character ?? ''}|${job ?? ''}`;
            const pos = supaPositionByGroup.get(k) ?? 0;
            supaPositionByGroup.set(k, pos + 1);
            return {
                user_id: user.id,
                name,
                character_name: character ?? '',
                job: job ?? '',
                position: pos,
                data: rest,
            };
        });
        const { error } = await supabase.from('equipsets').insert(rows);
        if (error) throw error;
    }

    localStorage.setItem(flagKey, '1');
    return { uploaded: charsToUpload.length + setsToUpload.length };
}

// プロパティセット (docs/adr/0015) の同期。競合解決は他と同じ Supabase 優先:
// 行が既にあればローカル doc はアップロードしない (docs/adr/0006)。
async function syncLocalPropsetsToSupabase(user: User): Promise<{ uploaded: number }> {
    const flagKey = propsetSyncFlagKey(user.id);
    if (localStorage.getItem(flagKey)) return { uploaded: 0 };

    let localDoc;
    try {
        localDoc = normalizePropsetDoc(JSON.parse(localStorage.getItem(PROPSET_STORAGE_KEY) || 'null'));
    } catch {
        localDoc = normalizePropsetDoc(null);
    }
    if (localDoc.sets.length === 0 && localDoc.userItems.length === 0) {
        localStorage.setItem(flagKey, '1');
        return { uploaded: 0 };
    }

    const { data: existing, error: e1 } = await supabase
        .from('property_sets')
        .select('user_id')
        .eq('user_id', user.id)
        .maybeSingle();
    if (e1) throw e1;

    if (!existing) {
        const { error } = await supabase
            .from('property_sets')
            .insert({ user_id: user.id, data: localDoc });
        if (error) throw error;
    }

    localStorage.setItem(flagKey, '1');
    return { uploaded: existing ? 0 : 1 };
}

let _syncing = false;
onAuthChange(async (user, _event) => {
    if (!user) return;
    if (_syncing) return;
    _syncing = true;
    try {
        const { uploaded: mainUploaded } = await syncLocalToSupabase(user);
        const { uploaded: propsetUploaded } = await syncLocalPropsetsToSupabase(user);
        const uploaded = mainUploaded + propsetUploaded;
        if (uploaded > 0) {
            console.log(`[sync] uploaded ${uploaded} items to Supabase`);
        }
        // sync 完了 (件数 0 でも) → 画面再描画依頼
        window.dispatchEvent(new Event('ff11sim:synced'));
    } catch (e) {
        console.error('[sync] failed:', e);
    } finally {
        _syncing = false;
    }
});
