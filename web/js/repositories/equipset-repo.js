// EquipSet の永続化を抽象化する repository。
// 識別キー: (character, job, name) — Supabase 側 unique 制約 (user_id, name, character_name, job)

import { EQUIP_STORAGE_KEY } from '../constants.js';
import { supabase, getCurrentUser } from '../supabase-client.js';

class LocalEquipSetRepo {
    async list() {
        try {
            const data = localStorage.getItem(EQUIP_STORAGE_KEY);
            const sets = data ? JSON.parse(data) : [];
            // 後方互換: 旧データに job/character が無い場合のデフォルト
            return sets.map((s) => ({ job: '', character: '', ...s }));
        } catch {
            return [];
        }
    }

    async save(sets) {
        localStorage.setItem(EQUIP_STORAGE_KEY, JSON.stringify(sets));
    }
}

const setKey = (s) => `${s.character ?? ''}|${s.job ?? ''}|${s.name}`;

class SupabaseEquipSetRepo {
    async list() {
        const user = getCurrentUser();
        if (!user) return [];
        const { data, error } = await supabase
            .from('equipsets')
            .select('name, character_name, job, position, data')
            .eq('user_id', user.id)
            .order('character_name')
            .order('job')
            .order('position');
        if (error) {
            console.error('SupabaseEquipSetRepo.list failed:', error);
            return [];
        }
        return data.map((row) => ({
            name: row.name,
            character: row.character_name ?? '',
            job: row.job ?? '',
            ...row.data,
        }));
    }

    async save(sets) {
        const user = getCurrentUser();
        if (!user) throw new Error('not signed in');

        const existing = await this.list();
        const existingKeys = new Map(existing.map((s) => [setKey(s), s]));
        const newKeys = new Set(sets.map(setKey));

        // 削除
        const toDelete = [...existingKeys.values()].filter((s) => !newKeys.has(setKey(s)));
        if (toDelete.length > 0) {
            // composite key DELETE: 1 件ずつ送る方が PostgREST 的に安全
            for (const s of toDelete) {
                const { error } = await supabase
                    .from('equipsets')
                    .delete()
                    .eq('user_id', user.id)
                    .eq('name', s.name)
                    .eq('character_name', s.character ?? '')
                    .eq('job', s.job ?? '');
                if (error) throw error;
            }
        }

        // upsert: 同じ (character, job) 内のインデックスを position として保存
        const positionByGroup = new Map();
        const rows = sets.map((s) => {
            const { name, character, job, ...rest } = s;
            const groupKey = `${character ?? ''}|${job ?? ''}`;
            const pos = positionByGroup.get(groupKey) ?? 0;
            positionByGroup.set(groupKey, pos + 1);
            return {
                user_id: user.id,
                name,
                character_name: character ?? '',
                job: job ?? '',
                position: pos,
                data: rest,
            };
        });
        if (rows.length > 0) {
            const { error } = await supabase
                .from('equipsets')
                .upsert(rows, { onConflict: 'user_id,name,character_name,job' });
            if (error) throw error;
        }
    }
}

export function getEquipSetRepo() {
    return getCurrentUser() ? new SupabaseEquipSetRepo() : new LocalEquipSetRepo();
}
