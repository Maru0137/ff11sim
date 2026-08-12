// React / TS 層から使う WASM ファサードの型付きラッパー (docs/adr/0012)。
// web/js/wasm.js (生成物 ../pkg を import する唯一のモジュール) を包み、
// 型はこのファイルで与える。web/pkg の .d.ts は全引数・返値が any のため
// 安全性に寄与しない。ここの型は Rust 側の serde 属性と手で突き合わせている:
//
// - SearchOptions: rename_all = "camelCase" (rust/src/item_search.rs)
// - SearchResult / Item: rename なしの snake_case。Option::None は
//   serde-wasm-bindgen により undefined になる
//
// 不変条件: web/src/ から ../pkg/ を直接 import しない。CI では typecheck が
// wasm-pack のビルドより先に走り、その時点で web/pkg が存在しないため。
// (web/js/wasm.js は checkJs: false で型検査対象外なので解決失敗にならない。
//  同ファイルはこの制約のために意図的に JS のまま残している)
import { search_items, item_count, initWasmRuntime } from '../js/wasm.js';

// 検索以外の WASM 関数の再エクスポート。web/src/ からの WASM 参照は
// すべてこのファイルを経由する。シグネチャは pkg の .d.ts 由来で実質 any —
// 型付けは Rust 側での型自動生成 (tsify 等) 導入時にまとめて行う
// (docs/roadmap/react-migration.md の残課題)。
export {
    calculate_status_from_profile,
    calculate_default_skills,
    get_item_by_id,
    extract_all_stats,
    extract_skill_bonuses,
    extract_named_stat,
    sum_stats,
    empty_stats,
    isWasmReady,
    isItemsLoaded,
    initWasmRuntime,
} from '../js/wasm.js';

export interface SearchFilter {
    property: string;
    operator: string;
    value: string;
}

// 全フィールドが serde(default) なので部分指定できる (装備スロット検索は
// query/slot/job/limit のみ渡す)
export interface SearchOptions {
    query?: string;
    slot?: string;
    job?: string;
    filters?: SearchFilter[];
    sortBy?: string;
    descStat?: string;
    sortOrder?: 'asc' | 'desc';
    ilv119Only?: boolean;
    ilv119Slots?: string[];
    limit?: number;
    offset?: number;
}

export interface SearchResultItem {
    id: number;
    en: string;
    ja: string;
    level: number;
    item_level?: number;
    jobs: string[];
    slots: string[];
    /** 武器スキル ID (武器のみ) */
    skill?: number;
    /** 改行はリテラルの `\n` (バックスラッシュ + n) として入っている */
    description_ja?: string;
    description_en?: string;
}

export interface SearchResults {
    items: SearchResultItem[];
    total: number;
    offset: number;
    limit: number;
    has_more: boolean;
}

export function searchItems(options: SearchOptions): SearchResults {
    return search_items(options) as SearchResults;
}

/** WASM を初期化し、埋め込み装備データの件数を返す。 */
export async function loadItems(): Promise<number> {
    await initWasmRuntime();
    return item_count() as number;
}
