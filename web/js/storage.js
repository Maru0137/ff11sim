// localStorage アクセスの薄いラッパ。
// キー定数は constants.js に集約。

import { STORAGE_KEY, EQUIP_STORAGE_KEY } from './constants.js';

export function loadCharacters() {
    try {
        const data = localStorage.getItem(STORAGE_KEY);
        return data ? JSON.parse(data) : [];
    } catch {
        return [];
    }
}

export function saveCharacters(characters) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(characters));
}

export function loadEquipSets() {
    try {
        const data = localStorage.getItem(EQUIP_STORAGE_KEY);
        const sets = data ? JSON.parse(data) : [];
        // 後方互換性: job/character フィールドが無い古いセットにデフォルトを補完。
        return sets.map(s => ({ job: '', character: '', ...s }));
    } catch {
        return [];
    }
}

export function saveEquipSets(sets) {
    localStorage.setItem(EQUIP_STORAGE_KEY, JSON.stringify(sets));
}
