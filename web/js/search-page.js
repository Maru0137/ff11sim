// search.html のエントリポイント。装備検索ページのフィルタ UI・検索実行・
// 結果テーブル・ページネーションを持つ。
//
// 装備データと検索は Rust 側に移した (docs/adr/0009, docs/adr/0010)。
// items.json は WASM に埋め込まれているので JS からは読まない。
// フィルタ UI のメタデータ (プロパティ一覧 / 演算子一覧) だけは
// items.json を参照しない UI 文言なので、このモジュールに置いたままにする。
import { search_items, item_count, initWasmRuntime } from './wasm.js';
import { mountAuthUI } from '../src/auth-ui';
import './sync.js';

const FILTERABLE_PROPERTIES = [
    { name: 'id', type: 'number', label: 'ID' },
    { name: 'en', type: 'string', label: 'Name (EN)' },
    { name: 'ja', type: 'string', label: 'Name (JA)' },
    { name: 'category', type: 'string', label: 'Category' },
    { name: 'level', type: 'number', label: 'Level' },
    { name: 'item_level', type: 'number', label: 'Item Level' },
    { name: 'damage', type: 'number', label: 'Damage' },
    { name: 'delay', type: 'number', label: 'Delay' },
    { name: 'skill', type: 'number', label: 'Skill' },
    { name: 'jobs', type: 'array', label: 'Jobs' },
    { name: 'slots', type: 'array', label: 'Slots' },
    { name: 'races', type: 'array', label: 'Races' },
];
const OPERATORS_BY_TYPE = {
    number: [
        { value: '=', label: '=' }, { value: '!=', label: '!=' },
        { value: '>=', label: '>=' }, { value: '<=', label: '<=' },
        { value: '>', label: '>' }, { value: '<', label: '<' },
    ],
    string: [
        { value: 'contains', label: 'contains' }, { value: '=', label: '=' },
        { value: '!=', label: '!=' }, { value: 'starts', label: 'starts with' },
        { value: 'ends', label: 'ends with' },
    ],
    array: [
        { value: 'contains', label: 'contains' },
        { value: 'not_contains', label: 'not contains' },
    ],
};

export function startSearchPage() {
    mountAuthUI(document.getElementById('auth-ui'));

    // 旧 ItemSearch と同じ呼び出し形を保つための薄いアダプタ。
    const itemSearch = {
        search: (options) => search_items(options),
        getFilterableProperties: () => FILTERABLE_PROPERTIES,
        getOperators: (type) => OPERATORS_BY_TYPE[type] ?? [{ value: '=', label: '=' }],
        load: async () => {
            await initWasmRuntime();
            return item_count();
        },
    };
    let currentOffset = 0;
    const pageSize = 50;

    // DOM Elements
    const searchQuery = document.getElementById('searchQuery');
    const slotsFilter = document.getElementById('slotsFilter');
    const jobsFilter = document.getElementById('jobsFilter');
    const ilv119Filter = document.getElementById('ilv119Filter');
    const ilv119Label = document.getElementById('ilv119Label');
    const sortBy = document.getElementById('sortBy');
    const descStatInput = document.getElementById('descStatInput');
    const sortOrder = document.getElementById('sortOrder');
    const searchBtn = document.getElementById('searchBtn');
    const addFilterBtn = document.getElementById('addFilterBtn');
    const filtersContainer = document.getElementById('filtersContainer');
    const resultsContainer = document.getElementById('resultsContainer');
    const resultsCount = document.getElementById('resultsCount');
    const pagination = document.getElementById('pagination');
    const prevBtn = document.getElementById('prevBtn');
    const nextBtn = document.getElementById('nextBtn');
    const pageInfo = document.getElementById('pageInfo');

    // Slots that support iLv119 filter
    const ILV119_SLOTS = ['main', 'sub', 'range', 'head', 'body', 'hands', 'legs', 'feet'];

    function updateIlv119Visibility() {
        const selectedSlot = slotsFilter.value;
        // Show checkbox for: empty (all slots) or slots that support iLv119
        const showCheckbox = selectedSlot === '' || ILV119_SLOTS.includes(selectedSlot);
        ilv119Label.style.display = showCheckbox ? 'flex' : 'none';
    }

    // Initialize
    async function init() {
        try {
            updateIlv119Visibility();
            await itemSearch.load();
            performSearch();
        } catch (error) {
            resultsContainer.innerHTML = `<div class="error">データの読み込みに失敗しました: ${error.message}</div>`;
        }
    }

    // Filter Management
    function addFilter() {
        const properties = itemSearch.getFilterableProperties();
        const filterRow = document.createElement('div');
        filterRow.className = 'filter-row';

        const propSelect = document.createElement('select');
        propSelect.className = 'filter-property';
        for (const prop of properties) {
            const option = document.createElement('option');
            option.value = prop.name;
            option.dataset.type = prop.type;
            option.textContent = prop.label;
            propSelect.appendChild(option);
        }

        const opSelect = document.createElement('select');
        opSelect.className = 'filter-operator';
        updateOperators(opSelect, properties[0].type);

        const valueInput = document.createElement('input');
        valueInput.type = 'text';
        valueInput.className = 'filter-value';
        valueInput.placeholder = '値';

        const removeBtn = document.createElement('button');
        removeBtn.className = 'btn btn-danger';
        removeBtn.textContent = '×';
        removeBtn.addEventListener('click', () => filterRow.remove());

        propSelect.addEventListener('change', () => {
            const selectedOption = propSelect.options[propSelect.selectedIndex];
            updateOperators(opSelect, selectedOption.dataset.type);
        });

        filterRow.appendChild(propSelect);
        filterRow.appendChild(opSelect);
        filterRow.appendChild(valueInput);
        filterRow.appendChild(removeBtn);
        filtersContainer.appendChild(filterRow);
    }

    function updateOperators(select, type) {
        const operators = itemSearch.getOperators(type);
        select.innerHTML = '';
        for (const op of operators) {
            const option = document.createElement('option');
            option.value = op.value;
            option.textContent = op.label;
            select.appendChild(option);
        }
    }

    function getFilters() {
        const filters = [];
        const rows = filtersContainer.querySelectorAll('.filter-row');
        for (const row of rows) {
            const property = row.querySelector('.filter-property').value;
            const operator = row.querySelector('.filter-operator').value;
            const value = row.querySelector('.filter-value').value;
            if (property && operator && value !== '') {
                filters.push({ property, operator, value });
            }
        }
        return filters;
    }

    // Search
    function performSearch(resetOffset = true) {
        if (resetOffset) {
            currentOffset = 0;
        }

        const results = itemSearch.search({
            query: searchQuery.value,
            slot: slotsFilter.value,
            job: jobsFilter.value,
            filters: getFilters(),
            sortBy: sortBy.value,
            descStat: descStatInput.value,
            sortOrder: sortOrder.value,
            ilv119Only: ilv119Filter.checked && ilv119Label.style.display !== 'none',
            ilv119Slots: ILV119_SLOTS,
            limit: pageSize,
            offset: currentOffset
        });

        renderResults(results);
        updatePagination(results);
    }

    function renderResults(results) {
        if (results.items.length === 0) {
            resultsContainer.innerHTML = '<div class="loading">該当する装備がありません</div>';
            resultsCount.textContent = '0 件';
            return;
        }

        resultsCount.textContent = `${results.total.toLocaleString()} 件`;

        const table = document.createElement('table');
        table.className = 'results-table';

        // Header
        const thead = document.createElement('thead');
        const headerRow = document.createElement('tr');
        const columns = [
            { key: 'ja', label: '名前' },
            { key: 'level', label: 'Lv' },
            { key: 'item_level', label: 'iLv' },
            { key: 'jobs', label: 'ジョブ' },
            { key: 'slots', label: 'スロット' },
            { key: 'description_ja', label: '説明' },
        ];

        // Job abbreviation to Japanese kanji mapping
        const jobKanji = {
            'WAR': '戦', 'MNK': 'モ', 'WHM': '白', 'BLM': '黒', 'RDM': '赤', 'THF': 'シ',
            'PLD': 'ナ', 'DRK': '暗', 'BST': '獣', 'BRD': '吟', 'RNG': '狩', 'SAM': '侍',
            'NIN': '忍', 'DRG': '竜', 'SMN': '召', 'BLU': '青', 'COR': 'コ', 'PUP': 'か',
            'DNC': '踊', 'SCH': '学', 'GEO': '風', 'RUN': '剣'
        };

        const slotJa = {
            'main': 'メイン', 'sub': 'サブ', 'range': '遠隔', 'ammo': '矢弾',
            'head': '頭', 'body': '胴', 'hands': '両手', 'legs': '両脚',
            'feet': '両足', 'neck': '首', 'waist': '腰', 'ear1': '耳',
            'ear2': '耳', 'ring1': '指', 'ring2': '指', 'back': '背'
        };

        function slotsToJa(slots) {
            if (!slots || slots.length === 0) return '-';
            const seen = new Set();
            return slots.map(s => slotJa[s] || s).filter(s => {
                if (seen.has(s)) return false;
                seen.add(s);
                return true;
            }).join(' ');
        }

        function jobsToKanji(jobs) {
            if (!jobs || jobs.length === 0) return '-';
            if (jobs.length === 22) return 'All Jobs';
            return jobs.map(j => jobKanji[j] || j).join('');
        }

        function highlightStat(description, statName) {
            if (!description || !statName) return description || '';

            // Full-width to half-width for matching
            const fullToHalf = {
                'Ａ': 'A', 'Ｂ': 'B', 'Ｃ': 'C', 'Ｄ': 'D', 'Ｅ': 'E', 'Ｆ': 'F', 'Ｇ': 'G',
                'Ｈ': 'H', 'Ｉ': 'I', 'Ｊ': 'J', 'Ｋ': 'K', 'Ｌ': 'L', 'Ｍ': 'M', 'Ｎ': 'N',
                'Ｏ': 'O', 'Ｐ': 'P', 'Ｑ': 'Q', 'Ｒ': 'R', 'Ｓ': 'S', 'Ｔ': 'T', 'Ｕ': 'U',
                'Ｖ': 'V', 'Ｗ': 'W', 'Ｘ': 'X', 'Ｙ': 'Y', 'Ｚ': 'Z',
                '０': '0', '１': '1', '２': '2', '３': '3', '４': '4',
                '５': '5', '６': '6', '７': '7', '８': '8', '９': '9',
                '＋': '+', '－': '-', '―': '-'
            };

            // Normalize for pattern matching
            let normalized = '';
            for (const char of description) {
                normalized += fullToHalf[char] || char;
            }

            // Single regex to match all patterns: "防77", "DEF:77", "STR+5", "DMG:+165"
            const statUpper = statName.toUpperCase();
            const normalizedUpper = normalized.toUpperCase();
            const highlightRegex = new RegExp(`${statUpper}\\s*(?::\\s*)?[+\\-]?\\s*\\d+`, 'i');
            const match = normalizedUpper.match(highlightRegex);

            if (match) {
                const idx = match.index;
                const matchLen = match[0].length;
                const before = description.substring(0, idx);
                const matched = description.substring(idx, idx + matchLen);
                const after = description.substring(idx + matchLen);
                return before + '<span class="stat-highlight">' + matched + '</span>' + after;
            }

            return description;
        }

        for (const col of columns) {
            const th = document.createElement('th');
            th.textContent = col.label;
            th.dataset.key = col.key;
            th.addEventListener('click', () => sortByColumn(col.key));

            if (sortBy.value === col.key) {
                th.classList.add(sortOrder.value === 'asc' ? 'sorted-asc' : 'sorted-desc');
            }

            headerRow.appendChild(th);
        }
        thead.appendChild(headerRow);
        table.appendChild(thead);

        // Body
        const tbody = document.createElement('tbody');
        const isStatSort = sortBy.value === 'desc_stat' && descStatInput.value;
        for (const item of results.items) {
            const row = document.createElement('tr');

            let descHtml = (item.description_ja || '').replace(/\\n/g, '\n');
            if (isStatSort) {
                descHtml = highlightStat(descHtml, descStatInput.value);
            }

            row.innerHTML = `
                <td>${item.ja || ''}</td>
                <td>${item.level !== undefined ? item.level : '-'}</td>
                <td>${item.item_level !== undefined ? item.item_level : '-'}</td>
                <td class="jobs-list">${jobsToKanji(item.jobs)}</td>
                <td class="slots-list">${slotsToJa(item.slots)}</td>
                <td class="description">${descHtml}</td>
            `;

            tbody.appendChild(row);
        }
        table.appendChild(tbody);

        resultsContainer.innerHTML = '';
        resultsContainer.appendChild(table);
    }

    function sortByColumn(key) {
        if (sortBy.value === key) {
            sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc';
        } else {
            sortBy.value = key;
            sortOrder.value = 'asc';
        }
        performSearch();
    }

    function updatePagination(results) {
        if (results.total <= pageSize) {
            pagination.style.display = 'none';
            return;
        }

        pagination.style.display = 'flex';
        const currentPage = Math.floor(currentOffset / pageSize) + 1;
        const totalPages = Math.ceil(results.total / pageSize);
        pageInfo.textContent = `Page ${currentPage} of ${totalPages}`;

        prevBtn.disabled = currentOffset === 0;
        // SearchResult は Rust 側で rename されず snake_case で返る
        nextBtn.disabled = !results.has_more;
    }

    // Event Listeners
    searchBtn.addEventListener('click', () => performSearch());
    addFilterBtn.addEventListener('click', addFilter);

    searchQuery.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') performSearch();
    });

    slotsFilter.addEventListener('change', () => {
        updateIlv119Visibility();
        performSearch();
    });
    jobsFilter.addEventListener('change', () => performSearch());
    ilv119Filter.addEventListener('change', () => performSearch());
    sortBy.addEventListener('change', () => {
        descStatInput.style.display = sortBy.value === 'desc_stat' ? 'inline-block' : 'none';
        performSearch();
    });
    descStatInput.addEventListener('change', () => performSearch());
    descStatInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') performSearch();
    });
    sortOrder.addEventListener('change', () => performSearch());

    prevBtn.addEventListener('click', () => {
        currentOffset = Math.max(0, currentOffset - pageSize);
        performSearch(false);
    });

    nextBtn.addEventListener('click', () => {
        currentOffset += pageSize;
        performSearch(false);
    });

    // Start
    init();
}
