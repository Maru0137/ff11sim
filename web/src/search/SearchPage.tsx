// 装備検索ページの状態の所有者。フォーム状態・フィルタ行・検索結果・
// オフセットをここで持ち、検索の実行 (WASM 呼び出し) もここから行う。
//
// 装備データと検索は Rust 側にある (docs/adr/0009, docs/adr/0010)。
// items.json は WASM に埋め込まれているので JS からは読まない。
import { useEffect, useRef, useState } from 'react';
// スタイルは .search-page スコープに閉じている (index.css との衝突回避)
import '../../styles/search.css';
import { searchItems, loadItems } from '../wasm';
import type { SearchResults } from '../wasm';
import { SearchForm } from './SearchForm';
import { createFilterRow, type FilterRowState } from './FilterRows';
import { ResultsTable } from './ResultsTable';
import { Pagination } from './Pagination';

export interface FormState {
    query: string;
    slot: string;
    job: string;
    ilv119: boolean;
    sortBy: string;
    descStat: string;
    sortOrder: 'asc' | 'desc';
}

const PAGE_SIZE = 50;

// iLv119 フィルタが意味を持つスロット
const ILV119_SLOTS = ['main', 'sub', 'range', 'head', 'body', 'hands', 'legs', 'feet'];

// <select id="sortBy"> に存在するキー。旧実装は th クリックで select に
// 列キーを代入しており、select に無いキー (jobs / slots / description_ja)
// は '' に解決されていた。その挙動を踏襲する (handleHeaderClick)。
const SELECT_SORT_KEYS = ['ja', 'level', 'item_level', 'category', 'desc_stat'];

const INITIAL_FORM: FormState = {
    query: '',
    slot: '',
    job: '',
    ilv119: true,
    sortBy: 'ja',
    descStat: '',
    sortOrder: 'asc',
};

function ilv119VisibleFor(slot: string): boolean {
    return slot === '' || ILV119_SLOTS.includes(slot);
}

function executeSearch(form: FormState, filters: FilterRowState[], offset: number): SearchResults {
    return searchItems({
        query: form.query,
        slot: form.slot,
        job: form.job,
        filters: filters
            .filter((f) => f.property && f.operator && f.value !== '')
            .map(({ property, operator, value }) => ({ property, operator, value })),
        sortBy: form.sortBy,
        descStat: form.descStat,
        sortOrder: form.sortOrder,
        // チェックボックスが非表示のときは検索条件に含めない (旧実装踏襲)
        ilv119Only: form.ilv119 && ilv119VisibleFor(form.slot),
        ilv119Slots: ILV119_SLOTS,
        limit: PAGE_SIZE,
        offset,
    });
}

export function SearchPage() {
    const [form, setForm] = useState(INITIAL_FORM);
    // 直近の検索に使った条件。descStat の入力途中 (未検索) の値で結果表示の
    // ハイライトやソート表示が変わらないよう、表示は searched を参照する。
    const [searched, setSearched] = useState(INITIAL_FORM);
    const [filters, setFilters] = useState<FilterRowState[]>([]);
    const [offset, setOffset] = useState(0);
    const [results, setResults] = useState<SearchResults | null>(null);
    const [loadError, setLoadError] = useState<string | null>(null);
    const nextFilterId = useRef(1);

    useEffect(() => {
        let cancelled = false;
        loadItems()
            .then(() => {
                if (!cancelled) setResults(executeSearch(INITIAL_FORM, [], 0));
            })
            .catch((e: unknown) => {
                if (!cancelled) setLoadError(e instanceof Error ? e.message : String(e));
            });
        return () => {
            cancelled = true;
        };
    }, []);

    function search(nextForm: FormState, nextOffset = 0) {
        setForm(nextForm);
        setSearched(nextForm);
        setOffset(nextOffset);
        setResults(executeSearch(nextForm, filters, nextOffset));
    }

    function handleHeaderClick(columnId: string) {
        if (form.sortBy === columnId) {
            search({ ...form, sortOrder: form.sortOrder === 'asc' ? 'desc' : 'asc' });
        } else {
            const key = SELECT_SORT_KEYS.includes(columnId) ? columnId : '';
            search({ ...form, sortBy: key, sortOrder: 'asc' });
        }
    }

    const countText =
        results === null
            ? '読み込み中...'
            : results.items.length === 0
                ? '0 件'
                : `${results.total.toLocaleString()} 件`;

    return (
        <div className="search-page">
            <SearchForm
                form={form}
                ilv119Visible={ilv119VisibleFor(form.slot)}
                filters={filters}
                onEdit={(patch) => setForm({ ...form, ...patch })}
                onEditAndSearch={(patch) => search({ ...form, ...patch })}
                onSearch={() => search(form)}
                onDescStatCommit={() => {
                    if (form.descStat !== searched.descStat) search(form);
                }}
                onAddFilter={() =>
                    setFilters([...filters, createFilterRow(nextFilterId.current++)])
                }
                onFiltersChange={setFilters}
            />
            <div className="results-section">
                <div className="results-header">
                    <span className="results-count" id="resultsCount">{countText}</span>
                </div>
                <div id="resultsContainer">
                    {loadError !== null ? (
                        <div className="error">データの読み込みに失敗しました: {loadError}</div>
                    ) : results === null ? (
                        <div className="loading">データを読み込み中...</div>
                    ) : results.items.length === 0 ? (
                        <div className="loading">該当する装備がありません</div>
                    ) : (
                        <ResultsTable
                            items={results.items}
                            sortBy={searched.sortBy}
                            sortOrder={searched.sortOrder}
                            descStat={searched.sortBy === 'desc_stat' ? searched.descStat : ''}
                            onHeaderClick={handleHeaderClick}
                        />
                    )}
                </div>
                <Pagination
                    total={results?.total ?? 0}
                    offset={offset}
                    pageSize={PAGE_SIZE}
                    hasMore={results?.has_more ?? false}
                    onPrev={() => search(form, Math.max(0, offset - PAGE_SIZE))}
                    onNext={() => search(form, offset + PAGE_SIZE)}
                />
            </div>
        </div>
    );
}
