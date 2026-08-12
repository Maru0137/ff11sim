// ステータス表示値の整形ヘルパー (純粋)。
// 元は compute.ts のプライベート関数だったが、プロパティセットのカタログ
// (web/src/propsets/catalog.ts) からも同じ表示規則を使うため共有化した。
import { ALL_SKILL_KEYS } from '../constants';

export const numOrDash = (v: number | null | undefined) => (v != null && v !== 0 ? v : '-');
export const pctOrDash = (v: number | null | undefined) => (v ? `${v}%` : '-');
export const formatStatBonus = (val: number) => (val > 0 ? `+${val}` : val < 0 ? `${val}` : '-');
export const fmtPct = (v: number | null | undefined) => (v != null && v !== 0 ? `${v}%` : '-');

const SKILL_JA_MAP: Record<string, string> = Object.fromEntries(ALL_SKILL_KEYS);

export function formatWeaponSkill(
    kind: string | null | undefined,
    value: number | null | undefined
) {
    if (!kind || !value) return '-';
    return `${SKILL_JA_MAP[kind] || kind} (${value})`;
}
