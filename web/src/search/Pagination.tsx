// 検索結果のページネーション。オフセット + has_more 駆動の手組みで、
// TanStack Table の pagination 機能は使わない (ページングは WASM 実行済み)。
interface PaginationProps {
    total: number;
    offset: number;
    pageSize: number;
    hasMore: boolean;
    onPrev: () => void;
    onNext: () => void;
    /** 検索ビューと装備選択モーダルの同時マウントで id が重複しないよう、
        2 箇所目以降の利用では false にする (id は検索ビューのものとして残す) */
    withIds?: boolean;
}

export function Pagination({
    total,
    offset,
    pageSize,
    hasMore,
    onPrev,
    onNext,
    withIds = true,
}: PaginationProps) {
    const visible = total > pageSize;
    const currentPage = Math.floor(offset / pageSize) + 1;
    const totalPages = Math.ceil(total / pageSize);

    return (
        <div
            className="pagination"
            id={withIds ? 'pagination' : undefined}
            style={{ display: visible ? 'flex' : 'none' }}
        >
            <button id={withIds ? 'prevBtn' : undefined} disabled={offset === 0} onClick={onPrev}>前へ</button>
            <span className="page-info" id={withIds ? 'pageInfo' : undefined}>
                {visible ? `Page ${currentPage} of ${totalPages}` : ''}
            </span>
            <button id={withIds ? 'nextBtn' : undefined} disabled={!hasMore} onClick={onNext}>次へ</button>
        </div>
    );
}
