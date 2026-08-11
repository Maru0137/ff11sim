// 検索条件フォーム。検索の実行タイミングは旧実装のリスナー構成と 1:1:
//   - select / checkbox の変更 → 即検索 (onEditAndSearch)
//   - テキスト入力 (query / descStat) → 状態更新のみ (onEdit)。
//     Enter または検索ボタンで検索。descStat はフォーカスアウト時に
//     値が変わっていれば検索 (旧 change イベント相当、onDescStatCommit)
import type { KeyboardEvent } from 'react';
import type { FormState } from './SearchPage';
import { FilterRows } from './FilterRows';
import type { FilterRowState } from './FilterRows';

interface SearchFormProps {
    form: FormState;
    ilv119Visible: boolean;
    filters: FilterRowState[];
    onEdit: (patch: Partial<FormState>) => void;
    onEditAndSearch: (patch: Partial<FormState>) => void;
    onSearch: () => void;
    onDescStatCommit: () => void;
    onAddFilter: () => void;
    onFiltersChange: (filters: FilterRowState[]) => void;
}

export function SearchForm({
    form, ilv119Visible, filters,
    onEdit, onEditAndSearch, onSearch, onDescStatCommit, onAddFilter, onFiltersChange,
}: SearchFormProps) {
    // IME 変換確定の Enter では検索しない (旧実装の keypress は変換中の
    // Enter で発火しなかった。keydown に移すため isComposing で除外する)
    const searchOnEnter = (e: KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter' && !e.nativeEvent.isComposing) onSearch();
    };

    return (
        <div className="search-section">
            <div className="search-row">
                <input
                    type="text" id="searchQuery" className="search-input"
                    placeholder="装備名で検索..."
                    value={form.query}
                    onChange={(e) => onEdit({ query: e.target.value })}
                    onKeyDown={searchOnEnter}
                />
                <select
                    id="slotsFilter" value={form.slot}
                    onChange={(e) => onEditAndSearch({ slot: e.target.value })}
                >
                    <option value="">全スロット</option>
                    <option value="main">メイン</option>
                    <option value="sub">サブ</option>
                    <option value="range">遠隔</option>
                    <option value="ammo">矢弾</option>
                    <option value="head">頭</option>
                    <option value="body">胴</option>
                    <option value="hands">両手</option>
                    <option value="legs">両脚</option>
                    <option value="feet">両足</option>
                    <option value="neck">首</option>
                    <option value="waist">腰</option>
                    <option value="ear1">耳</option>
                    <option value="ring1">指</option>
                    <option value="back">背</option>
                </select>
                <select
                    id="jobsFilter" value={form.job}
                    onChange={(e) => onEditAndSearch({ job: e.target.value })}
                >
                    <option value="">All Jobs</option>
                    <option value="WAR">戦</option>
                    <option value="MNK">モ</option>
                    <option value="WHM">白</option>
                    <option value="BLM">黒</option>
                    <option value="RDM">赤</option>
                    <option value="THF">シ</option>
                    <option value="PLD">ナ</option>
                    <option value="DRK">暗</option>
                    <option value="BST">獣</option>
                    <option value="BRD">吟</option>
                    <option value="RNG">狩</option>
                    <option value="SAM">侍</option>
                    <option value="NIN">忍</option>
                    <option value="DRG">竜</option>
                    <option value="SMN">召</option>
                    <option value="BLU">青</option>
                    <option value="COR">コ</option>
                    <option value="PUP">か</option>
                    <option value="DNC">踊</option>
                    <option value="SCH">学</option>
                    <option value="GEO">風</option>
                    <option value="RUN">剣</option>
                </select>
                <label
                    id="ilv119Label"
                    style={{
                        display: ilv119Visible ? 'flex' : 'none',
                        alignItems: 'center', gap: 5, cursor: 'pointer',
                    }}
                >
                    <input
                        type="checkbox" id="ilv119Filter"
                        checked={form.ilv119}
                        onChange={(e) => onEditAndSearch({ ilv119: e.target.checked })}
                    />
                    <span>iLv119のみ</span>
                </label>
            </div>

            <div className="filters-section">
                <div className="filters-header">
                    <h3>フィルター</h3>
                    <button className="btn btn-secondary" id="addFilterBtn" onClick={onAddFilter}>
                        + フィルター追加
                    </button>
                </div>
                <FilterRows filters={filters} onChange={onFiltersChange} />
            </div>

            <div className="sort-section">
                <label>ソート:</label>
                <select
                    id="sortBy" value={form.sortBy}
                    onChange={(e) => onEditAndSearch({ sortBy: e.target.value })}
                >
                    <option value="ja">名前</option>
                    <option value="level">Lv</option>
                    <option value="item_level">iLv</option>
                    <option value="category">カテゴリ</option>
                    <option value="desc_stat">説明文ステータス</option>
                </select>
                <input
                    type="text" id="descStatInput" placeholder="例: STR, 攻"
                    style={{
                        display: form.sortBy === 'desc_stat' ? 'inline-block' : 'none',
                        width: 100,
                    }}
                    value={form.descStat}
                    onChange={(e) => onEdit({ descStat: e.target.value })}
                    onKeyDown={searchOnEnter}
                    onBlur={onDescStatCommit}
                />
                <select
                    id="sortOrder" value={form.sortOrder}
                    onChange={(e) => onEditAndSearch({ sortOrder: e.target.value as 'asc' | 'desc' })}
                >
                    <option value="asc">昇順</option>
                    <option value="desc">降順</option>
                </select>
                <button className="btn btn-primary" id="searchBtn" onClick={onSearch}>検索</button>
            </div>
        </div>
    );
}
