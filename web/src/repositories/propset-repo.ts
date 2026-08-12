// プロパティセット (docs/adr/0015) の永続化を抽象化する repository。
// ゲスト (未ログイン): localStorage に PropsetDoc を JSON で保存
// ログイン中: Supabase の property_sets テーブル (1 ユーザー 1 行、data jsonb)
//
// 装備セットと違い共有対象外・並び順カラム不要のため、コレクションを
// 行に分解せずドキュメント 1 個として扱う (character-repo より単純)。

import { PROPSET_STORAGE_KEY } from '../constants';
import { supabase, getCurrentUser } from '../supabase-client';
import { normalizePropsetDoc, type PropsetDoc } from '../propsets/types';

export interface PropsetRepo {
    load(): Promise<PropsetDoc>;
    save(doc: PropsetDoc): Promise<void>;
}

class LocalPropsetRepo implements PropsetRepo {
    async load() {
        try {
            const data = localStorage.getItem(PROPSET_STORAGE_KEY);
            return normalizePropsetDoc(data ? JSON.parse(data) : null);
        } catch {
            return normalizePropsetDoc(null);
        }
    }

    async save(doc: PropsetDoc) {
        localStorage.setItem(PROPSET_STORAGE_KEY, JSON.stringify(doc));
    }
}

class SupabasePropsetRepo implements PropsetRepo {
    async load() {
        const user = getCurrentUser();
        if (!user) return normalizePropsetDoc(null);
        const { data, error } = await supabase
            .from('property_sets')
            .select('data')
            .eq('user_id', user.id)
            .maybeSingle();
        if (error) {
            console.error('SupabasePropsetRepo.load failed:', error);
            return normalizePropsetDoc(null);
        }
        return normalizePropsetDoc(data?.data);
    }

    async save(doc: PropsetDoc) {
        const user = getCurrentUser();
        if (!user) throw new Error('not signed in');
        const { error } = await supabase
            .from('property_sets')
            .upsert(
                { user_id: user.id, data: doc, updated_at: new Date().toISOString() },
                { onConflict: 'user_id' }
            );
        if (error) throw error;
    }
}

export function getPropsetRepo(): PropsetRepo {
    return getCurrentUser() ? new SupabasePropsetRepo() : new LocalPropsetRepo();
}
