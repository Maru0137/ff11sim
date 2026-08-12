// WASM ファサード。生成物 ../pkg/ff11sim.js (gitignore 対象) を import する
// 唯一のモジュール。WASM エクスポートの改名・削除で壊れる import をここに
// 集約し、参照漏れ事故 (WEAPON_SKILL_KEYS 事故と同種) の影響範囲を限定する。
//
// readiness フラグもここが所有する。getter 関数で公開するのは、他モジュール
// からは読み取り専用であることを明示するため。
import init, {
    calculate_status_from_profile, calculate_default_skills,
    extract_all_stats, extract_skill_bonuses, extract_named_stat,
    search_items, get_item_by_id, item_count,
    sum_stats, empty_stats,
} from '../pkg/ff11sim.js';

export {
    calculate_status_from_profile, calculate_default_skills,
    extract_all_stats, extract_skill_bonuses, extract_named_stat,
    search_items, get_item_by_id, item_count,
    sum_stats, empty_stats,
};

let wasmReady = false;
let itemsLoaded = false;

export function isWasmReady() {
    return wasmReady;
}

export function isItemsLoaded() {
    return itemsLoaded;
}

// wasmBytes: 通常 (ブラウザ) は未指定で、glue が new URL(..., import.meta.url) から
// fetch する。node (Vitest) では file URL を fetch できないため、テスト側が
// .wasm のバイト列を読んで渡す (web/src/equip/equip-bonuses.test.ts)。
export async function initWasmRuntime(wasmBytes) {
    await init(wasmBytes ? { module_or_path: wasmBytes } : undefined);
    wasmReady = true;
    // 装備データは WASM の初期化と同時に使えるようになる。
    // 以前は items.json の fetch 完了で立てていたフラグ (docs/adr/0009)。
    itemsLoaded = item_count() > 0;
}
