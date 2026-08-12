// Storage facade. 認証状態に応じて Local / Supabase repo に処理を委譲する。
// (旧 web/js/storage.js の TS 化)
//
// 返り値の要素型は any のまま (repositories 参照)。呼び出し側が
// CharacterRecord / EquipSet として扱う。

import { getCharacterRepo } from './repositories/character-repo';
import { getEquipSetRepo } from './repositories/equipset-repo';
import { getPropsetRepo } from './repositories/propset-repo';
import type { PropsetDoc } from './propsets/types';

export async function loadCharacters(): Promise<any[]> {
    return getCharacterRepo().list();
}

export async function saveCharacters(characters: { name: string }[]): Promise<void> {
    return getCharacterRepo().save(characters);
}

export async function loadEquipSets(): Promise<any[]> {
    return getEquipSetRepo().list();
}

export async function saveEquipSets(sets: { name: string }[]): Promise<void> {
    return getEquipSetRepo().save(sets);
}

export async function loadPropsetDoc(): Promise<PropsetDoc> {
    return getPropsetRepo().load();
}

export async function savePropsetDoc(doc: PropsetDoc): Promise<void> {
    return getPropsetRepo().save(doc);
}
