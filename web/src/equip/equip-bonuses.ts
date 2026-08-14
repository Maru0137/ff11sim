// 装備セットのステータス・スキルボーナス集計の入口。
//
// 実体は Rust 側 (rust/src/equip.rs、docs/adr/0018)。アイテム説明文・オグメント・
// カスタム説明の 3 ソースの組み立ても、スロット別の振り分け (武器スロット装備の
// 武器スキルはそのスロット専用、それ以外は全スロット共通) も Rust が持つ。
// この層は「装備セットを渡して結果を受け取る」だけ。
//
// 既知の互換挙動については docs/tech-debt/equip-stats-js-quirks.md を参照
// (挙動を変えないこと)。
import { equip_set_bonuses, isItemsLoaded } from '../wasm';
import type { EquipSlotData } from './equip-store';

interface EquipSetLike {
    slots?: Record<string, EquipSlotData | null | undefined>;
}

export function calculateEquipSetBonuses(equipSet: EquipSetLike | null | undefined) {
    if (!equipSet || !equipSet.slots || !isItemsLoaded()) {
        // 空セットとして扱う。全項目 0 の結果が同じ形で返るので、
        // 参照側が未ロード時だけ形の違いを気にせずに済む
        return equip_set_bonuses({ slots: {} });
    }
    // undefined を null に寄せる。Rust 側は Option<Equip> で受けるため、
    // 欠損スロットの表現を 1 つに揃えておく。
    const slots = Object.fromEntries(
        Object.entries(equipSet.slots).map(([key, data]) => [key, data ?? null])
    );
    return equip_set_bonuses({ slots });
}
