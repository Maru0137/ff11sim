// index.html の 3 モーダル (共有 URL / インポート先選択 / カスタムオーグヘルプ)。
// 開閉は modal-store 経由 (開くきっかけがレガシー JS 側にもあるため)。
// 旧実装との対応: 背景クリックで閉じるのはカスタムオーグヘルプのみ。
import { useRef, useState, useSyncExternalStore } from 'react';
import { createRoot } from 'react-dom/client';
import { JOBS, RACE_NAMES } from '../../js/constants.js';
import { loadEquipSets, saveEquipSets } from '../../js/storage.js';
import {
    modalsStore,
    closeCustomAugHelp,
    closeShareUrlModal,
    closeImportShareModal,
    type ImportSharePayload,
} from './modal-store';

interface JobDef {
    key: string;
    name: string;
}

function Modals() {
    const state = useSyncExternalStore(modalsStore.subscribe, modalsStore.get);
    return (
        <>
            {state.shareUrl !== null && <ShareUrlModal url={state.shareUrl} />}
            {state.importShare !== null && (
                <ImportShareModal
                    key={state.importShare.openCount}
                    payload={state.importShare}
                />
            )}
            {state.customAugHelpOpen && <CustomAugHelpModal />}
        </>
    );
}

export function mountModals(container: HTMLElement) {
    createRoot(container).render(<Modals />);
}

function ShareUrlModal({ url }: { url: string }) {
    const inputRef = useRef<HTMLInputElement>(null);
    const [copyLabel, setCopyLabel] = useState('URL をコピー');

    async function copy() {
        try {
            await navigator.clipboard.writeText(url);
            setCopyLabel('コピーしました!');
            setTimeout(() => setCopyLabel('URL をコピー'), 1500);
        } catch {
            // フォールバック: 選択するだけ
            inputRef.current?.select();
        }
    }

    return (
        <div id="shareUrlModal" className="modal-backdrop">
            <div className="modal-box">
                <h3>装備セットを共有</h3>
                <p>下記 URL をコピーして共有してください。閲覧者はこの装備セットを見て、自分のキャラクターにインポートできます。</p>
                <div className="form-group">
                    <input type="text" id="shareUrlInput" readOnly value={url} ref={inputRef} />
                </div>
                <div className="modal-actions">
                    <button className="btn btn-secondary" id="btnCopyShareUrl" onClick={copy}>
                        {copyLabel}
                    </button>
                    <button className="btn btn-danger" id="btnCloseShareUrlModal" onClick={closeShareUrlModal}>
                        閉じる
                    </button>
                </div>
            </div>
        </div>
    );
}

function ImportShareModal({ payload }: { payload: ImportSharePayload }) {
    const { characters, shared } = payload;
    const jobs = JOBS as JobDef[];
    const [character, setCharacter] = useState(characters[0]?.name ?? '');
    const [job, setJob] = useState(
        shared.job && jobs.some((j) => j.key === shared.job) ? shared.job : jobs[0]?.key ?? ''
    );
    const [name, setName] = useState(shared.name || '共有装備セット');

    async function confirmImport() {
        const finalBase = name.trim();
        if (!character || !job || !finalBase) {
            alert('キャラクター・ジョブ・名前を全て指定してください。');
            return;
        }
        const sets = await loadEquipSets();
        // 重複名は (2), (3) ... を付与して回避
        const used = new Set<string>(
            sets
                .filter((s: { character: string; job: string }) => s.character === character && s.job === job)
                .map((s: { name: string }) => s.name)
        );
        let finalName = finalBase;
        let i = 2;
        while (used.has(finalName)) finalName = `${finalBase} (${i++})`;

        sets.push({
            name: finalName,
            character,
            job,
            slots: { ...(shared.slots || {}) },
        });
        try {
            await saveEquipSets(sets);
            alert(`「${finalName}」としてインポートしました。`);
            closeImportShareModal();
            // share=URL を外して通常モードに戻る
            window.location.href = window.location.origin + window.location.pathname;
        } catch (e) {
            console.error('import failed:', e);
            alert('インポートに失敗しました: ' + (e instanceof Error ? e.message : e));
        }
    }

    return (
        <div id="importShareModal" className="modal-backdrop">
            <div className="modal-box">
                <h3>装備セットをインポート</h3>
                <p id="importShareDescription">
                    {`共有元: ${shared.characterName || '(未設定)'} / ${shared.job || '(未設定)'} / ${shared.name}`}
                </p>
                <div className="form-group">
                    <label htmlFor="importShareCharSelect">対象キャラクター</label>
                    <select
                        id="importShareCharSelect"
                        value={character}
                        onChange={(e) => setCharacter(e.target.value)}
                    >
                        {characters.map((ch) => (
                            <option key={ch.name} value={ch.name}>
                                {`${ch.name} (${RACE_NAMES[ch.race] || ch.race})`}
                            </option>
                        ))}
                    </select>
                </div>
                <div className="form-group">
                    <label htmlFor="importShareJobSelect">対象ジョブ</label>
                    <select id="importShareJobSelect" value={job} onChange={(e) => setJob(e.target.value)}>
                        {jobs.map((j) => (
                            <option key={j.key} value={j.key}>{j.name}</option>
                        ))}
                    </select>
                </div>
                <div className="form-group">
                    <label htmlFor="importShareNameInput">装備セット名</label>
                    <input
                        type="text"
                        id="importShareNameInput"
                        placeholder="装備セット名"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                    />
                </div>
                <div className="modal-actions">
                    <button className="btn btn-primary" id="btnConfirmImportShare" onClick={confirmImport}>
                        インポート
                    </button>
                    <button className="btn btn-danger" id="btnCancelImportShare" onClick={closeImportShareModal}>
                        キャンセル
                    </button>
                </div>
            </div>
        </div>
    );
}

function CustomAugHelpModal() {
    return (
        <div
            id="customAugHelpModal"
            className="modal-backdrop"
            onClick={(e) => {
                // 背景クリックで閉じる
                if (e.target === e.currentTarget) closeCustomAugHelp();
            }}
        >
            <div className="modal-box modal-large aug-help-content">
                <h3>カスタムオーグメントの書き方</h3>
                <p>「カスタム」欄には、装備の説明欄に表示されるオーグメント効果（オーグメント候補に無い効果も含む）をテキストで自由に追加できます。</p>
                <p><strong>基本書式</strong>: <code>項目名+数値</code> または <code>項目名-数値</code>（複数項目は半角空白で区切る）</p>
                <p>例: <code>STR+11 命中+10 ヘイスト+3%</code></p>
                <h4>主な対応項目</h4>
                <table>
                    <thead><tr><th>分類</th><th>日本語表記</th><th>英語表記</th><th>記入例</th></tr></thead>
                    <tbody>
                        <tr><td rowSpan={3}>基本ステータス</td><td>HP / MP</td><td>HP / MP</td><td><code>HP+50</code> <code>MP+5%</code></td></tr>
                        <tr><td>STR / DEX / VIT / AGI / INT / MND / CHR</td><td>同左</td><td><code>STR+11 DEX+11</code></td></tr>
                        <tr><td colSpan={2}>スラッシュ区切りで複数指定可</td><td><code>STR/VIT+10</code></td></tr>

                        <tr><td rowSpan={6}>戦闘ステータス</td><td>命中</td><td>Accuracy</td><td><code>命中+15</code></td></tr>
                        <tr><td>攻 (攻撃)</td><td>Attack</td><td><code>攻+20</code> <code>攻+3%</code></td></tr>
                        <tr><td>回避</td><td>Evasion</td><td><code>回避+10</code></td></tr>
                        <tr><td>飛命 / 飛攻</td><td>Ranged Accuracy / Ranged Attack</td><td><code>飛攻+25</code></td></tr>
                        <tr><td>魔命 / 魔攻 / 魔回避</td><td>Magic Accuracy / "Magic Atk. Bonus" / Magic Evasion</td><td><code>魔攻+15</code></td></tr>
                        <tr><td>魔法ダメージ</td><td>Magic Damage</td><td><code>魔法ダメージ+10</code></td></tr>

                        <tr><td rowSpan={6}>特殊効果</td><td>ヘイスト+N%</td><td>Haste+N%</td><td><code>ヘイスト+3%</code></td></tr>
                        <tr><td>ダブルアタック+N%</td><td>"Double Attack"+N%</td><td><code>ダブルアタック+5%</code></td></tr>
                        <tr><td>トリプルアタック+N%</td><td>"Triple Attack"+N%</td><td><code>トリプルアタック+3%</code></td></tr>
                        <tr><td>クワッドアタック+N%</td><td>"Quadruple Attack"+N%</td><td><code>クワッドアタック+2%</code></td></tr>
                        <tr><td>クリティカルヒット率+N%</td><td>Critical hit rate+N%</td><td><code>クリティカルヒット率+5%</code></td></tr>
                        <tr><td>クリティカルヒットダメージ+N%</td><td>Critical hit damage+N%</td><td><code>クリティカルヒットダメージ+10%</code></td></tr>

                        <tr><td rowSpan={6}>WS / 連携 / TP</td><td>ウェポンスキルのダメージ+N%</td><td>Weapon skill damage+N%</td><td><code>ウェポンスキルのダメージ+5%</code></td></tr>
                        <tr><td>ストアTP+N</td><td>"Store TP"+N</td><td><code>ストアTP+10</code></td></tr>
                        <tr><td>TPボーナス+N</td><td>"TP Bonus"+N</td><td><code>TPボーナス+500</code></td></tr>
                        <tr><td>連携ボーナス+N / 連携ダメージ+N%</td><td>"Skillchain Bonus"+N</td><td><code>連携ボーナス+10</code></td></tr>
                        <tr><td>マジックバーストダメージ+N%</td><td>Magic burst damage+N%</td><td><code>マジックバーストダメージ+10%</code></td></tr>
                        <tr><td>モクシャ / モクシャII</td><td>"Subtle Blow" / "Subtle Blow II"</td><td><code>モクシャ+10</code></td></tr>
                    </tbody>
                </table>
                <p className="note">※ 日本語で書いた項目は内部で英語表記に自動変換されてからパースされます。一覧に無い項目（例: <code>"Triple Attack"+3%</code>）は装備説明欄と同じ英語表記で直接記入することもできます。</p>
                <p className="note">※ 数値の前に <code>+</code> または <code>-</code>、末尾に <code>%</code> を付けると割合扱いです（HP/MP/Attack の一部のみ対応）。</p>
                <div className="modal-actions">
                    <button className="btn btn-danger" id="btnCloseCustomAugHelp" onClick={closeCustomAugHelp}>
                        閉じる
                    </button>
                </div>
            </div>
        </div>
    );
}
