// index.html のページ全体 (旧 main.js のマウント群 + tabs.js の React 統合)。
// アクティブビューは location.hash が持つ (routing.ts)。共有閲覧モード
// (?share=) では装備セットビュー固定で、CSS (body.share-mode) が編集系 UI を隠す。
import { useState, useSyncExternalStore } from 'react';
import { isShareMode } from './share-ui';
import { parseHash, navigate, useHash, type ViewId } from './routing';
import { AuthWidget } from './auth-ui';
import { Modals } from './modals/Modals';
import { StatusPanel } from './status/StatusPanel';
import { EquipSlots } from './equip/EquipSlots';
import { EquipSetControls } from './equip/EquipSetControls';
import { EquipSetToolbar } from './equip/EquipSetToolbar';
import { CharacterTab } from './character/CharacterTab';
import { SearchPage } from './search/SearchPage';
import { equipSetsStore, shareHeaderStore } from './equip/equip-sets-store';

const TABS: { view: ViewId; label: string }[] = [
    { view: 'characters', label: 'キャラクター管理' },
    { view: 'equipsets', label: '装備セット' },
    { view: 'search', label: '装備検索' },
];

export function App() {
    const hash = useHash();
    const activeView: ViewId = isShareMode()
        ? 'equipsets'
        : (parseHash(hash) ?? 'characters');
    // 検索ビューは初回表示まで遅延マウント。SearchPage は mount 時に全件検索を
    // 走らせるため、使われないまま起動時コストを払わない。一度 mount した後は
    // display:none 切替なので検索条件・結果の state は保持される。
    const [searchMounted, setSearchMounted] = useState(false);
    if (activeView === 'search' && !searchMounted) setSearchMounted(true);
    const panel = useSyncExternalStore(equipSetsStore.subscribe, equipSetsStore.get);
    const shareHeader = useSyncExternalStore(shareHeaderStore.subscribe, shareHeaderStore.get);

    return (
        <>
            <header className="page-header">
                <h1>FF11 ステータスシミュレータ</h1>
                <div id="auth-ui" className="auth-ui">
                    <AuthWidget />
                </div>
            </header>

            <div className="tabs">
                {TABS.map((t) => (
                    <button
                        key={t.view}
                        className={t.view === activeView ? 'tab-btn active' : 'tab-btn'}
                        data-tab={`tab-${t.view}`}
                        onClick={() => navigate(t.view)}
                    >
                        {t.label}
                    </button>
                ))}
            </div>

            <div
                id="tab-characters"
                className={activeView === 'characters' ? 'tab-content active' : 'tab-content'}
            >
                <CharacterTab />
            </div>

            <div
                id="tab-equipsets"
                className={activeView === 'equipsets' ? 'tab-content active' : 'tab-content'}
            >
                <EquipSetControls />

                <div id="equipEditSection" className={panel.editVisible ? '' : 'hidden'}>
                    <div
                        id="sharedHeader"
                        className="shared-header"
                        style={{ display: shareHeader.visible ? undefined : 'none' }}
                    >
                        <span className="shared-badge">共有された装備セット</span>
                        <span className="shared-meta">
                            <strong id="sharedSetName">{shareHeader.name}</strong>
                            <span id="sharedSetMeta">{shareHeader.meta}</span>
                        </span>
                    </div>

                    <EquipSetToolbar />
                    <StatusPanel />
                    <div id="equipSlotsContainer" className="equip-slot-grid">
                        <EquipSlots />
                    </div>
                </div>
            </div>

            <div
                id="tab-search"
                className={activeView === 'search' ? 'tab-content active' : 'tab-content'}
            >
                {searchMounted && <SearchPage />}
            </div>

            <Modals />
        </>
    );
}
