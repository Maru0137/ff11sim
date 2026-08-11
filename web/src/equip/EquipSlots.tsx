// 装備スロット行 UI (旧 web/js/equip-slots.js の React 移行)。
// 16 部位それぞれの検索入力・候補ドロップダウン・選択/クリア・
// オーグメント選択・カスタム説明を持つ。
//
// スロットの選択内容は equipState.currentEquipSlots (equip-store 経由) が
// 持ち、検索テキストとドロップダウンだけが行のローカル状態。
// 装備セットの読み込み時は世代 key で行が作り直され、ローカル状態が
// 選択名にリセットされる (旧実装の input.value 代入に相当)。
import { useRef, useState, useSyncExternalStore } from 'react';
import { searchItems } from '../wasm';
import type { SearchResultItem } from '../wasm';
import { isItemsLoaded } from '../../js/wasm.js';
import { getItemAugments, getAugmentText } from '../augments';
import { updateEquipEditStatus } from '../status/status-store';
import { openCustomAugHelp } from '../modals/modal-store';
import {
    SLOT_DEFS,
    equipState,
    getSlots,
    subscribeEquipState,
    getEquipStateVersion,
    subscribeSlotsGeneration,
    getSlotsGeneration,
} from './equip-store';
import type { EquipSlotData, SlotDef } from './equip-store';

interface AugPathInfo {
    type: string;
    ranks?: { rank: number; text: string }[];
}

const HEADER_LABELS = ['スロット', '装備名', '', '説明', 'オーグメント', 'Aug説明', 'カスタム'];

export function EquipSlots() {
    useSyncExternalStore(subscribeEquipState, getEquipStateVersion);
    const generation = useSyncExternalStore(subscribeSlotsGeneration, getSlotsGeneration);

    return (
        <>
            <div className="equip-slot-header">
                {HEADER_LABELS.map((text, i) => (
                    <span key={i}>
                        {text}
                        {text === 'カスタム' && (
                            <button
                                type="button"
                                className="aug-help-btn"
                                title="カスタムオーグメント欄の書き方を表示"
                                onClick={openCustomAugHelp}
                            >
                                ?
                            </button>
                        )}
                    </span>
                ))}
            </div>
            {SLOT_DEFS.map((slot) => (
                <SlotRow key={`${generation}:${slot.key}`} slot={slot} />
            ))}
        </>
    );
}


function SlotRow({ slot }: { slot: SlotDef }) {
    const data = (getSlots()[slot.key] ?? null) as EquipSlotData | null;
    const selectedName = data ? data.name_ja || data.name_en || '' : '';

    const [text, setText] = useState(selectedName);
    // ドロップダウン候補。null = 閉じている
    const [items, setItems] = useState<SearchResultItem[] | null>(null);
    const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

    function doSearch(query: string) {
        if (!isItemsLoaded()) {
            setItems(null);
            return;
        }
        const results = searchItems({
            query: query.trim(),
            slot: slot.key,
            job: equipState.currentEquipJob ? equipState.currentEquipJob.toUpperCase() : '',
            sortBy: 'id',
            sortOrder: 'desc',
            limit: 30,
        });
        setItems(results.items.length > 0 ? results.items : null);
    }

    function selectItem(item: SearchResultItem) {
        getSlots()[slot.key] = {
            item_id: item.id,
            name_en: item.en,
            name_ja: item.ja,
            description_ja: item.description_ja || '',
            skill: item.skill != null ? item.skill : null,
            aug_path: null,
            aug_rank: null,
        };
        setText(item.ja || item.en);
        setItems(null);
        updateEquipEditStatus();
    }

    function clearSlot() {
        getSlots()[slot.key] = null;
        setText('');
        updateEquipEditStatus();
    }

    const augInfo = data ? getItemAugments(data.item_id) : null;
    const augPaths: AugPathInfo[] = augInfo?.paths || [];
    const hasAug = augPaths.length > 0;
    const augValue =
        data && data.aug_path != null && data.aug_rank != null
            ? `${data.aug_path}-${data.aug_rank}`
            : '';
    const augText = getAugmentText(data) || '';

    // 説明は保存データの description_ja を表示する。旧実装は選択直後のみ
    // description_en へフォールバックしていたが、再読込後は description_ja
    // だけだったため、永続時の挙動に統一した。
    const descriptionHtml = (data?.description_ja || '').replace(/\\n/g, '<br>');

    return (
        <div className="equip-slot-row">
            <span className="equip-slot-label">{slot.label}</span>
            <div className="equip-slot-input-wrapper">
                <input
                    type="text"
                    className="equip-slot-search"
                    data-slot={slot.key}
                    placeholder="検索..."
                    value={text}
                    onChange={(e) => {
                        const q = e.target.value;
                        setText(q);
                        if (debounceTimer.current) clearTimeout(debounceTimer.current);
                        debounceTimer.current = setTimeout(() => doSearch(q), 150);
                    }}
                    onFocus={() => doSearch(text)}
                    onBlur={() => {
                        setTimeout(() => setItems(null), 200);
                    }}
                />
                <span className="equip-slot-arrow">▼</span>
                <div
                    className={items === null ? 'equip-slot-dropdown hidden' : 'equip-slot-dropdown'}
                    data-slot={slot.key}
                >
                    {(items ?? []).map((item) => (
                        <div
                            key={item.id}
                            className="equip-slot-dropdown-item"
                            onMouseDown={(e) => {
                                e.preventDefault();
                                selectItem(item);
                            }}
                        >
                            {item.ja || item.en}
                        </div>
                    ))}
                </div>
            </div>
            <span className="equip-slot-selected" data-slot={slot.key}></span>
            <button className="equip-slot-clear" data-slot={slot.key} onClick={clearSlot}>
                ×
            </button>
            {/* 説明文は WASM 埋め込み item DB 由来 (旧実装も innerHTML)。\n → <br> */}
            <div
                className="equip-slot-description"
                data-slot={slot.key}
                dangerouslySetInnerHTML={{ __html: descriptionHtml }}
            />
            <div className="equip-slot-aug-container" data-slot={slot.key}>
                <select
                    className="equip-slot-aug-path"
                    data-slot={slot.key}
                    disabled={!hasAug}
                    value={augValue}
                    onChange={(e) => {
                        if (!data) return;
                        const val = e.target.value;
                        if (val) {
                            const [pathIdx, rank] = val.split('-').map(Number);
                            data.aug_path = pathIdx;
                            data.aug_rank = rank;
                        } else {
                            data.aug_path = null;
                            data.aug_rank = null;
                        }
                        updateEquipEditStatus();
                    }}
                >
                    <option value="">-- オーグメント --</option>
                    {hasAug &&
                        augPaths.flatMap((path, idx) =>
                            (path.ranks || []).map((r) => (
                                <option key={`${idx}-${r.rank}`} value={`${idx}-${r.rank}`}>
                                    {`${path.type} Rank ${r.rank}`}
                                </option>
                            ))
                        )}
                </select>
                <div className="equip-slot-aug-text" data-slot={slot.key}>
                    {augText}
                </div>
            </div>
            <input
                type="text"
                className="equip-slot-custom-desc"
                data-slot={slot.key}
                placeholder="例: STR+5 命中+10 ヘイスト+3%"
                value={data?.custom_description || ''}
                onChange={(e) => {
                    // 旧実装同様、装備未選択のスロットへの入力は保存しない
                    if (!data) return;
                    data.custom_description = e.target.value;
                    updateEquipEditStatus();
                }}
            />
        </div>
    );
}
