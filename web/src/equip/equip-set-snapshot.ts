// 装備セット編集の「保存済みからの変更有無」を判定するスナップショット
// (docs/adr/0020)。
//
// スロットはキー順が読み込み経路で変わる (保存レコードの展開順 / 空スロット
// 生成順) ため、キーの並びに依存しないよう slotKeys の順で正規化する。
// slotKeys を引数に取るのは、constants (top-level await の fetch) に依存させず
// この関数を Vitest から直接検証できるようにするため。
import type { EquipSlotData } from './equip-store';

/**
 * スロット 1 枠のうち「何を装備しているか」を決めるフィールドだけを固定順で返す。
 * name_ja / name_en / description_ja は item_id に従属する表示用データなので、
 * 比較対象に入れない (同じ装備の再選択が変更として検出されるのを避ける)。
 */
function slotFingerprint(data: EquipSlotData | null | undefined): unknown {
    if (!data) return null;
    return [
        data.item_id,
        data.skill ?? null,
        data.aug_path ?? null,
        data.aug_rank ?? null,
        data.custom_description ?? '',
    ];
}

export interface EquipSetSnapshotInput {
    name: string;
    slots: Record<string, EquipSlotData | null | undefined>;
    /** プロパティセット選択 (docs/adr/0015)。保存レコードに載るので比較対象 */
    propsetSelection: string | null;
}

export function equipSetSnapshot(
    input: EquipSetSnapshotInput,
    slotKeys: readonly string[]
): string {
    return JSON.stringify([
        input.name,
        input.propsetSelection ?? null,
        slotKeys.map((key) => slotFingerprint(input.slots[key])),
    ]);
}
