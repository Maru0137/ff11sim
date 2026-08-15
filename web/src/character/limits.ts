// キャラクター編集フォームの数値入力に掛けるクランプ (docs/adr/0021)。
//
// 上限は 2 種類ある:
//   - 項目ごとの上限だけのもの (メリット基礎 0-15 / スキル 0-8 / JP 0-20 / スキル値 0-上限)
//   - 項目ごとの上限に加えてグループ合計にも上限があるもの
//     (ジョブ別メリット 各 0-5 かつグループ計 10 / メリットその他 各 0-5 かつ計 10)
// 後者は「他項目の合計を引いた残り」で頭打ちにする。

/** 0 以上 max 以下に丸める */
export function clampToMax(value: number, max: number): number {
    if (!(value > 0)) return 0;
    return value > max ? max : value;
}

/**
 * グループ合計に上限がある項目のクランプ。
 * ranks[idx] を value にしたときグループ計が groupMax を超えないよう、
 * 他項目の合計を差し引いた残りまでに抑える。
 */
export function clampWithinGroup(
    ranks: readonly number[],
    idx: number,
    value: number,
    perItemMax: number,
    groupMax: number
): number {
    const v = clampToMax(value, perItemMax);
    const otherSum = ranks.reduce((s, r, i) => (i === idx ? s : s + (r || 0)), 0);
    return Math.min(v, Math.max(0, groupMax - otherSum));
}
