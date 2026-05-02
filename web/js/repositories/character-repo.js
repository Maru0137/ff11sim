// Character の永続化を抽象化する repository。
// ゲスト (未ログイン): localStorage に全件 JSON で保存
// ログイン中: Supabase の characters テーブル (RLS で auth.uid() = user_id の行のみ)
//
// 既存の loadCharacters/saveCharacters のシグネチャ互換を保つため
// list() / save(arr) の 2 関数のみを公開。Supabase 側は save 時に
// 既存と差分を取って upsert / delete する。

import { STORAGE_KEY } from '../constants.js';
import { supabase, getCurrentUser } from '../supabase-client.js';

class LocalCharacterRepo {
    async list() {
        try {
            const data = localStorage.getItem(STORAGE_KEY);
            return data ? JSON.parse(data) : [];
        } catch {
            return [];
        }
    }

    async save(characters) {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(characters));
    }
}

class SupabaseCharacterRepo {
    async list() {
        const user = getCurrentUser();
        if (!user) return [];
        const { data, error } = await supabase
            .from('characters')
            .select('name, data')
            .eq('user_id', user.id);
        if (error) {
            console.error('SupabaseCharacterRepo.list failed:', error);
            return [];
        }
        // data jsonb には name 以外の全フィールドを格納している
        return data.map((row) => ({ name: row.name, ...row.data }));
    }

    async save(characters) {
        const user = getCurrentUser();
        if (!user) throw new Error('not signed in');

        const existing = await this.list();
        const existingNames = new Set(existing.map((c) => c.name));
        const newNames = new Set(characters.map((c) => c.name));

        // 削除: existing にあって new にないもの
        const toDelete = [...existingNames].filter((n) => !newNames.has(n));
        if (toDelete.length > 0) {
            const { error } = await supabase
                .from('characters')
                .delete()
                .eq('user_id', user.id)
                .in('name', toDelete);
            if (error) throw error;
        }

        // upsert: 全件 (data に name 以外を入れて user_id+name で onConflict)
        const rows = characters.map((c) => {
            const { name, ...rest } = c;
            return { user_id: user.id, name, data: rest };
        });
        if (rows.length > 0) {
            const { error } = await supabase
                .from('characters')
                .upsert(rows, { onConflict: 'user_id,name' });
            if (error) throw error;
        }
    }
}

export function getCharacterRepo() {
    return getCurrentUser() ? new SupabaseCharacterRepo() : new LocalCharacterRepo();
}
