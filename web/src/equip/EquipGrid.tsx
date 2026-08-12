// 装備セットの 4x4 コンパクトグリッド (FF11 の装備画面準拠)。
//
// セルは表示専用で、クリックすると装備選択モーダル (EquipSelectModal) が
// 開き、検索・オーグメント・カスタム説明の編集はすべてモーダル側で行う。
// 選択内容は equipState.currentEquipSlots (equip-store) が持ち、ここでは
// バージョン番号の購読だけで再描画する。
import { useEffect, useState, useSyncExternalStore } from 'react';
// スタイルは .equip-grid-panel / .equip-select-modal スコープに閉じている
import '../../styles/equip-grid.css';
import { getAugmentText } from '../augments';
import {
    SLOT_DEFS,
    getSlots,
    subscribeEquipState,
    getEquipStateVersion,
    subscribeSlotsGeneration,
    getSlotsGeneration,
} from './equip-store';
import type { EquipSlotData, SlotDef } from './equip-store';
import { equipSummary } from './equip-summary';
import { EquipSelectModal } from './EquipSelectModal';

// FF11 の装備画面と同じ並び (データ定義順とは異なる)。
// 行1: 武器・遠隔 / 行2: 頭・首・耳 / 行3: 胴・手・指 / 行4: 背・腰・脚・足
const GRID_ORDER = [
    'main', 'sub', 'range', 'ammo',
    'head', 'neck', 'ear1', 'ear2',
    'body', 'hands', 'ring1', 'ring2',
    'back', 'waist', 'legs', 'feet',
];

export function EquipGrid() {
    useSyncExternalStore(subscribeEquipState, getEquipStateVersion);
    const generation = useSyncExternalStore(subscribeSlotsGeneration, getSlotsGeneration);
    const [openSlot, setOpenSlot] = useState<string | null>(null);

    // セット切替・新規フォームでスロットが丸ごと入れ替わったら、
    // 開いていたモーダルは古いスロットを指しているので閉じる
    useEffect(() => {
        setOpenSlot(null);
    }, [generation]);

    const slotByKey = new Map(SLOT_DEFS.map((s) => [s.key, s]));
    const gridSlots = GRID_ORDER.map((key) => slotByKey.get(key)).filter(
        (s): s is SlotDef => s !== undefined
    );
    const openSlotDef = openSlot !== null ? slotByKey.get(openSlot) : undefined;

    return (
        <section className="equip-grid-panel">
            <h3>装備</h3>
            <div className="equip-grid">
                {gridSlots.map((slot) => (
                    <EquipCell key={slot.key} slot={slot} onClick={() => setOpenSlot(slot.key)} />
                ))}
            </div>
            {openSlotDef && (
                // key: 開くたびに新規マウントして検索状態をリセットする
                <EquipSelectModal
                    key={openSlotDef.key}
                    slot={openSlotDef}
                    onClose={() => setOpenSlot(null)}
                />
            )}
        </section>
    );
}

function EquipCell({ slot, onClick }: { slot: SlotDef; onClick: () => void }) {
    const data = (getSlots()[slot.key] ?? null) as EquipSlotData | null;

    if (!data) {
        return (
            <button type="button" className="equip-cell empty" data-slot={slot.key} onClick={onClick}>
                <span className="equip-cell-top">
                    <span className="equip-cell-slot">{slot.label}</span>
                </span>
                <span className="equip-cell-name">未設定</span>
                <span className="equip-cell-summary">クリックして選択</span>
            </button>
        );
    }

    const summary = equipSummary({
        description: data.description_ja,
        augText: getAugmentText(data),
        custom: data.custom_description,
    });
    const hasAug = data.aug_path != null && data.aug_rank != null;
    const hasMemo = !!data.custom_description;

    return (
        <button
            type="button"
            className="equip-cell"
            data-slot={slot.key}
            onClick={onClick}
            title={summary}
        >
            <span className="equip-cell-top">
                <span className="equip-cell-slot">{slot.label}</span>
                <span className="equip-cell-badges">
                    {hasAug && <span className="equip-cell-badge">Aug</span>}
                    {hasMemo && <span className="equip-cell-badge memo">メモ</span>}
                </span>
            </span>
            <span className="equip-cell-name">{data.name_ja || data.name_en}</span>
            <span className="equip-cell-summary">{summary}</span>
        </button>
    );
}
