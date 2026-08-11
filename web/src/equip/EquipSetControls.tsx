// 装備セットタブのセレクタ (キャラ/ジョブ/サポ) とタブバー
// (旧 web/js/equip-sets.js の描画部分)。状態は equip-sets-store。
import { useState, useSyncExternalStore } from 'react';
import { JOBS, RACE_NAMES } from '../../js/constants.js';
import { equipState, subscribeEquipState, getEquipStateVersion } from './equip-store';
import {
    equipSetsStore,
    selectChar,
    selectJob,
    selectSupportJob,
    selectSet,
    newSet,
    reorderEquipSetTabs,
} from './equip-sets-store';

interface JobDef {
    key: string;
    name: string;
}

const jobs = JOBS as JobDef[];

export function EquipSetControls() {
    const state = useSyncExternalStore(equipSetsStore.subscribe, equipSetsStore.get);
    // セレクタの値は equipState 直読みのため、その変更通知も購読する
    useSyncExternalStore(subscribeEquipState, getEquipStateVersion);
    const [draggingName, setDraggingName] = useState<string | null>(null);
    const [dragOverName, setDragOverName] = useState<string | null>(null);

    const tabBarVisible = !!(equipState.currentEquipChar && equipState.currentEquipJob);

    return (
        <>
            <div className="equipset-selectors">
                <div className="form-group">
                    <label htmlFor="equipSelectChar">キャラクター</label>
                    <select
                        id="equipSelectChar"
                        value={equipState.currentEquipChar}
                        onChange={(e) => selectChar(e.target.value)}
                    >
                        <option value="">-- キャラクターを選択 --</option>
                        {state.characters.map((ch) => (
                            <option key={ch.name} value={ch.name}>
                                {`${ch.name} (${RACE_NAMES[ch.race] || ch.race})`}
                            </option>
                        ))}
                    </select>
                </div>
                <div className="form-group">
                    <label htmlFor="equipSelectJob">ジョブ</label>
                    <select
                        id="equipSelectJob"
                        disabled={!equipState.currentEquipChar}
                        value={equipState.currentEquipJob}
                        onChange={(e) => selectJob(e.target.value)}
                    >
                        <option value="">-- ジョブを選択 --</option>
                        {jobs.map((j) => (
                            <option key={j.key} value={j.key}>{j.name}</option>
                        ))}
                    </select>
                </div>
                <div className="form-group">
                    <label htmlFor="equipSelectSupportJob">サポートジョブ</label>
                    <select
                        id="equipSelectSupportJob"
                        disabled={!equipState.currentEquipJob}
                        value={equipState.currentEquipSupportJob}
                        onChange={(e) => selectSupportJob(e.target.value)}
                    >
                        <option value="">-- なし --</option>
                        {/* 旧実装踏襲: value は小文字。メインジョブの除外条件
                            (key.toLowerCase() === currentEquipJob) は大文字小文字の
                            不一致で実質機能していなかったため、そのまま全ジョブを出す */}
                        {jobs.map((j) => (
                            <option key={j.key} value={j.key.toLowerCase()}>{j.name}</option>
                        ))}
                    </select>
                </div>
            </div>

            <div
                id="equipSetTabBar"
                className={tabBarVisible ? 'equipset-tabs' : 'equipset-tabs hidden'}
            >
                {state.sets.map((es) => (
                    <button
                        key={es.name}
                        className={[
                            'equipset-tab',
                            es.name === state.selectedName ? 'active' : '',
                            es.name === draggingName ? 'dragging' : '',
                            es.name === dragOverName ? 'drag-over' : '',
                        ].filter(Boolean).join(' ')}
                        data-equipset-name={es.name}
                        draggable
                        onClick={() => selectSet(es.name)}
                        onDragStart={(e) => {
                            e.dataTransfer.effectAllowed = 'move';
                            e.dataTransfer.setData('text/plain', es.name);
                            setDraggingName(es.name);
                        }}
                        onDragEnd={() => setDraggingName(null)}
                        onDragOver={(e) => {
                            e.preventDefault();
                            e.dataTransfer.dropEffect = 'move';
                            setDragOverName(es.name);
                        }}
                        onDragLeave={() => setDragOverName(null)}
                        onDrop={(e) => {
                            e.preventDefault();
                            setDragOverName(null);
                            const fromName = e.dataTransfer.getData('text/plain');
                            if (fromName && fromName !== es.name) {
                                reorderEquipSetTabs(fromName, es.name);
                            }
                        }}
                    >
                        {es.name}
                    </button>
                ))}
                <button className="equipset-tab-add" onClick={newSet}>+</button>
            </div>
        </>
    );
}

