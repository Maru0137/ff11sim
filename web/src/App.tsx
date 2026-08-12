// index.html のページ全体。アクティブビューは location.hash が持つ (routing.ts)。
//
// 通常モード: 左サイドバーレイアウト (AppLayout) に 3 ビューを載せる。
// ビューは display:none 切替でアンマウントしない (検索条件等の state 保持)。
// 共有閲覧モード (?share=): サイドバーを出さず装備セットビューのみ表示し、
// CSS (body.share-mode) が編集系 UI を隠す。
import { useState, useSyncExternalStore } from 'react';
import { isShareMode } from './share-ui';
import { parseHash, useHash, type ViewId } from './routing';
import { AuthWidget } from './auth-ui';
import { AppLayout, IconCharacter, IconEquipSet, IconSearch, type ViewMeta } from './AppLayout';
import { Modals } from './modals/Modals';
import { StatusPanel } from './status/StatusPanel';
import { EquipGrid } from './equip/EquipGrid';
import { EquipSetControls } from './equip/EquipSetControls';
import { EquipSetToolbar } from './equip/EquipSetToolbar';
import { CharacterTab } from './character/CharacterTab';
import { SearchPage } from './search/SearchPage';
import { equipSetsStore, shareHeaderStore } from './equip/equip-sets-store';

const VIEWS: ViewMeta[] = [
    {
        view: 'characters',
        label: 'キャラクター',
        description: 'キャラクターの登録・編集',
        icon: <IconCharacter />,
    },
    {
        view: 'equipsets',
        label: '装備セット',
        description: '装備セットの作成とステータス計算',
        icon: <IconEquipSet />,
    },
    {
        view: 'search',
        label: '装備検索',
        description: '装備データベースの検索',
        icon: <IconSearch />,
    },
];

// 装備セットビュー。通常モードと共有閲覧モードの両方から使う。
function EquipSetsView() {
    const panel = useSyncExternalStore(equipSetsStore.subscribe, equipSetsStore.get);
    const shareHeader = useSyncExternalStore(shareHeaderStore.subscribe, shareHeaderStore.get);

    return (
        <>
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
                <EquipGrid />
            </div>
        </>
    );
}

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

    // 共有閲覧モード: サイドバー無しの単票レイアウト
    if (isShareMode()) {
        return (
            <div className="share-layout">
                <header className="page-header">
                    <h1>FF11 ステータスシミュレータ</h1>
                    <div id="auth-ui" className="auth-ui">
                        <AuthWidget />
                    </div>
                </header>
                <div id="tab-equipsets" className="view-content active">
                    <EquipSetsView />
                </div>
                <Modals />
            </div>
        );
    }

    return (
        <AppLayout views={VIEWS} activeView={activeView}>
            <div
                id="tab-characters"
                className={activeView === 'characters' ? 'view-content active' : 'view-content'}
            >
                <CharacterTab />
            </div>

            <div
                id="tab-equipsets"
                className={activeView === 'equipsets' ? 'view-content active' : 'view-content'}
            >
                <EquipSetsView />
            </div>

            <div
                id="tab-search"
                className={activeView === 'search' ? 'view-content active' : 'view-content'}
            >
                {searchMounted && <SearchPage />}
            </div>

            <Modals />
        </AppLayout>
    );
}
