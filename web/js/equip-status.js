// 装備編集ステータス表示のアダプタ。
// 表示本体は js/status-display.js (状態を持たないため呼び出し時に依存を全て注入する)。
// 注入する値の組み立てをここに集約し、呼び出し側は引数なしで呼べるようにする。
import { updateEquipEditStatus as _updateEquipEditStatus } from './status-display.js';
import {
    calculate_status_from_profile, calculate_default_skills,
    isWasmReady, isItemsLoaded,
} from './wasm.js';
import { equipState } from './equip-state.js';
import { calculateEquipSetBonuses } from './equip-bonuses.js';

export function updateEquipEditStatus() {
    return _updateEquipEditStatus({
        wasmReady: isWasmReady(),
        itemsLoaded: isItemsLoaded(),
        charName: equipState.currentEquipChar,
        jobKey: equipState.currentEquipJob,
        supportJob: equipState.currentEquipSupportJob || null,
        currentEquipSlots: equipState.currentEquipSlots,
        calculate_status_from_profile,
        calculate_default_skills,
        calculateEquipSetBonuses,
        // 共有閲覧モードでは元キャラの snapshot を渡す
        characterOverride: equipState.sharedCharacterOverride,
    });
}
