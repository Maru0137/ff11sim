// 装備セット編集フォームのツールバー (名前入力 + 保存/複製/共有/インポート/削除)。
// 共有・インポートの処理本体は share-ui.js (レガシー) に残っており、
// ここからは関数として呼び出す。share-mode での表示制御は既存 CSS
// (body.share-mode #btnSaveEquipSet 等) が id を参照するため id を維持する。
import { useSyncExternalStore } from 'react';
import {
    equipSetsStore,
    setNameInput,
    saveEquipSet,
    copyEquipSet,
    deleteEquipSet,
} from './equip-sets-store';
import { shareCurrentEquipSet, beginImportShare } from '../share-ui';

export function EquipSetToolbar() {
    const state = useSyncExternalStore(equipSetsStore.subscribe, equipSetsStore.get);

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
            <div className="btn-group" id="equipSetButtonGroup">
                <button className="btn btn-primary" id="btnSaveEquipSet" onClick={saveEquipSet}>
                    保存
                </button>
                <button className="btn btn-secondary" id="btnCopyEquipSet" onClick={copyEquipSet}>
                    複製
                </button>
                <button className="btn btn-share" id="btnShareEquipSet" onClick={shareCurrentEquipSet}>
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

