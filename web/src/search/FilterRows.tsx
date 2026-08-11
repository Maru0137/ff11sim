// 動的フィルタ行。プロパティ・演算子・値の 3 つ組で、値の型に応じて
// 演算子の選択肢が入れ替わる。行の編集では検索は実行されない
// (検索ボタンが押されたときにだけ読まれる) — 旧実装の挙動を踏襲。
//
// フィルタのメタデータ (プロパティ一覧 / 演算子一覧) は items.json を
// 参照しない UI 文言なので、このモジュールに置く。

interface FilterableProperty {
    name: string;
    type: 'number' | 'string' | 'array';
    label: string;
}

const FILTERABLE_PROPERTIES: FilterableProperty[] = [
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

const OPERATORS_BY_TYPE: Record<string, { value: string; label: string }[]> = {
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

function operatorsFor(type: string) {
    return OPERATORS_BY_TYPE[type] ?? [{ value: '=', label: '=' }];
}

function propertyType(name: string): string {
    return FILTERABLE_PROPERTIES.find((p) => p.name === name)?.type ?? 'number';
}

export interface FilterRowState {
    id: number;
    property: string;
    operator: string;
    value: string;
}

export function createFilterRow(id: number): FilterRowState {
    const first = FILTERABLE_PROPERTIES[0];
    return { id, property: first.name, operator: operatorsFor(first.type)[0].value, value: '' };
}

interface FilterRowsProps {
    filters: FilterRowState[];
    onChange: (filters: FilterRowState[]) => void;
}

export function FilterRows({ filters, onChange }: FilterRowsProps) {
    function update(id: number, patch: Partial<FilterRowState>) {
        onChange(filters.map((f) => (f.id === id ? { ...f, ...patch } : f)));
    }

    return (
        <div id="filtersContainer">
            {filters.map((f) => (
                <div className="filter-row" key={f.id}>
                    <select
                        className="filter-property"
                        value={f.property}
                        onChange={(e) => {
                            const property = e.target.value;
                            // プロパティの型が変わると演算子リストも変わるため先頭へ戻す
                            update(f.id, {
                                property,
                                operator: operatorsFor(propertyType(property))[0].value,
                            });
                        }}
                    >
                        {FILTERABLE_PROPERTIES.map((p) => (
                            <option key={p.name} value={p.name}>{p.label}</option>
                        ))}
                    </select>
                    <select
                        className="filter-operator"
                        value={f.operator}
                        onChange={(e) => update(f.id, { operator: e.target.value })}
                    >
                        {operatorsFor(propertyType(f.property)).map((op) => (
                            <option key={op.value} value={op.value}>{op.label}</option>
                        ))}
                    </select>
                    <input
                        type="text"
                        className="filter-value"
                        placeholder="値"
                        value={f.value}
                        onChange={(e) => update(f.id, { value: e.target.value })}
                    />
                    <button
                        className="btn btn-danger"
                        onClick={() => onChange(filters.filter((x) => x.id !== f.id))}
                    >
                        ×
                    </button>
                </div>
            ))}
        </div>
    );
}
