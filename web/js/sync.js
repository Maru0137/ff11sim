// ログイン時に localStorage の characters / equipsets を Supabase へマージ。
//
// 競合解決: Supabase に同名行が既にあればスキップ (Supabase 優先)。
// 同期完了後 `ff11sim_synced_<user.id>` フラグを localStorage に立て、
// 同 user で再度同期しないようにする (別ユーザーログインで再 sync 可能)。
//
// 同期完了後は `window.dispatchEvent(new Event('ff11sim:synced'))` を発火。
// index.html / search.html はこれを listen して画面を再描画する。

import { STORAGE_KEY, EQUIP_STORAGE_KEY } from './constants.js';
import { supabase, onAuthChange } from './supabase-client.js';

const syncFlagKey = (userId) => `ff11sim_synced_${userId}`;

function readLocalCharacters() {
    try {
        return JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]');
    } catch {
        return [];
    }
}

function readLocalEquipSets() {
    try {
        const sets = JSON.parse(localStorage.getItem(EQUIP_STORAGE_KEY) || '[]');
        return sets.map((s) => ({ job: '', character: '', ...s }));
    } catch {
        return [];
    }
}

async function syncLocalToSupabase(user) {
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
    const supaPositionByGroup = new Map();
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

let _syncing = false;
onAuthChange(async (user, _event) => {
    if (!user) return;
    if (_syncing) return;
    _syncing = true;
    try {
        const { uploaded } = await syncLocalToSupabase(user);
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
