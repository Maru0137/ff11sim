// 一覧のドラッグ並び替え (docs/adr/0022)。
// 「掴んだ要素を、落とした先の要素の位置へ挿入する」= EquipSetControls /
// PropsetManageModal と同じ規則。配列の順序がそのまま表示順・保存順になる。

/**
 * fromIdx の要素を取り出し、toIdx の位置へ挿入した新しい配列を返す。
 * 添字が範囲外か同じ位置なら元の配列をそのまま返す (呼び出し側で保存を省ける)。
 */
export function moveItem<T>(items: readonly T[], fromIdx: number, toIdx: number): readonly T[] {
    if (
        fromIdx === toIdx ||
        fromIdx < 0 || fromIdx >= items.length ||
        toIdx < 0 || toIdx >= items.length
    ) {
        return items;
    }
    const next = [...items];
    const [moved] = next.splice(fromIdx, 1);
    next.splice(toIdx, 0, moved);
    return next;
}
