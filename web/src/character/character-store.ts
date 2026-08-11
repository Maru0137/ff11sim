// キャラクター一覧のストア。
// 一覧の再読込は main.js (認証変化・同期完了) とフォーム保存の両方から
// 呼ばれるため、imperative な関数として公開する。
import { loadCharacters } from '../storage';
import { reloadCharacters } from '../equip/equip-sets-store';
import { createStore } from '../store-utils';

export interface CharacterRecord {
    name: string;
    race: string;
    job_levels: Record<string, { level: number; master_lv: number }>;
    merit_points?: Record<string, unknown>;
    job_points?: { categories?: Record<string, { ranks?: number[] }> };
    skills?: { values?: Record<string, number> };
}

export const charactersStore = createStore<CharacterRecord[]>([]);

/** 旧 renderCharList 相当: 一覧の再読込 + 装備セット側キャラセレクタの更新 */
export async function reloadCharacterList() {
    charactersStore.set(await loadCharacters());
    await reloadCharacters();
}
