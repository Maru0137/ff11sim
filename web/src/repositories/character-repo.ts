// Character の永続化を抽象化する repository。
// ゲスト (未ログイン): localStorage に全件 JSON で保存
// ログイン中: Supabase の characters テーブル (RLS で auth.uid() = user_id の行のみ)
//
// 既存の loadCharacters/saveCharacters のシグネチャ互換を保つため
// list() / save(arr) の 2 関数のみを公開。Supabase 側は save 時に
// 既存と差分を取って upsert / delete する。
//
// 配列の順序が一覧の表示順そのもの (docs/adr/0022)。localStorage は JSON 配列なので
// そのまま保たれるが、Supabase 側は position カラムに配列インデックスを書き、
// list() で order('position') して復元する。position は行の列であって data jsonb には
// 入れないため、UI 側が扱うレコードの形は変わらない。
//
// データ形状の型は any のまま (UI 層の CharacterRecord とデータ由来 jsonb の
// 突き合わせは tsify 等での型自動生成時にまとめて行う)。

import { STORAGE_KEY } from '../constants';
import { supabase, getCurrentUser } from '../supabase-client';

export interface CharacterRepo {
    list(): Promise<any[]>;
    save(characters: { name: string }[]): Promise<void>;
}

class LocalCharacterRepo implements CharacterRepo {
    async list() {
        try {
            const data = localStorage.getItem(STORAGE_KEY);
            return data ? JSON.parse(data) : [];
        } catch {
            return [];
        }
    }

    async save(characters: { name: string }[]) {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(characters));
    }
}

class SupabaseCharacterRepo implements CharacterRepo {
    async list() {
        const user = getCurrentUser();
        if (!user) return [];
        const { data, error } = await supabase
            .from('characters')
            .select('name, position, data')
            .eq('user_id', user.id)
            .order('position');
        if (error) {
            console.error('SupabaseCharacterRepo.list failed:', error);
            return [];
        }
        // data jsonb には name 以外の全フィールドを格納している。
        // position は並び順の復元にだけ使い、レコードには載せない。
        return data.map((row) => ({ name: row.name, ...row.data }));
    }

    async save(characters: { name: string }[]) {
        const user = getCurrentUser();
        if (!user) throw new Error('not signed in');

        const existing = await this.list();
        const existingNames = new Set<string>(existing.map((c: { name: string }) => c.name));
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

        // upsert: 全件 (data に name 以外を入れて user_id+name で onConflict)。
        // 配列インデックスを position として保存し、list() の並び順を確定させる。
        const rows = characters.map((c, index) => {
            const { name, ...rest } = c;
            return { user_id: user.id, name, position: index, data: rest };
        });
        if (rows.length > 0) {
            const { error } = await supabase
                .from('characters')
                .upsert(rows, { onConflict: 'user_id,name' });
            if (error) throw error;
        }
    }
}

export function getCharacterRepo(): CharacterRepo {
    return getCurrentUser() ? new SupabaseCharacterRepo() : new LocalCharacterRepo();
}
