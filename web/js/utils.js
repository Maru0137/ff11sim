// 純粋関数ユーティリティ。状態 / DOM / WASM への依存なし。

import {
    JP_CATEGORY_COUNT,
    JP_MAX_RANK,
    JOB_MERIT_GROUP_SIZE,
    JOB_MERIT_CATEGORIES,
    JOB_MERIT_PLACEHOLDER_RE,
    AUGMENT_JA_TO_EN,
} from './constants.js';

// === ジョブポイント (JP) ===
// 三角数: ランク r まで振るために必要な JP は r*(r+1)/2
export function jpCategoryCost(rank) {
    return rank * (rank + 1) / 2;
}

export function jpJobTotal(ranks) {
    return ranks.reduce((s, r) => s + jpCategoryCost(r), 0);
}

// デフォルトの全振り（ランク 20 × 10 カテゴリ）
export function jpDefaultRanks() {
    return new Array(JP_CATEGORY_COUNT).fill(JP_MAX_RANK);
}

// === ジョブ別メリットポイント ===
export function jobMeritDefaultRanks() {
    return new Array(JOB_MERIT_GROUP_SIZE).fill(0);
}

export function jobMeritCategoryName(jobKey, group, idx) {
    const job = JOB_MERIT_CATEGORIES[jobKey];
    const name = job && job[group] && job[group][idx];
    if (name) return name;
    return `カテゴリ ${idx + 1}`;
}

export function isJobMeritPlaceholder(jobKey, group, idx) {
    const name = jobMeritCategoryName(jobKey, group, idx);
    return JOB_MERIT_PLACEHOLDER_RE.test(name);
}

// SAM の Group 1 で「ストアTP」相当の項目のインデックス（カテゴリ名から動的解決）
export function samStoreTpIndex() {
    const g1 = JOB_MERIT_CATEGORIES.Sam.group1;
    return g1.findIndex(name => name && name.startsWith('ストアTP'));
}

// === 数値フォーマッタ ===
export function formatBonus(value) {
    if (!value || value === 0) return '-';
    return value > 0 ? `+${value}` : `${value}`;
}

export function formatPctBonus(value) {
    if (!value || value === 0) return '-';
    return value > 0 ? `+${value}%` : `${value}%`;
}

// === オーグメント JA→EN 変換 ===
export function convertAugmentJaToEn(text) {
    let result = text;
    for (const [ja, en] of AUGMENT_JA_TO_EN) {
        result = result.replaceAll(ja, en);
    }
    return result;
}
