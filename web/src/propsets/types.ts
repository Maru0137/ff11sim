// プロパティセット (docs/adr/0015) のデータ形状。
// 装備セットと違い id (UUID) で識別する: 選択記憶 (装備セットレコードの
// propset_selection) がセットを参照するため、リネームで参照が壊れないようにする。

export interface PropertySet {
    /** crypto.randomUUID() */
    id: string;
    name: string;
    /** 表示順のプロパティ項目 id。カタログ id ('store_tp') または 'user:<term>' */
    items: string[];
}

/** ユーザー定義プロパティ項目。term (抽出する日本語プロパティ名) 自体が識別子。 */
export interface UserPropertyItem {
    /** 'user:' + term */
    id: string;
    term: string;
}

export const USER_ITEM_PREFIX = 'user:';

export function userItemId(term: string): string {
    return USER_ITEM_PREFIX + term;
}

/** localStorage / Supabase jsonb に保存する単位 (1 ユーザー 1 ドキュメント) */
export interface PropsetDoc {
    sets: PropertySet[];
    userItems: UserPropertyItem[];
}

/** 指定 id の項目を全セットから取り除く (ユーザー定義項目の削除時に使う) */
export function stripItemFromSets(sets: PropertySet[], itemId: string): PropertySet[] {
    return sets.map((s) =>
        s.items.includes(itemId) ? { ...s, items: s.items.filter((i) => i !== itemId) } : s
    );
}

/** 欠損フィールドを補完して PropsetDoc に正規化する (repo / sync 共用) */
export function normalizePropsetDoc(raw: unknown): PropsetDoc {
    const obj = (raw && typeof raw === 'object' ? raw : {}) as Partial<PropsetDoc>;
    const sets = Array.isArray(obj.sets)
        ? obj.sets.filter(
              (s): s is PropertySet =>
                  !!s && typeof s.id === 'string' && typeof s.name === 'string'
          ).map((s) => ({ ...s, items: Array.isArray(s.items) ? s.items : [] }))
        : [];
    const userItems = Array.isArray(obj.userItems)
        ? obj.userItems.filter(
              (u): u is UserPropertyItem =>
                  !!u && typeof u.id === 'string' && typeof u.term === 'string'
          )
        : [];
    return { sets, userItems };
}
