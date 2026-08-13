// 装備編集タブのステータス表示テーブル群。
// 左の常時表示テーブル (LeftStatusTables) は固定 JSX、プロパティセットの
// テンプレートは template-defs.ts の宣言的定義を汎用レンダラで描画する。
// 値は v(id) 経由で参照する。id は旧実装の DOM id と同一に保っている。
import type { ReactNode } from 'react';

import {
    TEMPLATE_PROPSET_DEFS,
    type TemplateFlag,
    type TemplateTableDef,
} from './template-defs';

export type ValueGetter = (id: string) => string | number;

// プロパティセット選択 UI (StatusPanel / PropsetManageModal) 向けの再エクスポート。
// 一覧の実体は template-defs.ts (表示順 = 配列順、docs/adr/0015)
export { TEMPLATE_PROPSET_GROUPS } from './template-defs';
export { TEMPLATE_PROPSET_DEFS as TEMPLATE_PROPSETS } from './template-defs';

function subtabClass(id: string, activeId: string): string {
    return id === activeId ? 'status-subtab-content active' : 'status-subtab-content';
}

export function LeftStatusTables({ v }: { v: ValueGetter }) {
    return (
        <div className="status-tables-row">
            <table className="status-table">
                <thead>
                    <tr>
                        <th></th>
                        <th>素</th>
                        <th>装備</th>
                        <th>合計</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>HP</td>
                        <td className="base-val" id="equipBaseHp">{v('equipBaseHp')}</td>
                        <td className="equip-val" id="equipEquipHp">{v('equipEquipHp')}</td>
                        <td className="total-val" id="equipTotalHp">{v('equipTotalHp')}</td>
                    </tr>
                    <tr>
                        <td>MP</td>
                        <td className="base-val" id="equipBaseMp">{v('equipBaseMp')}</td>
                        <td className="equip-val" id="equipEquipMp">{v('equipEquipMp')}</td>
                        <td className="total-val" id="equipTotalMp">{v('equipTotalMp')}</td>
                    </tr>
                    <tr>
                        <td>STR</td>
                        <td className="base-val" id="equipBaseStr">{v('equipBaseStr')}</td>
                        <td className="equip-val" id="equipEquipStr">{v('equipEquipStr')}</td>
                        <td className="total-val" id="equipTotalStr">{v('equipTotalStr')}</td>
                    </tr>
                    <tr>
                        <td>DEX</td>
                        <td className="base-val" id="equipBaseDex">{v('equipBaseDex')}</td>
                        <td className="equip-val" id="equipEquipDex">{v('equipEquipDex')}</td>
                        <td className="total-val" id="equipTotalDex">{v('equipTotalDex')}</td>
                    </tr>
                    <tr>
                        <td>VIT</td>
                        <td className="base-val" id="equipBaseVit">{v('equipBaseVit')}</td>
                        <td className="equip-val" id="equipEquipVit">{v('equipEquipVit')}</td>
                        <td className="total-val" id="equipTotalVit">{v('equipTotalVit')}</td>
                    </tr>
                    <tr>
                        <td>AGI</td>
                        <td className="base-val" id="equipBaseAgi">{v('equipBaseAgi')}</td>
                        <td className="equip-val" id="equipEquipAgi">{v('equipEquipAgi')}</td>
                        <td className="total-val" id="equipTotalAgi">{v('equipTotalAgi')}</td>
                    </tr>
                    <tr>
                        <td>INT</td>
                        <td className="base-val" id="equipBaseInt">{v('equipBaseInt')}</td>
                        <td className="equip-val" id="equipEquipInt">{v('equipEquipInt')}</td>
                        <td className="total-val" id="equipTotalInt">{v('equipTotalInt')}</td>
                    </tr>
                    <tr>
                        <td>MND</td>
                        <td className="base-val" id="equipBaseMnd">{v('equipBaseMnd')}</td>
                        <td className="equip-val" id="equipEquipMnd">{v('equipEquipMnd')}</td>
                        <td className="total-val" id="equipTotalMnd">{v('equipTotalMnd')}</td>
                    </tr>
                    <tr>
                        <td>CHR</td>
                        <td className="base-val" id="equipBaseChr">{v('equipBaseChr')}</td>
                        <td className="equip-val" id="equipEquipChr">{v('equipEquipChr')}</td>
                        <td className="total-val" id="equipTotalChr">{v('equipTotalChr')}</td>
                    </tr>
                </tbody>
            </table>

            <table className="status-table">
                <thead>
                    <tr>
                        <th></th>
                        <th>素</th>
                        <th>装備</th>
                        <th>合計</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>ヘイスト</td>
                        <td className="base-val">-</td>
                        <td className="equip-val" id="equipEquipHaste">{v('equipEquipHaste')}</td>
                        <td className="total-val" id="equipTotalHaste">{v('equipTotalHaste')}</td>
                    </tr>
                    <tr>
                        <td>防御力</td>
                        <td className="base-val" id="equipBaseDef">{v('equipBaseDef')}</td>
                        <td className="equip-val" id="equipEquipDef">{v('equipEquipDef')}</td>
                        <td className="total-val" id="equipTotalDef">{v('equipTotalDef')}</td>
                    </tr>
                    <tr>
                        <td>回避</td>
                        <td className="base-val" id="equipBaseEva">{v('equipBaseEva')}</td>
                        <td className="equip-val" id="equipEquipEva">{v('equipEquipEva')}</td>
                        <td className="total-val" id="equipTotalEva">{v('equipTotalEva')}</td>
                    </tr>
                    <tr>
                        <td>魔防</td>
                        <td className="base-val" id="equipBaseMdef">{v('equipBaseMdef')}</td>
                        <td className="equip-val" id="equipEquipMdef">{v('equipEquipMdef')}</td>
                        <td className="total-val" id="equipTotalMdef">{v('equipTotalMdef')}</td>
                    </tr>
                    <tr>
                        <td>魔回避</td>
                        <td className="base-val" id="equipBaseMeva">{v('equipBaseMeva')}</td>
                        <td className="equip-val" id="equipEquipMeva">{v('equipEquipMeva')}</td>
                        <td className="total-val" id="equipTotalMeva">{v('equipTotalMeva')}</td>
                    </tr>
                    {/* 被ダメ系は軽減方向 (負値) が望ましいため「〜-」表記で符号を
                        反転して表示する (compute.ts 側で反転済み)。-30% → 30% */}
                    <tr>
                        <td>被ダメージ-</td>
                        <td className="base-val">-</td>
                        <td className="equip-val" id="equipEquipDt">{v('equipEquipDt')}</td>
                        <td className="total-val" id="equipTotalDt">{v('equipTotalDt')}</td>
                    </tr>
                    <tr>
                        <td>被物理ダメージ-</td>
                        <td className="base-val">-</td>
                        <td className="equip-val" id="equipEquipPdt">{v('equipEquipPdt')}</td>
                        <td className="total-val" id="equipTotalPdt">{v('equipTotalPdt')}</td>
                    </tr>
                    <tr>
                        <td>被魔法ダメージ-</td>
                        <td className="base-val">-</td>
                        <td className="equip-val" id="equipEquipMdt">{v('equipEquipMdt')}</td>
                        <td className="total-val" id="equipTotalMdt">{v('equipTotalMdt')}</td>
                    </tr>
                </tbody>
            </table>
        </div>
    );
}

/** テンプレートテーブル 1 枚の汎用レンダラ (template-defs.ts の定義を描画) */
function TemplateTable({ def, v, flags }: {
    def: TemplateTableDef;
    v: ValueGetter;
    flags: Record<TemplateFlag, boolean>;
}) {
    const visible = (f?: TemplateFlag) => f === undefined || flags[f];
    if (!visible(def.visibleIf)) return null;

    const colIndexes = def.columns
        .map((_, i) => i)
        .filter((i) => visible(def.columns[i].visibleIf));
    const rows = def.rows.filter((r) => visible(r.visibleIf));
    const hasRowLabels = def.rows.some((r) => r.label !== undefined);

    return (
        <table className="status-table">
            <thead>
                {def.title !== undefined && (
                    <tr><th colSpan={colIndexes.length + (hasRowLabels ? 1 : 0)}>{def.title}</th></tr>
                )}
                <tr>
                    {hasRowLabels && <th></th>}
                    {colIndexes.map((i) => <th key={i}>{def.columns[i].label}</th>)}
                </tr>
            </thead>
            <tbody>
                {rows.map((row, ri) => (
                    <tr key={ri}>
                        {hasRowLabels && <td>{row.label}</td>}
                        {colIndexes.map((i) => {
                            const id = row.cells[i];
                            return id === null
                                ? <td key={i} className="equip-val">-</td>
                                : <td key={i} className="equip-val" id={id}>{v(id)}</td>;
                        })}
                    </tr>
                ))}
            </tbody>
        </table>
    );
}

export function SubtabContents({ v, activeId, showRangedWsRow, songInstrument, geoHandbell }: {
    v: ValueGetter;
    activeId: string;
    /** レンジスロットが弓術/射撃のときのみ WS 系テンプレートのレンジ行を表示する */
    showRangedWsRow: boolean;
    /** レンジスロットの楽器種別。呪歌テンプレートの楽器スキル列切替に使う */
    songInstrument: 'wind' | 'string' | null;
    /** レンジスロットに風水鈴装備時のみ風水テンプレートの風水鈴スキル列を表示する */
    geoHandbell: boolean;
}) {
    const flags: Record<TemplateFlag, boolean> = {
        rangedWs: showRangedWsRow,
        songString: songInstrument === 'string',
        songWind: songInstrument === 'wind',
        geoHandbell,
    };
    return (
        <>
            {TEMPLATE_PROPSET_DEFS.map((def) => (
                <div key={def.id} id={def.id} className={subtabClass(def.id, activeId)}>
                    {def.tables.map((table, ti) => (
                        <TemplateTable key={ti} def={table} v={v} flags={flags} />
                    ))}
                </div>
            ))}
        </>
    );
}

export function EffectiveSkillsSection({ children }: { children: ReactNode }) {
    return (
        <div className="combat-stats-section">
            <h4>有効スキル値（キャラクター値とこのジョブでのキャップの小さい方）</h4>
            <div id="equipEffectiveSkills" style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '6px', fontSize: '12px' }}>
                {children}
            </div>
        </div>
    );
}
