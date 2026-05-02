// Storage facade. 認証状態に応じて Local / Supabase repo に処理を委譲する。
// 既存呼び出し側との互換のため関数名は維持するが、async API になっている点に注意
// (呼び出し箇所は await 必須)。
//
// 過去: localStorage を直接読み書きする同期関数。
// 現在: getCharacterRepo() / getEquipSetRepo() を経由 (constants 由来のキー名は repo 側で参照)。

import { getCharacterRepo } from './repositories/character-repo.js';
import { getEquipSetRepo } from './repositories/equipset-repo.js';

export async function loadCharacters() {
    return getCharacterRepo().list();
}

export async function saveCharacters(characters) {
    return getCharacterRepo().save(characters);
}

export async function loadEquipSets() {
    return getEquipSetRepo().list();
}

export async function saveEquipSets(sets) {
    return getEquipSetRepo().save(sets);
}
