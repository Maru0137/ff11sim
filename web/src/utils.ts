// 純粋関数ユーティリティ。状態 / DOM / WASM への依存なし。
// (旧 web/js/utils.js の TS 化)

import {
    JP_CATEGORY_COUNT,
    JP_MAX_RANK,
    JOB_MERIT_GROUP_SIZE,
    JOB_MERIT_CATEGORIES,
    JOB_MERIT_PLACEHOLDER_RE,
} from './constants';

// === ジョブポイント (JP) ===
// 三角数: ランク r まで振るために必要な JP は r*(r+1)/2
export function jpCategoryCost(rank: number): number {
    return rank * (rank + 1) / 2;
}

export function jpJobTotal(ranks: number[]): number {
    return ranks.reduce((s, r) => s + jpCategoryCost(r), 0);
}

// デフォルトの全振り（ランク 20 × 10 カテゴリ）
export function jpDefaultRanks(): number[] {
    return new Array(JP_CATEGORY_COUNT).fill(JP_MAX_RANK);
}

// === ジョブ別メリットポイント ===
export function jobMeritDefaultRanks(): number[] {
    return new Array(JOB_MERIT_GROUP_SIZE).fill(0);
}

export function jobMeritCategoryName(jobKey: string, group: string, idx: number): string {
    const job = JOB_MERIT_CATEGORIES[jobKey];
    const name = job && job[group] && job[group][idx];
    if (name) return name;
    return `カテゴリ ${idx + 1}`;
}

export function isJobMeritPlaceholder(jobKey: string, group: string, idx: number): boolean {
    const name = jobMeritCategoryName(jobKey, group, idx);
    return JOB_MERIT_PLACEHOLDER_RE.test(name);
}

// SAM の Group 1 で「ストアTP」相当の項目のインデックス（カテゴリ名から動的解決）
export function samStoreTpIndex(): number {
    const g1: (string | null)[] = JOB_MERIT_CATEGORIES.Sam.group1;
    return g1.findIndex((name) => name && name.startsWith('ストアTP'));
}

// === 数値フォーマッタ ===
export function formatBonus(value: number | null | undefined): string {
    if (!value || value === 0) return '-';
    return value > 0 ? `+${value}` : `${value}`;
}

export function formatPctBonus(value: number | null | undefined): string {
    if (!value || value === 0) return '-';
    return value > 0 ? `+${value}%` : `${value}%`;
}

