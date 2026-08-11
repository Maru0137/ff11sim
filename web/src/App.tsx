// index.html のページ全体 (旧 main.js のマウント群 + tabs.js の React 統合)。
// メインタブの状態はここが持つ。共有閲覧モード (?share=) では装備セット
// タブを初期表示し、CSS (body.share-mode) が編集系 UI を隠す。
import { useState, useSyncExternalStore } from 'react';
import { isShareMode } from './share-ui';
import { AuthWidget } from './auth-ui';
import { Modals } from './modals/Modals';
import { StatusPanel } from './status/StatusPanel';
import { EquipSlots } from './equip/EquipSlots';
import { EquipSetControls } from './equip/EquipSetControls';
import { EquipSetToolbar } from './equip/EquipSetToolbar';
import { CharacterTab } from './character/CharacterTab';
import { equipSetsStore, shareHeaderStore } from './equip/equip-sets-store';

const TABS = [
    { id: 'tab-characters', label: 'キャラクター管理' },
    { id: 'tab-equipsets', label: '装備セット' },
];

export function App() {
    const [activeTab, setActiveTab] = useState(
        isShareMode() ? 'tab-equipsets' : 'tab-characters'
    );
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
                        key={t.id}
                        className={t.id === activeTab ? 'tab-btn active' : 'tab-btn'}
                        data-tab={t.id}
                        onClick={() => setActiveTab(t.id)}
                    >
                        {t.label}
                    </button>
                ))}
            </div>

            <div
                id="tab-characters"
                className={activeTab === 'tab-characters' ? 'tab-content active' : 'tab-content'}
            >
                <CharacterTab />
            </div>

            <div
                id="tab-equipsets"
                className={activeTab === 'tab-equipsets' ? 'tab-content active' : 'tab-content'}
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

            <Modals />
        </>
    );
}
