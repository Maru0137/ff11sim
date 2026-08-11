// 検索結果のページネーション。オフセット + has_more 駆動の手組みで、
// TanStack Table の pagination 機能は使わない (ページングは WASM 実行済み)。
interface PaginationProps {
    total: number;
    offset: number;
    pageSize: number;
    hasMore: boolean;
    onPrev: () => void;
    onNext: () => void;
}

export function Pagination({ total, offset, pageSize, hasMore, onPrev, onNext }: PaginationProps) {
    const visible = total > pageSize;
    const currentPage = Math.floor(offset / pageSize) + 1;
    const totalPages = Math.ceil(total / pageSize);

    return (
        <div className="pagination" id="pagination" style={{ display: visible ? 'flex' : 'none' }}>
            <button id="prevBtn" disabled={offset === 0} onClick={onPrev}>前へ</button>
            <span className="page-info" id="pageInfo">
                {visible ? `Page ${currentPage} of ${totalPages}` : ''}
            </span>
            <button id="nextBtn" disabled={!hasMore} onClick={onNext}>次へ</button>
        </div>
    );
}
