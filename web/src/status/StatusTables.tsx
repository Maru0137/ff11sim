// 装備編集タブのステータス表示テーブル群 (index.html から機械変換)。
// 値は v(id) 経由で参照する。id は旧実装の DOM id と同一に保っている。
import type { ReactNode } from 'react';

export type ValueGetter = (id: string) => string | number;

function subtabClass(id: string, activeId: string): string {
    return id === activeId ? 'status-subtab-content active' : 'status-subtab-content';
}

// プロパティセット選択 UI (StatusPanel / PropsetManageModal) の
// テンプレート分類。表示順 = この配列順 (docs/adr/0015)。
export const TEMPLATE_PROPSET_GROUPS = [
    '待機/防御/回避',
    'オートアタック/遠隔攻撃',
    'ウェポンスキル',
    '魔法',
] as const;

export const TEMPLATE_PROPSETS: { id: string; label: string; group: (typeof TEMPLATE_PROPSET_GROUPS)[number] }[] = [
    { id: 'subtab-defense', label: '待機/回避/防御', group: '待機/防御/回避' },
    { id: 'subtab-melee-auto', label: 'オートアタック', group: 'オートアタック/遠隔攻撃' },
    { id: 'subtab-ranged-auto', label: '遠隔攻撃', group: 'オートアタック/遠隔攻撃' },
    { id: 'subtab-melee-ws', label: '近接物理WS', group: 'ウェポンスキル' },
    { id: 'subtab-ranged-ws', label: '遠隔物理WS', group: 'ウェポンスキル' },
    { id: 'subtab-elemental-ws', label: '属性WS', group: 'ウェポンスキル' },
    { id: 'subtab-melee-elemental-ws', label: '近接属性物理WS', group: 'ウェポンスキル' },
    { id: 'subtab-ranged-elemental-ws', label: '遠隔属性物理WS', group: 'ウェポンスキル' },
    { id: 'subtab-magic-divine', label: '神聖魔法', group: '魔法' },
    { id: 'subtab-magic-healing', label: '回復魔法', group: '魔法' },
    { id: 'subtab-magic-enhancing', label: '強化魔法', group: '魔法' },
    { id: 'subtab-magic-enfeebling', label: '弱体魔法', group: '魔法' },
    { id: 'subtab-magic-elemental', label: '精霊魔法', group: '魔法' },
    { id: 'subtab-magic-dark', label: '暗黒魔法', group: '魔法' },
    { id: 'subtab-magic-summoning', label: '召喚魔法', group: '魔法' },
    { id: 'subtab-magic-ninjutsu', label: '忍術', group: '魔法' },
    { id: 'subtab-magic-song', label: '呪歌', group: '魔法' },
    { id: 'subtab-magic-blue', label: '青魔法', group: '魔法' },
    { id: 'subtab-magic-geomancy', label: '風水魔法', group: '魔法' },
];

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

export function SubtabContents({ v, activeId }: { v: ValueGetter; activeId: string }) {
    return (
        <>
            {/* 1. 待機/回避/防御 */}
            <div id="subtab-defense" className={subtabClass('subtab-defense', activeId)}>
                <table className="status-table">
                    <thead><tr><th>リジェネ</th><th>リフレシュ</th><th>リゲイン</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statDefRegen">{v('statDefRegen')}</td>
                        <td className="equip-val" id="statDefRefresh">{v('statDefRefresh')}</td>
                        <td className="equip-val" id="statDefRegain">{v('statDefRegain')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>ファストキャスト</th><th>クイックマジック</th><th>スナップショット</th><th>ラピッドショット</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statDefFastCast">{v('statDefFastCast')}</td>
                        <td className="equip-val" id="statDefQuickMagic">{v('statDefQuickMagic')}</td>
                        <td className="equip-val" id="statDefSnapshot">{v('statDefSnapshot')}</td>
                        <td className="equip-val" id="statDefRapidShot">{v('statDefRapidShot')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th colSpan={8}>属性耐性</th></tr><tr><th>火</th><th>氷</th><th>風</th><th>土</th><th>雷</th><th>水</th><th>光</th><th>闇</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statDefResFire">{v('statDefResFire')}</td>
                        <td className="equip-val" id="statDefResIce">{v('statDefResIce')}</td>
                        <td className="equip-val" id="statDefResWind">{v('statDefResWind')}</td>
                        <td className="equip-val" id="statDefResEarth">{v('statDefResEarth')}</td>
                        <td className="equip-val" id="statDefResLightning">{v('statDefResLightning')}</td>
                        <td className="equip-val" id="statDefResWater">{v('statDefResWater')}</td>
                        <td className="equip-val" id="statDefResLight">{v('statDefResLight')}</td>
                        <td className="equip-val" id="statDefResDark">{v('statDefResDark')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead>
                        <tr><th colSpan={8}>状態異常レジスト</th></tr>
                        <tr><th>睡眠</th><th>麻痺</th><th>バインド</th><th>沈黙</th><th>ヘヴィ</th><th>スロウ</th><th>石化</th><th>スタン</th></tr>
                    </thead>
                    <tbody><tr>
                        <td className="equip-val" id="statDefResSleep">{v('statDefResSleep')}</td>
                        <td className="equip-val" id="statDefResParalysis">{v('statDefResParalysis')}</td>
                        <td className="equip-val" id="statDefResBind">{v('statDefResBind')}</td>
                        <td className="equip-val" id="statDefResSilence">{v('statDefResSilence')}</td>
                        <td className="equip-val" id="statDefResGravity">{v('statDefResGravity')}</td>
                        <td className="equip-val" id="statDefResSlow">{v('statDefResSlow')}</td>
                        <td className="equip-val" id="statDefResPetrification">{v('statDefResPetrification')}</td>
                        <td className="equip-val" id="statDefResStun">{v('statDefResStun')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead>
                        <tr><th>毒</th><th>魅了</th><th>暗闇</th><th>呪い</th><th>病気</th><th>アムネジア</th><th>テラー</th><th>デス</th></tr>
                    </thead>
                    <tbody><tr>
                        <td className="equip-val" id="statDefResPoison">{v('statDefResPoison')}</td>
                        <td className="equip-val" id="statDefResCharm">{v('statDefResCharm')}</td>
                        <td className="equip-val" id="statDefResBlind">{v('statDefResBlind')}</td>
                        <td className="equip-val" id="statDefResCurse">{v('statDefResCurse')}</td>
                        <td className="equip-val" id="statDefResVirus">{v('statDefResVirus')}</td>
                        <td className="equip-val" id="statDefResAmnesia">{v('statDefResAmnesia')}</td>
                        <td className="equip-val" id="statDefResTerror">{v('statDefResTerror')}</td>
                        <td className="equip-val" id="statDefResDeath">{v('statDefResDeath')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 2. オートアタック (近接) */}
            <div id="subtab-melee-auto" className={subtabClass('subtab-melee-auto', activeId)}>
                <table className="status-table">
                    <thead><tr><th></th><th>武器スキル</th><th>攻撃</th><th>命中</th></tr></thead>
                    <tbody>
                        <tr>
                            <td>メイン</td>
                            <td className="equip-val" id="statAaMainSkill">{v('statAaMainSkill')}</td>
                            <td className="equip-val" id="statAaMainAtk">{v('statAaMainAtk')}</td>
                            <td className="equip-val" id="statAaMainAcc">{v('statAaMainAcc')}</td>
                        </tr>
                        <tr>
                            <td>サブ</td>
                            <td className="equip-val" id="statAaSubSkill">{v('statAaSubSkill')}</td>
                            <td className="equip-val" id="statAaSubAtk">{v('statAaSubAtk')}</td>
                            <td className="equip-val" id="statAaSubAcc">{v('statAaSubAcc')}</td>
                        </tr>
                    </tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>ストアTP</th><th>モクシャ</th><th>モクシャII</th><th>DA</th><th>TA</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statAaStp">{v('statAaStp')}</td>
                        <td className="equip-val" id="statAaSb">{v('statAaSb')}</td>
                        <td className="equip-val" id="statAaSb2">{v('statAaSb2')}</td>
                        <td className="equip-val" id="statAaDa">{v('statAaDa')}</td>
                        <td className="equip-val" id="statAaTa">{v('statAaTa')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>QA</th><th>DAダメージ</th><th>TAダメージ</th><th>クリティカル率</th><th>クリティカルダメージ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statAaQa">{v('statAaQa')}</td>
                        <td className="equip-val" id="statAaDaDmg">{v('statAaDaDmg')}</td>
                        <td className="equip-val" id="statAaTaDmg">{v('statAaTaDmg')}</td>
                        <td className="equip-val" id="statAaCrit">{v('statAaCrit')}</td>
                        <td className="equip-val" id="statAaCritDmg">{v('statAaCritDmg')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 3. 遠隔攻撃 */}
            <div id="subtab-ranged-auto" className={subtabClass('subtab-ranged-auto', activeId)}>
                <table className="status-table">
                    <thead><tr><th></th><th>武器スキル</th><th>飛攻</th><th>飛命</th></tr></thead>
                    <tbody><tr>
                        <td>レンジ</td>
                        <td className="equip-val" id="statRaSkill">{v('statRaSkill')}</td>
                        <td className="equip-val" id="statRaAtk">{v('statRaAtk')}</td>
                        <td className="equip-val" id="statRaAcc">{v('statRaAcc')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>STR</th><th>AGI</th><th>Store TP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statRaStr">{v('statRaStr')}</td>
                        <td className="equip-val" id="statRaAgi">{v('statRaAgi')}</td>
                        <td className="equip-val" id="statRaStp">{v('statRaStp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 4. 近接物理 WS */}
            <div id="subtab-melee-ws" className={subtabClass('subtab-melee-ws', activeId)}>
                <table className="status-table">
                    <thead><tr><th></th><th>武器スキル</th><th>攻撃</th><th>命中</th></tr></thead>
                    <tbody>
                        <tr>
                            <td>メイン</td>
                            <td className="equip-val" id="statMwsSkill">{v('statMwsSkill')}</td>
                            <td className="equip-val" id="statMwsAtk">{v('statMwsAtk')}</td>
                            <td className="equip-val" id="statMwsAcc">{v('statMwsAcc')}</td>
                        </tr>
                        <tr>
                            <td>サブ</td>
                            <td className="equip-val" id="statMwsSubSkill">{v('statMwsSubSkill')}</td>
                            <td className="equip-val" id="statMwsSubAtk">{v('statMwsSubAtk')}</td>
                            <td className="equip-val" id="statMwsSubAcc">{v('statMwsSubAcc')}</td>
                        </tr>
                    </tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>ストアTP</th><th>モクシャ</th><th>モクシャII</th><th>DA</th><th>TA</th><th>QA</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMwsStp">{v('statMwsStp')}</td>
                        <td className="equip-val" id="statMwsSb">{v('statMwsSb')}</td>
                        <td className="equip-val" id="statMwsSb2">{v('statMwsSb2')}</td>
                        <td className="equip-val" id="statMwsDa">{v('statMwsDa')}</td>
                        <td className="equip-val" id="statMwsTa">{v('statMwsTa')}</td>
                        <td className="equip-val" id="statMwsQa">{v('statMwsQa')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>クリ率</th><th>クリダメ</th><th>WSダメ</th><th>TPボーナス</th><th>連携ボーナス</th><th>物理ダメ上限</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMwsCrit">{v('statMwsCrit')}</td>
                        <td className="equip-val" id="statMwsCritDmg">{v('statMwsCritDmg')}</td>
                        <td className="equip-val" id="statMwsWsdmg">{v('statMwsWsdmg')}</td>
                        <td className="equip-val" id="statMwsTpb">{v('statMwsTpb')}</td>
                        <td className="equip-val" id="statMwsScb">{v('statMwsScb')}</td>
                        <td className="equip-val" id="statMwsPdl">{v('statMwsPdl')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 5. 遠隔物理 WS */}
            <div id="subtab-ranged-ws" className={subtabClass('subtab-ranged-ws', activeId)}>
                <table className="status-table">
                    <thead><tr><th></th><th>武器スキル</th><th>飛攻</th><th>飛命</th></tr></thead>
                    <tbody><tr>
                        <td>レンジ</td>
                        <td className="equip-val" id="statRwsSkill">{v('statRwsSkill')}</td>
                        <td className="equip-val" id="statRwsAtk">{v('statRwsAtk')}</td>
                        <td className="equip-val" id="statRwsAcc">{v('statRwsAcc')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>ストアTP</th><th>モクシャ</th><th>モクシャII</th><th>クリ率</th><th>クリダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statRwsStp">{v('statRwsStp')}</td>
                        <td className="equip-val" id="statRwsSb">{v('statRwsSb')}</td>
                        <td className="equip-val" id="statRwsSb2">{v('statRwsSb2')}</td>
                        <td className="equip-val" id="statRwsCrit">{v('statRwsCrit')}</td>
                        <td className="equip-val" id="statRwsCritDmg">{v('statRwsCritDmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>WSダメ</th><th>TPボーナス</th><th>連携ボーナス</th><th>物理ダメ上限</th><th>トゥルーショット</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statRwsWsdmg">{v('statRwsWsdmg')}</td>
                        <td className="equip-val" id="statRwsTpb">{v('statRwsTpb')}</td>
                        <td className="equip-val" id="statRwsScb">{v('statRwsScb')}</td>
                        <td className="equip-val" id="statRwsPdl">{v('statRwsPdl')}</td>
                        <td className="equip-val" id="statRwsTs">{v('statRwsTs')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 6. 属性 WS */}
            <div id="subtab-elemental-ws" className={subtabClass('subtab-elemental-ws', activeId)}>
                <table className="status-table">
                    <thead><tr><th></th><th>武器スキル</th><th>魔命スキル</th><th>魔命合計</th></tr></thead>
                    <tbody>
                        <tr>
                            <td>メイン</td>
                            <td className="equip-val" id="statEwsMainSkill">{v('statEwsMainSkill')}</td>
                            <td className="equip-val" id="statEwsMainMaccSkill">{v('statEwsMainMaccSkill')}</td>
                            <td className="equip-val" id="statEwsMainMaccTotal">{v('statEwsMainMaccTotal')}</td>
                        </tr>
                        <tr>
                            <td>レンジ</td>
                            <td className="equip-val" id="statEwsRangedSkill">{v('statEwsRangedSkill')}</td>
                            <td className="equip-val">-</td>
                            <td className="equip-val" id="statEwsRangedMaccTotal">{v('statEwsRangedMaccTotal')}</td>
                        </tr>
                    </tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>魔攻</th><th>魔命</th><th>ストアTP</th><th>モクシャ</th><th>モクシャII</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statEwsMatk">{v('statEwsMatk')}</td>
                        <td className="equip-val" id="statEwsMacc">{v('statEwsMacc')}</td>
                        <td className="equip-val" id="statEwsStp">{v('statEwsStp')}</td>
                        <td className="equip-val" id="statEwsSb">{v('statEwsSb')}</td>
                        <td className="equip-val" id="statEwsSb2">{v('statEwsSb2')}</td>
                        <td className="equip-val" id="statEwsMdmg">{v('statEwsMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>アフィニティ</th><th>魔クリII</th><th>WSダメ</th><th>TPボーナス</th><th>連携ボーナス</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statEwsAff">{v('statEwsAff')}</td>
                        <td className="equip-val" id="statEwsMcrit2">{v('statEwsMcrit2')}</td>
                        <td className="equip-val" id="statEwsWsdmg">{v('statEwsWsdmg')}</td>
                        <td className="equip-val" id="statEwsTpb">{v('statEwsTpb')}</td>
                        <td className="equip-val" id="statEwsScb">{v('statEwsScb')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 7. 近接属性物理 WS */}
            <div id="subtab-melee-elemental-ws" className={subtabClass('subtab-melee-elemental-ws', activeId)}>
                <table className="status-table">
                    <thead><tr><th></th><th>武器スキル</th><th>魔命スキル</th><th>攻撃</th><th>命中</th><th>魔命合計</th></tr></thead>
                    <tbody>
                        <tr>
                            <td>メイン</td>
                            <td className="equip-val" id="statMewsSkill">{v('statMewsSkill')}</td>
                            <td className="equip-val" id="statMewsMaccSkill">{v('statMewsMaccSkill')}</td>
                            <td className="equip-val" id="statMewsAtk">{v('statMewsAtk')}</td>
                            <td className="equip-val" id="statMewsAcc">{v('statMewsAcc')}</td>
                            <td className="equip-val" id="statMewsMaccTotal">{v('statMewsMaccTotal')}</td>
                        </tr>
                        <tr>
                            <td>サブ</td>
                            <td className="equip-val" id="statMewsSubSkill">{v('statMewsSubSkill')}</td>
                            <td className="equip-val" id="statMewsSubMaccSkill">{v('statMewsSubMaccSkill')}</td>
                            <td className="equip-val" id="statMewsSubAtk">{v('statMewsSubAtk')}</td>
                            <td className="equip-val" id="statMewsSubAcc">{v('statMewsSubAcc')}</td>
                            <td className="equip-val" id="statMewsSubMaccTotal">{v('statMewsSubMaccTotal')}</td>
                        </tr>
                    </tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>ストアTP</th><th>モクシャ</th><th>モクシャII</th><th>DA</th><th>TA</th><th>QA</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMewsStp">{v('statMewsStp')}</td>
                        <td className="equip-val" id="statMewsSb">{v('statMewsSb')}</td>
                        <td className="equip-val" id="statMewsSb2">{v('statMewsSb2')}</td>
                        <td className="equip-val" id="statMewsDa">{v('statMewsDa')}</td>
                        <td className="equip-val" id="statMewsTa">{v('statMewsTa')}</td>
                        <td className="equip-val" id="statMewsQa">{v('statMewsQa')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>クリ率</th><th>クリダメ</th><th>WSダメ</th><th>TPボーナス</th><th>連携ボーナス</th><th>物理ダメ上限</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMewsCrit">{v('statMewsCrit')}</td>
                        <td className="equip-val" id="statMewsCritDmg">{v('statMewsCritDmg')}</td>
                        <td className="equip-val" id="statMewsWsdmg">{v('statMewsWsdmg')}</td>
                        <td className="equip-val" id="statMewsTpb">{v('statMewsTpb')}</td>
                        <td className="equip-val" id="statMewsScb">{v('statMewsScb')}</td>
                        <td className="equip-val" id="statMewsPdl">{v('statMewsPdl')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>魔攻</th><th>魔命</th><th>魔法ダメ</th><th>アフィニティ</th><th>魔クリII</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMewsMatk">{v('statMewsMatk')}</td>
                        <td className="equip-val" id="statMewsMacc">{v('statMewsMacc')}</td>
                        <td className="equip-val" id="statMewsMdmg">{v('statMewsMdmg')}</td>
                        <td className="equip-val" id="statMewsAff">{v('statMewsAff')}</td>
                        <td className="equip-val" id="statMewsMcrit2">{v('statMewsMcrit2')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 8. 遠隔属性物理 WS */}
            <div id="subtab-ranged-elemental-ws" className={subtabClass('subtab-ranged-elemental-ws', activeId)}>
                <table className="status-table">
                    <thead><tr><th></th><th>武器スキル</th><th>魔命スキル</th><th>飛攻</th><th>飛命</th><th>魔命合計</th></tr></thead>
                    <tbody><tr>
                        <td>レンジ</td>
                        <td className="equip-val" id="statRewsSkill">{v('statRewsSkill')}</td>
                        <td className="equip-val" id="statRewsMaccSkill">{v('statRewsMaccSkill')}</td>
                        <td className="equip-val" id="statRewsAtk">{v('statRewsAtk')}</td>
                        <td className="equip-val" id="statRewsAcc">{v('statRewsAcc')}</td>
                        <td className="equip-val" id="statRewsMaccTotal">{v('statRewsMaccTotal')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>ストアTP</th><th>モクシャ</th><th>モクシャII</th><th>クリ率</th><th>クリダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statRewsStp">{v('statRewsStp')}</td>
                        <td className="equip-val" id="statRewsSb">{v('statRewsSb')}</td>
                        <td className="equip-val" id="statRewsSb2">{v('statRewsSb2')}</td>
                        <td className="equip-val" id="statRewsCrit">{v('statRewsCrit')}</td>
                        <td className="equip-val" id="statRewsCritDmg">{v('statRewsCritDmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>WSダメ</th><th>TPボーナス</th><th>連携ボーナス</th><th>物理ダメ上限</th><th>トゥルーショット</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statRewsWsdmg">{v('statRewsWsdmg')}</td>
                        <td className="equip-val" id="statRewsTpb">{v('statRewsTpb')}</td>
                        <td className="equip-val" id="statRewsScb">{v('statRewsScb')}</td>
                        <td className="equip-val" id="statRewsPdl">{v('statRewsPdl')}</td>
                        <td className="equip-val" id="statRewsTs">{v('statRewsTs')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>魔攻</th><th>魔命</th><th>魔法ダメ</th><th>アフィニティ</th><th>魔クリII</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statRewsMatk">{v('statRewsMatk')}</td>
                        <td className="equip-val" id="statRewsMacc">{v('statRewsMacc')}</td>
                        <td className="equip-val" id="statRewsMdmg">{v('statRewsMdmg')}</td>
                        <td className="equip-val" id="statRewsAff">{v('statRewsAff')}</td>
                        <td className="equip-val" id="statRewsMcrit2">{v('statRewsMcrit2')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 9. 神聖魔法 */}
            <div id="subtab-magic-divine" className={subtabClass('subtab-magic-divine', activeId)}>
                <table className="status-table">
                    <thead><tr><th>神聖魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgDivineSkill">{v('statMgDivineSkill')}</td>
                        <td className="equip-val" id="statMgDivineMatk">{v('statMgDivineMatk')}</td>
                        <td className="equip-val" id="statMgDivineMacc">{v('statMgDivineMacc')}</td>
                        <td className="equip-val" id="statMgDivineMeva">{v('statMgDivineMeva')}</td>
                        <td className="equip-val" id="statMgDivineMdmg">{v('statMgDivineMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgDivineInt">{v('statMgDivineInt')}</td>
                        <td className="equip-val" id="statMgDivineMnd">{v('statMgDivineMnd')}</td>
                        <td className="equip-val" id="statMgDivineChr">{v('statMgDivineChr')}</td>
                        <td className="equip-val" id="statMgDivineMp">{v('statMgDivineMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 10. 回復魔法 */}
            <div id="subtab-magic-healing" className={subtabClass('subtab-magic-healing', activeId)}>
                <table className="status-table">
                    <thead><tr><th>回復魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgHealingSkill">{v('statMgHealingSkill')}</td>
                        <td className="equip-val" id="statMgHealingMatk">{v('statMgHealingMatk')}</td>
                        <td className="equip-val" id="statMgHealingMacc">{v('statMgHealingMacc')}</td>
                        <td className="equip-val" id="statMgHealingMeva">{v('statMgHealingMeva')}</td>
                        <td className="equip-val" id="statMgHealingMdmg">{v('statMgHealingMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgHealingInt">{v('statMgHealingInt')}</td>
                        <td className="equip-val" id="statMgHealingMnd">{v('statMgHealingMnd')}</td>
                        <td className="equip-val" id="statMgHealingChr">{v('statMgHealingChr')}</td>
                        <td className="equip-val" id="statMgHealingMp">{v('statMgHealingMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 11. 強化魔法 */}
            <div id="subtab-magic-enhancing" className={subtabClass('subtab-magic-enhancing', activeId)}>
                <table className="status-table">
                    <thead><tr><th>強化魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgEnhancingSkill">{v('statMgEnhancingSkill')}</td>
                        <td className="equip-val" id="statMgEnhancingMatk">{v('statMgEnhancingMatk')}</td>
                        <td className="equip-val" id="statMgEnhancingMacc">{v('statMgEnhancingMacc')}</td>
                        <td className="equip-val" id="statMgEnhancingMeva">{v('statMgEnhancingMeva')}</td>
                        <td className="equip-val" id="statMgEnhancingMdmg">{v('statMgEnhancingMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgEnhancingInt">{v('statMgEnhancingInt')}</td>
                        <td className="equip-val" id="statMgEnhancingMnd">{v('statMgEnhancingMnd')}</td>
                        <td className="equip-val" id="statMgEnhancingChr">{v('statMgEnhancingChr')}</td>
                        <td className="equip-val" id="statMgEnhancingMp">{v('statMgEnhancingMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 12. 弱体魔法 */}
            <div id="subtab-magic-enfeebling" className={subtabClass('subtab-magic-enfeebling', activeId)}>
                <table className="status-table">
                    <thead><tr><th>弱体魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgEnfeeblingSkill">{v('statMgEnfeeblingSkill')}</td>
                        <td className="equip-val" id="statMgEnfeeblingMatk">{v('statMgEnfeeblingMatk')}</td>
                        <td className="equip-val" id="statMgEnfeeblingMacc">{v('statMgEnfeeblingMacc')}</td>
                        <td className="equip-val" id="statMgEnfeeblingMeva">{v('statMgEnfeeblingMeva')}</td>
                        <td className="equip-val" id="statMgEnfeeblingMdmg">{v('statMgEnfeeblingMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgEnfeeblingInt">{v('statMgEnfeeblingInt')}</td>
                        <td className="equip-val" id="statMgEnfeeblingMnd">{v('statMgEnfeeblingMnd')}</td>
                        <td className="equip-val" id="statMgEnfeeblingChr">{v('statMgEnfeeblingChr')}</td>
                        <td className="equip-val" id="statMgEnfeeblingMp">{v('statMgEnfeeblingMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 13. 精霊魔法 */}
            <div id="subtab-magic-elemental" className={subtabClass('subtab-magic-elemental', activeId)}>
                <table className="status-table">
                    <thead><tr><th>精霊魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgElementalSkill">{v('statMgElementalSkill')}</td>
                        <td className="equip-val" id="statMgElementalMatk">{v('statMgElementalMatk')}</td>
                        <td className="equip-val" id="statMgElementalMacc">{v('statMgElementalMacc')}</td>
                        <td className="equip-val" id="statMgElementalMeva">{v('statMgElementalMeva')}</td>
                        <td className="equip-val" id="statMgElementalMdmg">{v('statMgElementalMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgElementalInt">{v('statMgElementalInt')}</td>
                        <td className="equip-val" id="statMgElementalMnd">{v('statMgElementalMnd')}</td>
                        <td className="equip-val" id="statMgElementalChr">{v('statMgElementalChr')}</td>
                        <td className="equip-val" id="statMgElementalMp">{v('statMgElementalMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 14. 暗黒魔法 */}
            <div id="subtab-magic-dark" className={subtabClass('subtab-magic-dark', activeId)}>
                <table className="status-table">
                    <thead><tr><th>暗黒魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgDarkSkill">{v('statMgDarkSkill')}</td>
                        <td className="equip-val" id="statMgDarkMatk">{v('statMgDarkMatk')}</td>
                        <td className="equip-val" id="statMgDarkMacc">{v('statMgDarkMacc')}</td>
                        <td className="equip-val" id="statMgDarkMeva">{v('statMgDarkMeva')}</td>
                        <td className="equip-val" id="statMgDarkMdmg">{v('statMgDarkMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgDarkInt">{v('statMgDarkInt')}</td>
                        <td className="equip-val" id="statMgDarkMnd">{v('statMgDarkMnd')}</td>
                        <td className="equip-val" id="statMgDarkChr">{v('statMgDarkChr')}</td>
                        <td className="equip-val" id="statMgDarkMp">{v('statMgDarkMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 15. 召喚魔法 */}
            <div id="subtab-magic-summoning" className={subtabClass('subtab-magic-summoning', activeId)}>
                <table className="status-table">
                    <thead><tr><th>召喚魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgSummoningSkill">{v('statMgSummoningSkill')}</td>
                        <td className="equip-val" id="statMgSummoningMatk">{v('statMgSummoningMatk')}</td>
                        <td className="equip-val" id="statMgSummoningMacc">{v('statMgSummoningMacc')}</td>
                        <td className="equip-val" id="statMgSummoningMeva">{v('statMgSummoningMeva')}</td>
                        <td className="equip-val" id="statMgSummoningMdmg">{v('statMgSummoningMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgSummoningInt">{v('statMgSummoningInt')}</td>
                        <td className="equip-val" id="statMgSummoningMnd">{v('statMgSummoningMnd')}</td>
                        <td className="equip-val" id="statMgSummoningChr">{v('statMgSummoningChr')}</td>
                        <td className="equip-val" id="statMgSummoningMp">{v('statMgSummoningMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 16. 忍術 */}
            <div id="subtab-magic-ninjutsu" className={subtabClass('subtab-magic-ninjutsu', activeId)}>
                <table className="status-table">
                    <thead><tr><th>忍術スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgNinjutsuSkill">{v('statMgNinjutsuSkill')}</td>
                        <td className="equip-val" id="statMgNinjutsuMatk">{v('statMgNinjutsuMatk')}</td>
                        <td className="equip-val" id="statMgNinjutsuMacc">{v('statMgNinjutsuMacc')}</td>
                        <td className="equip-val" id="statMgNinjutsuMeva">{v('statMgNinjutsuMeva')}</td>
                        <td className="equip-val" id="statMgNinjutsuMdmg">{v('statMgNinjutsuMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgNinjutsuInt">{v('statMgNinjutsuInt')}</td>
                        <td className="equip-val" id="statMgNinjutsuMnd">{v('statMgNinjutsuMnd')}</td>
                        <td className="equip-val" id="statMgNinjutsuChr">{v('statMgNinjutsuChr')}</td>
                        <td className="equip-val" id="statMgNinjutsuMp">{v('statMgNinjutsuMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 17. 呪歌 */}
            <div id="subtab-magic-song" className={subtabClass('subtab-magic-song', activeId)}>
                <table className="status-table">
                    <thead><tr><th>歌唱スキル</th><th>弦楽器スキル</th><th>管楽器スキル</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgSongSingingSkill">{v('statMgSongSingingSkill')}</td>
                        <td className="equip-val" id="statMgSongStringSkill">{v('statMgSongStringSkill')}</td>
                        <td className="equip-val" id="statMgSongWindSkill">{v('statMgSongWindSkill')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgSongMatk">{v('statMgSongMatk')}</td>
                        <td className="equip-val" id="statMgSongMacc">{v('statMgSongMacc')}</td>
                        <td className="equip-val" id="statMgSongMeva">{v('statMgSongMeva')}</td>
                        <td className="equip-val" id="statMgSongMdmg">{v('statMgSongMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgSongInt">{v('statMgSongInt')}</td>
                        <td className="equip-val" id="statMgSongMnd">{v('statMgSongMnd')}</td>
                        <td className="equip-val" id="statMgSongChr">{v('statMgSongChr')}</td>
                        <td className="equip-val" id="statMgSongMp">{v('statMgSongMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 18. 青魔法 */}
            <div id="subtab-magic-blue" className={subtabClass('subtab-magic-blue', activeId)}>
                <table className="status-table">
                    <thead><tr><th>青魔法スキル</th><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgBlueSkill">{v('statMgBlueSkill')}</td>
                        <td className="equip-val" id="statMgBlueMatk">{v('statMgBlueMatk')}</td>
                        <td className="equip-val" id="statMgBlueMacc">{v('statMgBlueMacc')}</td>
                        <td className="equip-val" id="statMgBlueMeva">{v('statMgBlueMeva')}</td>
                        <td className="equip-val" id="statMgBlueMdmg">{v('statMgBlueMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgBlueInt">{v('statMgBlueInt')}</td>
                        <td className="equip-val" id="statMgBlueMnd">{v('statMgBlueMnd')}</td>
                        <td className="equip-val" id="statMgBlueChr">{v('statMgBlueChr')}</td>
                        <td className="equip-val" id="statMgBlueMp">{v('statMgBlueMp')}</td>
                    </tr></tbody>
                </table>
            </div>

            {/* 19. 風水魔法 */}
            <div id="subtab-magic-geomancy" className={subtabClass('subtab-magic-geomancy', activeId)}>
                <table className="status-table">
                    <thead><tr><th>風水魔法スキル</th><th>風水鈴スキル</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgGeomancySkill">{v('statMgGeomancySkill')}</td>
                        <td className="equip-val" id="statMgGeomancyHandbellSkill">{v('statMgGeomancyHandbellSkill')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>魔攻</th><th>魔命</th><th>魔回避</th><th>魔法ダメ</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgGeomancyMatk">{v('statMgGeomancyMatk')}</td>
                        <td className="equip-val" id="statMgGeomancyMacc">{v('statMgGeomancyMacc')}</td>
                        <td className="equip-val" id="statMgGeomancyMeva">{v('statMgGeomancyMeva')}</td>
                        <td className="equip-val" id="statMgGeomancyMdmg">{v('statMgGeomancyMdmg')}</td>
                    </tr></tbody>
                </table>
                <table className="status-table">
                    <thead><tr><th>INT</th><th>MND</th><th>CHR</th><th>MP</th></tr></thead>
                    <tbody><tr>
                        <td className="equip-val" id="statMgGeomancyInt">{v('statMgGeomancyInt')}</td>
                        <td className="equip-val" id="statMgGeomancyMnd">{v('statMgGeomancyMnd')}</td>
                        <td className="equip-val" id="statMgGeomancyChr">{v('statMgGeomancyChr')}</td>
                        <td className="equip-val" id="statMgGeomancyMp">{v('statMgGeomancyMp')}</td>
                    </tr></tbody>
                </table>
            </div>
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
