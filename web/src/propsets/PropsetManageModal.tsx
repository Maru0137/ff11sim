// プロパティセット管理モーダル (docs/adr/0015)。
// StatusPanel のローカル state で開閉する (ADR 0014: React 専用モーダルに
// modal-store は使わない)。1 枚の Modal 内で一覧 ⇄ 編集をビュー状態機械で
// 切り替え、モーダルの重なり (z-index 梯子) を増やさない。
//
// Mantine は Modal のみ。内部の入力は index.css の既存手書きパターン
// (素の input/button) に乗せる (EquipSelectModal と同じ方針)。
import { useState, useSyncExternalStore } from 'react';
import { Modal } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import { updateEquipEditStatus } from '../status/status-store';
import { TEMPLATE_PROPSETS, TEMPLATE_PROPSET_GROUPS } from '../status/StatusTables';
import {
    BUILTIN_PROPERTY_ITEMS,
    PROPERTY_CATEGORIES,
    TEMPLATE_ITEM_IDS,
} from './catalog';
import {
    propsetsStore,
    savePropertySet,
    deletePropertySet,
    addUserItem,
    deleteUserItem,
} from './propsets-store';
import type { PropertySet } from './types';
import '../../styles/propsets.css';

type View = { kind: 'list' } | { kind: 'edit'; draft: PropertySet; isNew: boolean };

export function PropsetManageModal({
    onClose,
    initialSetId,
}: {
    onClose: () => void;
    /** 指定時はそのカスタムセットの編集ビューで開く (「このセットを編集」用) */
    initialSetId?: string;
}) {
    // getInitialValueInEffect: false — 初回レンダーから fullScreen を確定させる
    const isMobile = useMediaQuery('(max-width: 768px)', false, {
        getInitialValueInEffect: false,
    });
    const [view, setView] = useState<View>(() => {
        if (initialSetId) {
            const s = propsetsStore.get().sets.find((x) => x.id === initialSetId);
            if (s) return { kind: 'edit', draft: { ...s, items: [...s.items] }, isNew: false };
        }
        return { kind: 'list' };
    });

    return (
        <Modal
            opened
            onClose={onClose}
            title={view.kind === 'list' ? 'プロパティセット管理' : 'カスタムセットの編集'}
            size={760}
            // AppShell (zIndex 200) より上、レガシーモーダル (z-index 1000) より下
            zIndex={900}
            fullScreen={isMobile}
            // スクロールロック中もピンチズーム/パンを許可 (iOS Safari 対策)
            removeScrollProps={{ allowPinchZoom: true }}
        >
            {view.kind === 'list' ? (
                <ListView
                    onEdit={(draft, isNew) => setView({ kind: 'edit', draft, isNew })}
                    onClose={onClose}
                />
            ) : (
                <EditView
                    draft={view.draft}
                    isNew={view.isNew}
                    onBack={() => setView({ kind: 'list' })}
                />
            )}
        </Modal>
    );
}

function ListView({
    onEdit,
    onClose,
}: {
    onEdit: (draft: PropertySet, isNew: boolean) => void;
    onClose: () => void;
}) {
    const { sets } = useSyncExternalStore(propsetsStore.subscribe, propsetsStore.get);

    const newSet = (name: string, items: string[]) =>
        onEdit({ id: crypto.randomUUID(), name, items }, true);

    return (
        <div className="propset-manage">
            <h4 className="propset-section-title">カスタム</h4>
            {sets.length === 0 && (
                <div className="propset-row propset-row-empty">カスタムセットはまだありません</div>
            )}
            {sets.map((s) => (
                <div key={s.id} className="propset-row">
                    <span className="propset-row-name">{s.name}</span>
                    <span className="propset-row-meta">{s.items.length} 項目</span>
                    <button type="button" onClick={() => onEdit({ ...s, items: [...s.items] }, false)}>
                        編集
                    </button>
                    <button
                        type="button"
                        className="propset-btn-danger"
                        onClick={async () => {
                            if (!confirm(`プロパティセット「${s.name}」を削除しますか？`)) return;
                            await deletePropertySet(s.id);
                        }}
                    >
                        削除
                    </button>
                </div>
            ))}

            {TEMPLATE_PROPSET_GROUPS.map((group) => (
                <div key={group}>
                    <h4 className="propset-section-title">{group}</h4>
                    {TEMPLATE_PROPSETS.filter((t) => t.group === group).map(({ id, label }) => (
                        <div key={id} className="propset-row">
                            <span className="propset-row-name">{label}</span>
                            <span className="propset-row-meta">
                                {(TEMPLATE_ITEM_IDS[id] ?? []).length} 項目
                            </span>
                            <button
                                type="button"
                                onClick={() =>
                                    newSet(`${label}のコピー`, [...(TEMPLATE_ITEM_IDS[id] ?? [])])
                                }
                            >
                                複製して編集
                            </button>
                        </div>
                    ))}
                </div>
            ))}

            <div className="propset-modal-actions">
                <button type="button" onClick={() => newSet('', [])}>
                    ＋ 新規カスタムセット
                </button>
                <button type="button" onClick={onClose}>
                    閉じる
                </button>
            </div>
        </div>
    );
}

function EditView({
    draft,
    isNew,
    onBack,
}: {
    draft: PropertySet;
    isNew: boolean;
    onBack: () => void;
}) {
    const { userItems } = useSyncExternalStore(propsetsStore.subscribe, propsetsStore.get);
    const [name, setName] = useState(draft.name);
    const [items, setItems] = useState<string[]>(draft.items);
    const [query, setQuery] = useState('');
    const [newTerm, setNewTerm] = useState('');
    const [draggingId, setDraggingId] = useState<string | null>(null);
    const [dragOverId, setDragOverId] = useState<string | null>(null);

    const toggle = (id: string) =>
        setItems((prev) => (prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]));

    const labelOf = (id: string) => {
        const builtin = BUILTIN_PROPERTY_ITEMS.find((b) => b.id === id);
        if (builtin) return builtin.label;
        return userItems.find((u) => u.id === id)?.term ?? id;
    };

    async function handleAddUserItem() {
        const term = newTerm.trim();
        if (!term) return;
        const item = await addUserItem(term);
        setItems((prev) => (prev.includes(item.id) ? prev : [...prev, item.id]));
        setNewTerm('');
        // 新しい term の抽出値を propertyValues に載せる
        await updateEquipEditStatus();
    }

    async function handleDeleteUserItem(id: string, term: string) {
        if (!confirm(`ユーザー定義項目「${term}」を削除しますか？\n全カスタムセットからも取り除かれます。`)) {
            return;
        }
        await deleteUserItem(id);
        setItems((prev) => prev.filter((i) => i !== id));
        await updateEquipEditStatus();
    }

    async function handleSave() {
        const trimmed = name.trim();
        if (!trimmed) {
            alert('セット名を入力してください。');
            return;
        }
        await savePropertySet({ id: draft.id, name: trimmed, items });
        onBack();
    }

    // 並び替え: draggingId を dropTarget の位置へ挿入 (EquipSetControls と同じ HTML5 DnD)
    function moveItem(fromId: string, toId: string) {
        setItems((prev) => {
            const fromIdx = prev.indexOf(fromId);
            const toIdx = prev.indexOf(toId);
            if (fromIdx < 0 || toIdx < 0 || fromIdx === toIdx) return prev;
            const next = [...prev];
            next.splice(fromIdx, 1);
            next.splice(toIdx, 0, fromId);
            return next;
        });
    }

    const q = query.trim();
    return (
        <div className="propset-edit">
            <div className="propset-edit-name">
                <label htmlFor="propsetNameInput">セット名</label>
                <input
                    id="propsetNameInput"
                    type="text"
                    maxLength={30}
                    value={name}
                    placeholder="例: 手数重視"
                    onChange={(e) => setName(e.target.value)}
                />
            </div>

            <div className="propset-edit-cols">
                <div className="propset-catalog">
                    <div className="propset-col-title">プロパティ項目カタログ</div>
                    <input
                        type="search"
                        className="propset-catalog-search"
                        placeholder="項目名で絞り込み…"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                    />
                    {PROPERTY_CATEGORIES.map((cat) => {
                        const hits = BUILTIN_PROPERTY_ITEMS.filter(
                            (b) => b.category === cat && (!q || b.label.includes(q))
                        );
                        if (hits.length === 0) return null;
                        return (
                            <div key={cat} className="propset-cat-group">
                                <div className="propset-cat-title">{cat}</div>
                                <div className="propset-chips">
                                    {hits.map((b) => (
                                        <button
                                            key={b.id}
                                            type="button"
                                            className={
                                                'propset-chip' +
                                                (items.includes(b.id) ? ' selected' : '')
                                            }
                                            onClick={() => toggle(b.id)}
                                        >
                                            {b.label}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        );
                    })}
                    <div className="propset-cat-group">
                        <div className="propset-cat-title">ユーザー定義</div>
                        <div className="propset-chips">
                            {userItems
                                .filter((u) => !q || u.term.includes(q))
                                .map((u) => (
                                    <span
                                        key={u.id}
                                        className={
                                            'propset-chip propset-chip-user' +
                                            (items.includes(u.id) ? ' selected' : '')
                                        }
                                        role="button"
                                        tabIndex={0}
                                        onClick={() => toggle(u.id)}
                                        onKeyDown={(e) => {
                                            if (e.key === 'Enter' || e.key === ' ') {
                                                e.preventDefault();
                                                toggle(u.id);
                                            }
                                        }}
                                    >
                                        {u.term}
                                        <button
                                            type="button"
                                            className="propset-chip-del"
                                            title="この定義を削除"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                handleDeleteUserItem(u.id, u.term);
                                            }}
                                        >
                                            ✕
                                        </button>
                                    </span>
                                ))}
                        </div>
                        <div className="propset-user-add">
                            <input
                                type="text"
                                maxLength={20}
                                placeholder="例: 二刀流"
                                value={newTerm}
                                onChange={(e) => setNewTerm(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') {
                                        e.preventDefault();
                                        handleAddUserItem();
                                    }
                                }}
                            />
                            <button type="button" onClick={handleAddUserItem}>
                                ＋ 追加
                            </button>
                        </div>
                        <div className="propset-cat-hint">
                            装備説明文から「入力した文字列+N」を抽出して合算します。
                        </div>
                    </div>
                </div>

                <div className="propset-selected">
                    <div className="propset-col-title">選択中の項目 (表示順 / ⠿ をドラッグで並び替え)</div>
                    {items.length === 0 && (
                        <div className="propset-selected-empty">
                            左のカタログから項目を選択してください
                        </div>
                    )}
                    {items.map((id) => (
                        <div
                            key={id}
                            className={[
                                'propset-sel-row',
                                id === draggingId ? 'dragging' : '',
                                id === dragOverId ? 'drag-over' : '',
                            ].filter(Boolean).join(' ')}
                            onDragOver={(e) => {
                                e.preventDefault();
                                e.dataTransfer.dropEffect = 'move';
                                setDragOverId(id);
                            }}
                            onDragLeave={() => setDragOverId(null)}
                            onDrop={(e) => {
                                e.preventDefault();
                                const fromId = e.dataTransfer.getData('text/plain');
                                if (fromId) moveItem(fromId, id);
                                setDraggingId(null);
                                setDragOverId(null);
                            }}
                        >
                            <span
                                className="propset-drag-handle"
                                title="ドラッグで並び替え"
                                draggable
                                onDragStart={(e) => {
                                    e.dataTransfer.effectAllowed = 'move';
                                    e.dataTransfer.setData('text/plain', id);
                                    setDraggingId(id);
                                }}
                                onDragEnd={() => {
                                    setDraggingId(null);
                                    setDragOverId(null);
                                }}
                            >
                                ⠿
                            </span>
                            <span className="propset-sel-name">{labelOf(id)}</span>
                            <button type="button" onClick={() => toggle(id)}>
                                ✕
                            </button>
                        </div>
                    ))}
                </div>
            </div>

            <div className="propset-modal-actions">
                <button type="button" onClick={onBack}>
                    {isNew ? 'キャンセル' : '戻る'}
                </button>
                <button type="button" className="propset-btn-primary" onClick={handleSave}>
                    保存
                </button>
            </div>
        </div>
    );
}
