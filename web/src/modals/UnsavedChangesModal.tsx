// 未保存の変更があるまま編集内容を失う操作をしたときの確認ダイアログ
// (docs/adr/0020)。開閉は dirty-guard の guardStore が持つ。
import { useSyncExternalStore } from 'react';
import {
    guardStore,
    guardSaveAndProceed,
    guardDiscardAndProceed,
    guardCancel,
} from '../dirty-guard';

export function UnsavedChangesModal() {
    const prompt = useSyncExternalStore(guardStore.subscribe, guardStore.get);
    if (!prompt) return null;

    return (
        <div id="unsavedChangesModal" className="modal-backdrop">
            <div className="modal-box">
                <h3>保存されていない変更があります</h3>
                <p>
                    {prompt.editor.label()}の変更が保存されていません。
                    <br />
                    このまま続行すると変更内容は失われます。
                </p>
                <p className="modal-note">操作: {prompt.action}</p>
                {prompt.error && <p className="modal-error">{prompt.error}</p>}
                <div className="modal-actions">
                    <button
                        className="btn btn-primary"
                        id="btnUnsavedSave"
                        disabled={prompt.saving}
                        onClick={() => void guardSaveAndProceed()}
                    >
                        {prompt.saving ? '保存中...' : '保存して続行'}
                    </button>
                    <button
                        className="btn btn-danger"
                        id="btnUnsavedDiscard"
                        disabled={prompt.saving}
                        onClick={guardDiscardAndProceed}
                    >
                        破棄して続行
                    </button>
                    <button
                        className="btn btn-neutral"
                        id="btnUnsavedCancel"
                        disabled={prompt.saving}
                        onClick={guardCancel}
                    >
                        キャンセル
                    </button>
                </div>
            </div>
        </div>
    );
}
