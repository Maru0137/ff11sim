// 検索結果テーブル。TanStack Table v9 (ヘッドレス) を使う (docs/adr/0012)。
//
// 検索・ソート・ページングはすべて WASM (rust/src/item_search.rs) が実行
// 済みなので、TanStack の役割は列定義とソート状態の表現のみ。
// manualSorting: true でクライアント側の再ソートを無効化しており、
// 行はデータ順のまま描画される。
import {
    createColumnHelper,
    rowSortingFeature,
    tableFeatures,
    useTable,
} from '@tanstack/react-table';
import type { SortingState } from '@tanstack/react-table';
import type { SearchResultItem } from '../wasm';
import { jobsToKanji, slotsToJa, highlightStat } from './format';

const features = tableFeatures({ rowSortingFeature });
const helper = createColumnHelper<typeof features, SearchResultItem>();

const columns = helper.columns([
    helper.accessor('ja', { header: '名前' }),
    helper.accessor('level', {
        header: 'Lv',
        cell: ({ cell }) => cell.getValue() ?? '-',
    }),
    helper.accessor('item_level', {
        header: 'iLv',
        cell: ({ cell }) => cell.getValue() ?? '-',
    }),
    helper.accessor('jobs', {
        header: 'ジョブ',
        cell: ({ cell }) => jobsToKanji(cell.getValue()),
    }),
    helper.accessor('slots', {
        header: 'スロット',
        cell: ({ cell }) => slotsToJa(cell.getValue()),
    }),
    // description_ja は stat-highlight の span を挿すため td 側で HTML 描画する
    helper.accessor('description_ja', { header: '説明' }),
]);

const SORTABLE_COLUMN_IDS = ['ja', 'level', 'item_level'];
const TD_CLASS: Record<string, string> = {
    jobs: 'jobs-list',
    slots: 'slots-list',
    description_ja: 'description',
};

interface ResultsTableProps {
    items: SearchResultItem[];
    /** 直近の検索に使ったソート条件 (th の sorted-asc/desc 表示用) */
    sortBy: string;
    sortOrder: 'asc' | 'desc';
    /** sortBy が desc_stat のときのハイライト対象。それ以外は '' */
    descStat: string;
    onHeaderClick: (columnId: string) => void;
    /** 行クリックで選択する用途 (装備選択モーダル) 向け。未指定なら行は非インタラクティブ */
    onRowClick?: (item: SearchResultItem) => void;
    /** selected-row でハイライトするアイテム id */
    selectedId?: number | null;
}

export function ResultsTable({
    items,
    sortBy,
    sortOrder,
    descStat,
    onHeaderClick,
    onRowClick,
    selectedId,
}: ResultsTableProps) {
    // WASM 側で適用済みのソートを table 状態として持たせる (manual なので
    // 行の並びには影響しない)。sortBy が列以外 (category / desc_stat) の
    // ときは空。
    const sorting: SortingState = SORTABLE_COLUMN_IDS.includes(sortBy)
        ? [{ id: sortBy, desc: sortOrder === 'desc' }]
        : [];

    const table = useTable({
        features,
        columns,
        data: items,
        manualSorting: true,
        state: { sorting },
        getRowId: (row) => String(row.id),
    });

    return (
        <table className="results-table">
            <thead>
                {table.getHeaderGroups().map((group) => (
                    <tr key={group.id}>
                        {group.headers.map((header) => (
                            <th
                                key={header.id}
                                data-key={header.column.id}
                                className={
                                    sortBy === header.column.id
                                        ? (sortOrder === 'asc' ? 'sorted-asc' : 'sorted-desc')
                                        : undefined
                                }
                                onClick={() => onHeaderClick(header.column.id)}
                            >
                                <table.FlexRender header={header} />
                            </th>
                        ))}
                    </tr>
                ))}
            </thead>
            <tbody>
                {table.getRowModel().rows.map((row) => (
                    <tr
                        key={row.id}
                        className={
                            selectedId != null && row.original.id === selectedId
                                ? 'selected-row'
                                : undefined
                        }
                        onClick={onRowClick ? () => onRowClick(row.original) : undefined}
                    >
                        {row.getAllCells().map((cell) =>
                            cell.column.id === 'description_ja' ? (
                                <DescriptionCell
                                    key={cell.id}
                                    text={row.original.description_ja}
                                    descStat={descStat}
                                />
                            ) : (
                                <td key={cell.id} className={TD_CLASS[cell.column.id]}>
                                    <table.FlexRender cell={cell} />
                                </td>
                            )
                        )}
                    </tr>
                ))}
            </tbody>
        </table>
    );
}

function DescriptionCell({ text, descStat }: { text?: string; descStat: string }) {
    // データ上の改行はリテラルの "\n" (rust/src/items.rs)。実改行へ直し、
    // 表示は CSS の .description { white-space: pre-line } が担う。
    let html = (text || '').replace(/\\n/g, '\n');
    if (descStat) {
        html = highlightStat(html, descStat);
    }
    // 説明文は WASM 埋め込みの item DB 由来で、旧実装も innerHTML で描画して
    // いた (同じ信頼境界)。stat-highlight の span を挿すためここだけ HTML。
    return <td className="description" dangerouslySetInnerHTML={{ __html: html }} />;
}
