// 装備セット編集フォームのツールバー (名前入力 + 保存/複製/共有/インポート/削除)。
// 共有・インポートの処理本体は share-ui.js (レガシー) に残っており、
// ここからは関数として呼び出す。share-mode での表示制御は既存 CSS
// (body.share-mode #btnSaveEquipSet 等) が id を参照するため id を維持する。
//
// 保存は未編集なら無効、複製・共有は「保存済みの内容」に対する操作なので
// 未保存の変更があるときに無効化する (docs/adr/0020)。
import { useSyncExternalStore } from 'react';
import {
    equipSetsStore,
    setNameInput,
    saveEquipSet,
    copyEquipSet,
    deleteEquipSet,
    isEquipSetDirty,
} from './equip-sets-store';
import { subscribeEquipState, getEquipStateVersion } from './equip-store';
import { shareCurrentEquipSet, beginImportShare } from '../share-ui';

export function EquipSetToolbar() {
    const state = useSyncExternalStore(equipSetsStore.subscribe, equipSetsStore.get);
    // スロットの変更は equipState 側の通知しか来ないので、そちらも購読する
    useSyncExternalStore(subscribeEquipState, getEquipStateVersion);

    const dirty = isEquipSetDirty();
    const savedOnlyTitle = dirty ? '未保存の変更があります。先に保存してください' : undefined;

    async function handleSave() {
        const result = await saveEquipSet();
        if (result !== true) alert(result);
    }

    return (
        <div className="equipset-toolbar">
            <div className="form-group" id="equipSetNameGroup">
                <label htmlFor="equipSetName">名前</label>
                <input
                    type="text"
                    id="equipSetName"
                    placeholder="装備セット名"
                    value={state.nameInput}
                    onChange={(e) => setNameInput(e.target.value)}
                />
            </div>
            {dirty && <span className="unsaved-badge">未保存</span>}
            <div className="btn-group" id="equipSetButtonGroup">
                <button
                    className="btn btn-primary"
                    id="btnSaveEquipSet"
                    disabled={!dirty}
                    title={dirty ? undefined : '変更がありません'}
                    onClick={() => void handleSave()}
                >
                    保存
                </button>
                <button
                    className="btn btn-secondary"
                    id="btnCopyEquipSet"
                    disabled={dirty}
                    title={savedOnlyTitle}
                    onClick={copyEquipSet}
                >
                    複製
                </button>
                <button
                    className="btn btn-share"
                    id="btnShareEquipSet"
                    disabled={dirty}
                    title={savedOnlyTitle}
                    onClick={shareCurrentEquipSet}
                >
                    共有
                </button>
                <button
                    className="btn btn-primary"
                    id="btnImportShare"
                    style={{ display: state.shareMode ? undefined : 'none' }}
                    onClick={beginImportShare}
                >
                    インポート
                </button>
                <button
                    className={state.deleteVisible ? 'btn btn-danger' : 'btn btn-danger hidden'}
                    id="btnDeleteEquipSet"
                    onClick={deleteEquipSet}
                >
                    削除
                </button>
            </div>
        </div>
    );
}

