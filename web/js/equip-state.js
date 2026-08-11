// 装備セットエディタの共有状態。
//
// スロット UI・セット管理・ステータス表示・共有閲覧モードの複数モジュールが
// 読み書きするため、明示的な状態オブジェクトとして 1 箇所に置く。
// ESM の export let は他モジュールから代入できない (live binding は読み取り
// 専用) ので、書き込みを共有する状態はオブジェクトのプロパティにする。
export const equipState = {
    /** 選択中キャラクター名 */
    currentEquipChar: '',
    /** 選択中ジョブキー */
    currentEquipJob: '',
    /** 選択中サポートジョブキー */
    currentEquipSupportJob: '',
    /** { slotKey: { item_id, name_en, name_ja } | null } */
    currentEquipSlots: {},
    /** 編集中セット名 (新規なら null) */
    editingEquipSetName: null,
    /** "+" タブから新規作成中なら true */
    isNewEquipSet: false,
    /** 共有閲覧モード時のキャラクター snapshot (loadCharacters() に存在しないため) */
    sharedCharacterOverride: null,
};
