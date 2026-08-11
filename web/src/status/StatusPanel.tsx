// 装備編集タブのステータス表示パネル (旧 status-display.js の DOM 書き込みと
// tabs.js のサブタブ切替を統合)。値の計算は compute.ts、テーブル構造は
// StatusTables.tsx (index.html から機械変換)。
import { useEffect, useState, useSyncExternalStore } from 'react';
import { statusStore } from './status-store';
import {
    SUBTABS,
    LeftStatusTables,
    SubtabContents,
    EffectiveSkillsSection,
} from './StatusTables';

const MAGIC_PREFIX = 'subtab-magic-';
const DEFAULT_SUBTAB = 'subtab-defense';

export function StatusPanel() {
    const view = useSyncExternalStore(statusStore.subscribe, statusStore.get);
    const [active, setActive] = useState(DEFAULT_SUBTAB);

    // 魔法タブはジョブが該当スキルを持つ場合のみ表示。
    // view が無い (クリア状態) 間は旧実装同様すべて表示のまま。
    const isVisible = (id: string) =>
        !id.startsWith(MAGIC_PREFIX) || view === null || (view.magicTabVisible[id] ?? true);

    // 旧実装踏襲: active な魔法タブが非表示になったら、
    // 可視の魔法タブ → 既定 (待機/回避/防御) へフォールバック
    useEffect(() => {
        if (
            active.startsWith(MAGIC_PREFIX) &&
            view !== null &&
            !(view.magicTabVisible[active] ?? true)
        ) {
            const firstVisibleMagic = SUBTABS.find(
                (t) => t.id.startsWith(MAGIC_PREFIX) && view.magicTabVisible[t.id]
            );
            setActive(firstVisibleMagic ? firstVisibleMagic.id : DEFAULT_SUBTAB);
        }
    }, [active, view]);

    const v = (id: string): string | number => view?.values[id] ?? '-';

    return (
        <div id="equipStatusSection" className="status-section">
            <h3>ステータス</h3>
            <LeftStatusTables v={v} />

            {/* 用途別ステータスサブタブ */}
            <div className="status-subtabs">
                {SUBTABS.map(({ id, label }) => (
                    <button
                        key={id}
                        type="button"
                        data-subtab={id}
                        className={id === active ? 'status-subtab-btn active' : 'status-subtab-btn'}
                        style={{ display: isVisible(id) ? undefined : 'none' }}
                        onClick={() => setActive(id)}
                    >
                        {label}
                    </button>
                ))}
            </div>

            <SubtabContents v={v} activeId={active} />

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
        </div>
    );
}

