// 装備選択モーダル (EquipGrid のセルクリックで開く)。
//
// 上段: 選択中装備の詳細 (説明全文・オーグメント・カスタム説明・外す)。
// 下段: 検索。結果テーブルは検索ビューの ResultsTable を流用し、
// .search-page ラッパで search.css のスコープに乗せる。
// 行クリックで装備を差し替えるが、続けてオーグメント等を設定できるよう
// モーダルは閉じない。
//
// Mantine コンポーネントは Modal のみ (docs/adr/0013 の段階導入)。
// 内部の入力は index.css の既存手書きパターン (素の input/select) に乗せ、
// Mantine input 系と裸の select/input グローバルルールの干渉を避ける。
import { useEffect, useRef, useState, useSyncExternalStore } from 'react';
import { Modal } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import { searchItems, isItemsLoaded } from '../wasm';
import type { SearchResultItem, SearchResults } from '../wasm';
import { getItemAugments, getAugmentText, getDefaultAugmentSelection } from '../augments';
import { updateEquipEditStatus } from '../status/status-store';
import { openCustomAugHelp } from '../modals/modal-store';
import { isShareMode } from '../share-ui';
import { ResultsTable } from '../search/ResultsTable';
import { Pagination } from '../search/Pagination';
import { ILV119_SLOTS } from '../search/SearchPage';
import {
    equipState,
    getSlots,
    subscribeEquipState,
    getEquipStateVersion,
} from './equip-store';
import type { EquipSlotData, SlotDef } from './equip-store';

const PAGE_SIZE = 50;
// th クリックで並び替えられる列 (SearchPage の SELECT_SORT_KEYS と違い、
// モーダルにはソート select が無いので列に対応するキーのみ)
const SORTABLE_COLUMN_IDS = ['ja', 'level', 'item_level'];

interface SearchState {
    ilv119: boolean;
    sortBy: string;
    sortOrder: 'asc' | 'desc';
    offset: number;
}

// 新しい装備ほど上に出す (旧スロット行ドロップダウンの並びを踏襲)
const INITIAL_SEARCH: SearchState = {
    ilv119: true,
    sortBy: 'id',
    sortOrder: 'desc',
    offset: 0,
};

export function EquipSelectModal({ slot, onClose }: { slot: SlotDef; onClose: () => void }) {
    useSyncExternalStore(subscribeEquipState, getEquipStateVersion);
    // getInitialValueInEffect: false — 初回レンダーから正しい値にしないと、
    // モバイルで一瞬デスクトップ用モーダルが出てから fullScreen に切り替わる
    const isMobile = useMediaQuery('(max-width: 768px)', false, {
        getInitialValueInEffect: false,
    });
    const readOnly = isShareMode();

    const [query, setQuery] = useState('');
    const [search, setSearch] = useState(INITIAL_SEARCH);
    const [results, setResults] = useState<SearchResults | null>(null);
    const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

    const ilvVisible = ILV119_SLOTS.includes(slot.key);
    const data = (getSlots()[slot.key] ?? null) as EquipSlotData | null;

    function runSearch(q: string, next: SearchState) {
        setSearch(next);
        if (!isItemsLoaded()) {
            setResults(null);
            return;
        }
        setResults(
            searchItems({
                query: q.trim(),
                slot: slot.key,
                job: equipState.currentEquipJob ? equipState.currentEquipJob.toUpperCase() : '',
                sortBy: next.sortBy,
                sortOrder: next.sortOrder,
                ilv119Only: next.ilv119 && ilvVisible,
                ilv119Slots: ILV119_SLOTS,
                limit: PAGE_SIZE,
                offset: next.offset,
            })
        );
    }

    // 開いた直後に空クエリで初回検索 (旧スロット行の onFocus 検索を踏襲)
    useEffect(() => {
        if (!readOnly) runSearch('', INITIAL_SEARCH);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    function handleQueryChange(q: string) {
        setQuery(q);
        if (debounceTimer.current) clearTimeout(debounceTimer.current);
        debounceTimer.current = setTimeout(() => runSearch(q, { ...search, offset: 0 }), 150);
    }

    function handleHeaderClick(columnId: string) {
        if (!SORTABLE_COLUMN_IDS.includes(columnId)) return;
        const next: SearchState =
            search.sortBy === columnId
                ? { ...search, sortOrder: search.sortOrder === 'asc' ? 'desc' : 'asc', offset: 0 }
                : { ...search, sortBy: columnId, sortOrder: 'asc', offset: 0 };
        runSearch(query, next);
    }

    function selectItem(item: SearchResultItem) {
        const defaultAug = getDefaultAugmentSelection(getItemAugments(item.id));
        getSlots()[slot.key] = {
            item_id: item.id,
            name_en: item.en,
            name_ja: item.ja,
            description_ja: item.description_ja || '',
            skill: item.skill != null ? item.skill : null,
            aug_path: defaultAug ? defaultAug.path : null,
            aug_rank: defaultAug ? defaultAug.rank : null,
        };
        updateEquipEditStatus();
    }

    function clearSlot() {
        getSlots()[slot.key] = null;
        updateEquipEditStatus();
    }

    return (
        <Modal
            opened
            onClose={onClose}
            title={`${slot.label} の装備`}
            size={760}
            // AppShell (zIndex 200) より上、レガシーモーダル (.modal-backdrop,
            // z-index 1000) より下。カスタムAugヘルプが正しく重なるようにする
            zIndex={900}
            fullScreen={isMobile}
            // スクロールロック中もピンチズーム/パンを許可する。ズームした状態で
            // 塞がれると閉じるボタンに到達できなくなる (iOS Safari)
            removeScrollProps={{ allowPinchZoom: true }}
            classNames={{ content: 'equip-select-modal', title: 'equip-select-modal-title' }}
        >
            <SelectedSection
                data={data}
                readOnly={readOnly}
                onClear={clearSlot}
            />

            {!readOnly && (
                <div className="equip-modal-search">
                    <div className="equip-modal-search-row">
                        <input
                            type="text"
                            className="equip-modal-query"
                            placeholder="装備名で検索 (スロットとジョブで自動絞り込み)..."
                            value={query}
                            // モバイルではソフトキーボードが即座に開いて邪魔なので
                            // autofocus しない (undefined で属性ごと外す)
                            data-autofocus={isMobile ? undefined : true}
                            onChange={(e) => handleQueryChange(e.target.value)}
                        />
                        {ilvVisible && (
                            <label className="equip-modal-ilv">
                                <input
                                    type="checkbox"
                                    checked={search.ilv119}
                                    onChange={(e) =>
                                        runSearch(query, {
                                            ...search,
                                            ilv119: e.target.checked,
                                            offset: 0,
                                        })
                                    }
                                />
                                IL119のみ
                            </label>
                        )}
                        <span className="equip-modal-count">
                            {results === null ? '' : `${results.total.toLocaleString()} 件`}
                        </span>
                    </div>
                    <div className="search-page equip-modal-results">
                        {results === null || results.items.length === 0 ? (
                            <div className="loading">該当する装備がありません</div>
                        ) : (
                            <ResultsTable
                                items={results.items}
                                sortBy={search.sortBy}
                                sortOrder={search.sortOrder}
                                descStat=""
                                onHeaderClick={handleHeaderClick}
                                onRowClick={selectItem}
                                selectedId={data?.item_id ?? null}
                            />
                        )}
                        <Pagination
                            withIds={false}
                            total={results?.total ?? 0}
                            offset={search.offset}
                            pageSize={PAGE_SIZE}
                            hasMore={results?.has_more ?? false}
                            onPrev={() =>
                                runSearch(query, {
                                    ...search,
                                    offset: Math.max(0, search.offset - PAGE_SIZE),
                                })
                            }
                            onNext={() =>
                                runSearch(query, { ...search, offset: search.offset + PAGE_SIZE })
                            }
                        />
                    </div>
                </div>
            )}
        </Modal>
    );
}

function SelectedSection({
    data,
    readOnly,
    onClear,
}: {
    data: EquipSlotData | null;
    readOnly: boolean;
    onClear: () => void;
}) {
    if (!data) {
        return (
            <div className="equip-modal-selected empty">
                装備が選択されていません。下の一覧から選択してください。
            </div>
        );
    }

    const augInfo = getItemAugments(data.item_id);
    const augPaths = augInfo?.paths || [];
    const hasAug = augPaths.length > 0;
    const augValue =
        data.aug_path != null && data.aug_rank != null
            ? `${data.aug_path}-${data.aug_rank}`
            : '';
    const augText = getAugmentText(data) || '';
    // 説明文は WASM 埋め込み item DB 由来 (旧実装も innerHTML)。\n → <br>
    const descriptionHtml = (data.description_ja || '').replace(/\\n/g, '<br>');

    return (
        <div className="equip-modal-selected">
            <div className="equip-modal-selected-name">{data.name_ja || data.name_en}</div>
            <div
                className="equip-modal-selected-desc"
                dangerouslySetInnerHTML={{ __html: descriptionHtml }}
            />
            {readOnly ? (
                <>
                    {augText && <div className="equip-modal-aug-text">{augText}</div>}
                    {data.custom_description && (
                        <div className="equip-modal-readonly-custom">
                            カスタム: {data.custom_description}
                        </div>
                    )}
                </>
            ) : (
                <div className="equip-modal-controls">
                    <div className="equip-modal-control-row">
                        <label>オーグメント</label>
                        <select
                            className="equip-modal-aug-path"
                            disabled={!hasAug}
                            value={augValue}
                            onChange={(e) => {
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
                        <span className="equip-modal-aug-text">{augText}</span>
                    </div>
                    <div className="equip-modal-control-row">
                        <label>
                            カスタム説明
                            <button
                                type="button"
                                className="aug-help-btn"
                                title="カスタムオーグメント欄の書き方を表示"
                                onClick={openCustomAugHelp}
                            >
                                ?
                            </button>
                        </label>
                        <input
                            type="text"
                            className="equip-modal-custom-desc"
                            placeholder="例: STR+5 命中+10 ヘイスト+3%"
                            value={data.custom_description || ''}
                            onChange={(e) => {
                                data.custom_description = e.target.value;
                                updateEquipEditStatus();
                            }}
                        />
                    </div>
                    <button type="button" className="btn equip-modal-remove" onClick={onClear}>
                        装備を外す
                    </button>
                </div>
            )}
        </div>
    );
}
