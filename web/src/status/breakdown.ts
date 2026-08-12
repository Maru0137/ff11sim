// 内訳モーダル (docs/adr/0016) のテーブルモデル構築。DOM に依存しない純関数。
//
// 行 = 寄与元 (装備 16 部位 → 基礎 → 種族 → メイン → サポ → メリポ → JP →
// ギフト → ML)、列 = 項目。装備行の値は per_slot_stats (equip-bonuses)、
// キャラ由来行の値は calculate_status_breakdown (WASM) の rows から引く。
// 合計行はパネル表示値 (StatusView.values / propertyValues) をそのまま使い、
// 内訳の合計とパネル表示が定義レベルで一致することを保証する。
import { SLOT_DEFS, type EquipSlotData } from '../equip/equip-store';
import { BUILTIN_ITEM_BY_ID } from '../propsets/catalog';
import { USER_ITEM_PREFIX } from '../propsets/types';
import { get_item_by_id } from '../wasm';

export interface BreakdownColumnSpec {
    key: string;
    label: string;
    /** 装備スロット行: per_slot_stats のキー */
    equipKey?: string;
    /** キャラ由来行: calculate_status_breakdown の rows 列キー */
    charKey?: string;
    /** 合計行の参照先 (status → view.values の id / propset → propertyValues の id) */
    totalId: string;
    /** ユーザー定義項目 ('user:<term>')。装備行はスロット別ユーザー値から引く */
    userItemId?: string;
    /**
     * セル値の符号を反転して表示する (被ダメ系の「〜-」表記用。軽減 -30 → 30)。
     * 合計行はパネル表示値の参照なので反転済み (compute.ts と対で保つ)
     */
    negate?: boolean;
}

/** null = 寄与なし ('-' 表示)。合計行は表示値 (文字列含む) をそのまま持つ */
export type BreakdownCell = number | string | null;

export interface BreakdownRow {
    key: string;
    /** 行ヘッダ (部位名 / 種族 / メイン ...) */
    label: string;
    /** 装備名 or キャラ側の補足 (種族名、ジョブ表記など) */
    itemLabel: string;
    kind: 'slot' | 'char';
    cells: BreakdownCell[];
}

export interface BreakdownModel {
    columns: BreakdownColumnSpec[];
    rows: BreakdownRow[];
    totals: BreakdownCell[];
}

// ステータス内訳 (モーダル①) の固定列。LeftStatusTables の行項目と対応し、
// 合計行は同テーブルの「合計」列と同じ id を参照する。
export const STATUS_BREAKDOWN_COLUMNS: BreakdownColumnSpec[] = [
    { key: 'hp', label: 'HP', equipKey: 'hp', charKey: 'hp', totalId: 'equipTotalHp' },
    { key: 'mp', label: 'MP', equipKey: 'mp', charKey: 'mp', totalId: 'equipTotalMp' },
    { key: 'str', label: 'STR', equipKey: 'str', charKey: 'str', totalId: 'equipTotalStr' },
    { key: 'dex', label: 'DEX', equipKey: 'dex', charKey: 'dex', totalId: 'equipTotalDex' },
    { key: 'vit', label: 'VIT', equipKey: 'vit', charKey: 'vit', totalId: 'equipTotalVit' },
    { key: 'agi', label: 'AGI', equipKey: 'agi', charKey: 'agi', totalId: 'equipTotalAgi' },
    { key: 'int', label: 'INT', equipKey: 'int', charKey: 'int', totalId: 'equipTotalInt' },
    { key: 'mnd', label: 'MND', equipKey: 'mnd', charKey: 'mnd', totalId: 'equipTotalMnd' },
    { key: 'chr', label: 'CHR', equipKey: 'chr', charKey: 'chr', totalId: 'equipTotalChr' },
    { key: 'haste', label: 'ヘイスト', equipKey: 'haste_pct', totalId: 'equipTotalHaste' },
    { key: 'def', label: '防御', equipKey: 'def', charKey: 'def', totalId: 'equipTotalDef' },
    { key: 'evasion', label: '回避', equipKey: 'evasion', charKey: 'evasion', totalId: 'equipTotalEva' },
    { key: 'mdef', label: '魔防', equipKey: 'magic_def_bonus', charKey: 'mdef', totalId: 'equipTotalMdef' },
    { key: 'magic_evasion', label: '魔回避', equipKey: 'magic_evasion', charKey: 'magic_evasion', totalId: 'equipTotalMeva' },
    { key: 'dt', label: '被ダメ-', equipKey: 'damage_taken_pct', totalId: 'equipTotalDt', negate: true },
    { key: 'pdt', label: '被物理-', equipKey: 'physical_damage_taken_pct', totalId: 'equipTotalPdt', negate: true },
    { key: 'mdt', label: '被魔法-', equipKey: 'magic_damage_taken_pct', totalId: 'equipTotalMdt', negate: true },
];

/**
 * プロパティセット内訳 (モーダル②) の列を項目 id リストから組み立てる。
 * 内訳非対応の項目 (breakdown メタなし = 武器スキル等の文字列値) は列から除外する。
 */
export function buildPropsetColumns(itemIds: string[]): BreakdownColumnSpec[] {
    const cols: BreakdownColumnSpec[] = [];
    for (const id of itemIds) {
        if (id.startsWith(USER_ITEM_PREFIX)) {
            cols.push({
                key: id,
                label: id.slice(USER_ITEM_PREFIX.length),
                totalId: id,
                userItemId: id,
            });
            continue;
        }
        const item = BUILTIN_ITEM_BY_ID.get(id);
        if (!item || !item.breakdown) continue;
        cols.push({
            key: id,
            label: item.label,
            equipKey: item.breakdown.equipKey,
            charKey: item.breakdown.charKey,
            totalId: id,
        });
    }
    return cols;
}

// キャラ由来行の表示順とラベル (calculate_status_breakdown のソースキー順)
const CHAR_ROW_DEFS: [string, string][] = [
    ['base', '基礎'],
    ['race', '種族'],
    ['main_job', 'メイン'],
    ['support_job', 'サポ'],
    ['merit', 'メリポ'],
    ['job_points', 'JP'],
    ['gift', 'ギフト'],
    ['master_level', 'ML'],
];

export interface BreakdownInputs {
    columns: BreakdownColumnSpec[];
    slots: Record<string, EquipSlotData | null | undefined> | undefined;
    /** calculateEquipSetBonuses の per_slot_stats */
    perSlotStats: Record<string, Record<string, number>> | undefined;
    /** calculateUserPropertyValuesPerSlot の結果 */
    perSlotUserValues: Record<string, Record<string, number>>;
    /** calculate_status_breakdown の rows */
    charRows: Record<string, Record<string, number>>;
    /** 合計行の参照 (StatusView.values / propertyValues) */
    totals: Record<string, string | number>;
    /** ソースキー → 補足ラベル (種族名、"ナ99 (ジョブ+特性)" 等) */
    charItemLabels: Partial<Record<string, string>>;
}

export function buildBreakdownModel(inp: BreakdownInputs): BreakdownModel {
    const rows: BreakdownRow[] = [];

    for (const def of SLOT_DEFS) {
        const slotStats = inp.perSlotStats?.[def.key];
        const userValues = inp.perSlotUserValues[def.key];
        const cells = inp.columns.map((col): BreakdownCell => {
            let v = 0;
            if (col.userItemId) {
                v = userValues?.[col.userItemId] || 0;
            } else if (col.equipKey) {
                v = slotStats?.[col.equipKey] || 0;
            }
            return v === 0 ? null : col.negate ? -v : v;
        });
        rows.push({
            key: def.key,
            label: def.label,
            itemLabel: formatSlotItemLabel(inp.slots?.[def.key]),
            kind: 'slot',
            cells,
        });
    }

    for (const [src, label] of CHAR_ROW_DEFS) {
        const srcRow = inp.charRows[src];
        const cells = inp.columns.map((col): BreakdownCell => {
            const v = (col.charKey && srcRow?.[col.charKey]) || 0;
            if (v === 0) return null;
            // f32 由来の値 (種族 37.5 など) の表示桁を丸める (サポは 0.25 刻み)
            const rounded = Math.round(v * 100) / 100;
            return col.negate ? -rounded : rounded;
        });
        rows.push({
            key: src,
            label,
            itemLabel: inp.charItemLabels[src] || '',
            kind: 'char',
            cells,
        });
    }

    const totals = inp.columns.map((col): BreakdownCell => inp.totals[col.totalId] ?? '-');
    return { columns: inp.columns, rows, totals };
}

/** 装備行の表示名: 日本語名 (保存値 → アイテム DB の順) + オーグメントランク */
export function formatSlotItemLabel(slotData: EquipSlotData | null | undefined): string {
    if (!slotData) return '';
    const item = slotData.name_ja ? null : get_item_by_id(slotData.item_id);
    const name = slotData.name_ja || item?.ja || slotData.name_en || '';
    const rank = slotData.aug_rank != null ? ` R${slotData.aug_rank}` : '';
    return `${name}${rank}`;
}
