// 装備セットエディタの共有状態。
//
// スロット UI・セット管理・ステータス表示・共有閲覧モードの複数モジュールが
// 読み書きするため、明示的な状態オブジェクトとして 1 箇所に置く。
// ESM の export let は他モジュールから代入できない (live binding は読み取り
// 専用) ので、書き込みを共有する状態はオブジェクトのプロパティにする。
// 型注釈は TS 層 (web/src/) からの参照用。初期値 null のままだと
// tsc に null 型と推論され、文字列等を代入できなくなる。
export const equipState = {
    /** 選択中キャラクター名 */
    currentEquipChar: '',
    /** 選択中ジョブキー */
    currentEquipJob: '',
    /** 選択中サポートジョブキー */
    currentEquipSupportJob: '',
    /** @type {Record<string, any>} slotKey → { item_id, name_en, name_ja } | null */
    currentEquipSlots: {},
    /** @type {string | null} 編集中セット名 (新規なら null) */
    editingEquipSetName: null,
    /** "+" タブから新規作成中なら true */
    isNewEquipSet: false,
    /** @type {any} 共有閲覧モード時のキャラクター snapshot (loadCharacters() に存在しないため)。null なら通常モード */
    sharedCharacterOverride: null,
};
