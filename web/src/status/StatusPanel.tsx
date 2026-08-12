// 装備編集タブのステータス表示パネル。
// 用途別ステータスはサブタブバーからプロパティセットのドロップダウンに
// 置き換え (docs/adr/0015)。テンプレート (現行 19 タブ) の表示 JSX は
// StatusTables.tsx の SubtabContents をそのまま使い、カスタムセットは
// CustomPropsetGrid で描画する。選択は装備セットごとに記憶する
// (selection-prefs、ローカル専用)。
import { useEffect, useState, useSyncExternalStore } from 'react';
import { statusStore } from './status-store';
import {
    SUBTABS,
    LeftStatusTables,
    SubtabContents,
    EffectiveSkillsSection,
} from './StatusTables';
import { equipState, subscribeEquipState, getEquipStateVersion } from '../equip/equip-store';
import { isShareMode } from '../share-ui';
import { propsetsStore } from '../propsets/propsets-store';
import { CustomPropsetGrid } from '../propsets/CustomPropsetGrid';
import { PropsetManageModal } from '../propsets/PropsetManageModal';
import {
    equipSetPrefKey,
    getSelectedPropsetId,
    setSelectedPropsetId,
} from '../propsets/selection-prefs';
import '../../styles/propsets.css';

const TEMPLATE_PREFIX = 'template:';
const MAGIC_SELECTION_PREFIX = 'template:subtab-magic-';
const DEFAULT_SELECTION = 'template:subtab-defense';

export function StatusPanel() {
    const view = useSyncExternalStore(statusStore.subscribe, statusStore.get);
    const propsets = useSyncExternalStore(propsetsStore.subscribe, propsetsStore.get);
    // editingEquipSetName (装備セット切替) の変化を拾う
    useSyncExternalStore(subscribeEquipState, getEquipStateVersion);
    const [selection, setSelection] = useState(DEFAULT_SELECTION);
    const [modal, setModal] = useState<{ setId?: string } | null>(null);

    const readOnly = isShareMode();
    const currentSetKey = equipState.editingEquipSetName
        ? equipSetPrefKey(
              equipState.currentEquipChar,
              equipState.currentEquipJob,
              equipState.editingEquipSetName
          )
        : null;

    // 魔法タブはジョブが該当スキルを持つ場合のみ表示。
    // view が無い (クリア状態) 間は旧実装同様すべて表示のまま。
    const isVisible = (subtabId: string) =>
        !subtabId.startsWith('subtab-magic-') ||
        view === null ||
        (view.magicTabVisible[subtabId] ?? true);

    // 装備セット切替時: 記憶していた選択を検証付きで復元
    useEffect(() => {
        if (!currentSetKey) return;
        const stored = getSelectedPropsetId(currentSetKey);
        if (!stored) {
            setSelection(DEFAULT_SELECTION);
            return;
        }
        const valid = stored.startsWith(TEMPLATE_PREFIX)
            ? SUBTABS.some((t) => t.id === stored.slice(TEMPLATE_PREFIX.length))
            : propsetsStore.get().sets.some((s) => s.id === stored);
        setSelection(valid ? stored : DEFAULT_SELECTION);
    }, [currentSetKey]);

    // 旧実装踏襲: 選択中の魔法テンプレートが非表示になったら、
    // 可視の魔法テンプレート → 既定 (待機/回避/防御) へフォールバック
    useEffect(() => {
        if (!selection.startsWith(MAGIC_SELECTION_PREFIX) || view === null) return;
        const subtabId = selection.slice(TEMPLATE_PREFIX.length);
        if (view.magicTabVisible[subtabId] ?? true) return;
        const firstVisibleMagic = SUBTABS.find(
            (t) => t.id.startsWith('subtab-magic-') && view.magicTabVisible[t.id]
        );
        setSelection(firstVisibleMagic ? TEMPLATE_PREFIX + firstVisibleMagic.id : DEFAULT_SELECTION);
    }, [selection, view]);

    // 選択中のカスタムセットが削除されたら既定へフォールバック
    useEffect(() => {
        if (selection.startsWith(TEMPLATE_PREFIX) || !propsets.loaded) return;
        if (!propsets.sets.some((s) => s.id === selection)) {
            setSelection(DEFAULT_SELECTION);
        }
    }, [selection, propsets]);

    const handleSelect = (id: string) => {
        setSelection(id);
        if (currentSetKey && !readOnly) {
            setSelectedPropsetId(currentSetKey, id);
        }
    };

    const v = (id: string): string | number => view?.values[id] ?? '-';

    const customSet = selection.startsWith(TEMPLATE_PREFIX)
        ? null
        : propsets.sets.find((s) => s.id === selection) ?? null;

    return (
        <div id="equipStatusSection" className="status-section">
            <h3>ステータス</h3>
            <LeftStatusTables v={v} />

            {/* 用途別ステータス: プロパティセット選択 */}
            <div className="propset-selector">
                <label htmlFor="propsetSelect">プロパティセット:</label>
                <select
                    id="propsetSelect"
                    value={selection}
                    onChange={(e) => handleSelect(e.target.value)}
                >
                    <optgroup label="テンプレート">
                        {SUBTABS.filter((t) => isVisible(t.id)).map((t) => (
                            <option key={t.id} value={TEMPLATE_PREFIX + t.id}>
                                {t.label}
                            </option>
                        ))}
                    </optgroup>
                    {propsets.sets.length > 0 && (
                        <optgroup label="カスタム">
                            {propsets.sets.map((s) => (
                                <option key={s.id} value={s.id}>
                                    {s.name}
                                </option>
                            ))}
                        </optgroup>
                    )}
                </select>
                {!readOnly && (
                    <>
                        <button
                            type="button"
                            className="propset-manage-btn"
                            onClick={() => setModal({})}
                        >
                            ⚙ プロパティセット管理
                        </button>
                        {customSet && (
                            <button
                                type="button"
                                className="propset-manage-btn"
                                onClick={() => setModal({ setId: customSet.id })}
                            >
                                ✎ このセットを編集
                            </button>
                        )}
                    </>
                )}
            </div>

            {customSet ? (
                <div className="status-subtab-content active">
                    <CustomPropsetGrid set={customSet} userItems={propsets.userItems} view={view} />
                </div>
            ) : (
                <SubtabContents v={v} activeId={selection.slice(TEMPLATE_PREFIX.length)} />
            )}

            <EffectiveSkillsSection>
                {view === null ? null : view.effectiveSkills.length === 0 ? (
                    <div style={{ color: '#666' }}>(表示できるスキルなし)</div>
                ) : (
                    view.effectiveSkills.map((s) => (
                        <div key={s.key}>
                            <span style={{ color: '#888' }}>{s.ja}:</span> <strong>{s.value}</strong>
                            {s.isMain && (
                                <>
                                    {' '}
                                    <span style={{ color: '#8ab4f8' }}>(主武器)</span>
                                </>
                            )}
                        </div>
                    ))
                )}
            </EffectiveSkillsSection>

            {modal !== null && (
                <PropsetManageModal
                    initialSetId={modal.setId}
                    onClose={() => setModal(null)}
                />
            )}
        </div>
    );
}
