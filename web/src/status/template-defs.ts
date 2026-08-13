// プロパティセットテンプレートの宣言的定義 (旧 SubtabContents の固定 JSX を
// データ化したもの。描画は StatusTables.tsx の汎用レンダラが行う)。
//
// 値セルは compute.ts (StatusView.values) の id を参照し、描画時の DOM id にも
// そのまま使う (旧実装からの id 互換。smoke テストも #subtab-* / #stat* を参照)。
// 純データモジュールとして保つこと (他モジュールを import しない)。

/** 表示条件フラグ。SubtabContents が StatusView 由来の props から解決する */
export type TemplateFlag = 'rangedWs' | 'songString' | 'songWind' | 'geoHandbell';

export interface TemplateColumnDef {
    label: string;
    /** フラグが偽のとき列ごと非表示 (各行の該当セルも落ちる) */
    visibleIf?: TemplateFlag;
}

export interface TemplateRowDef {
    /** 行ヘッダ (メイン/サブ/レンジ)。いずれかの行にあるテーブルは先頭に空ヘッダ列が付く */
    label?: string;
    visibleIf?: TemplateFlag;
    /** 各列のセル: StatusView.values の id。null はダッシュ固定表示 */
    cells: (string | null)[];
}

export interface TemplateTableDef {
    /** colSpan 見出し行 (「属性耐性」など) */
    title?: string;
    visibleIf?: TemplateFlag;
    columns: TemplateColumnDef[];
    rows: TemplateRowDef[];
}

export const TEMPLATE_PROPSET_GROUPS = [
    '待機/防御/回避',
    'オートアタック/遠隔攻撃',
    'ウェポンスキル',
    '魔法',
] as const;

export interface TemplatePropsetDef {
    id: string;
    label: string;
    group: (typeof TEMPLATE_PROPSET_GROUPS)[number];
    tables: TemplateTableDef[];
}

/** 単一行テーブルの略記: [列ラベル, valueId] の組から生成 */
const simpleTable = (cells: [string, string][], title?: string): TemplateTableDef => ({
    ...(title !== undefined ? { title } : {}),
    columns: cells.map(([label]) => ({ label })),
    rows: [{ cells: cells.map(([, id]) => id) }],
});

// 魔法系 (標準形): 魔命テーブル + 魔攻/魔法ダメ (+MB 系) + 詠唱系 + INT/MND/CHR/MP。
// 呪歌・風水はスキル列の切替があるため個別定義。
// MB 系は MB の成立する種別のみ (神聖/精霊/暗黒/忍術/青)、
// 再詠唱間隔は装備が存在する種別のみ (精霊/青/呪歌/忍術)
const magicCastTable = (prefix: string, recastId?: string): TemplateTableDef =>
    simpleTable([
        ['コンサーブMP', `statMg${prefix}ConserveMp`],
        ['ファストキャスト', `statMg${prefix}FastCast`],
        ['クイックマジック', `statMg${prefix}QuickMagic`],
        ...(recastId !== undefined ? [['再詠唱間隔', recastId] as [string, string]] : []),
    ]);

const magicTemplate = (
    kind: string,
    prefix: string,
    name: string,
    opts: { mb?: boolean; recastId?: string } = {}
): TemplatePropsetDef => ({
    id: `subtab-magic-${kind}`,
    label: name,
    group: '魔法',
    tables: [
        simpleTable([
            [`${name}スキル`, `statMg${prefix}Skill`],
            ['魔命スキル', `statMg${prefix}MaccSkill`],
            ['魔命', `statMg${prefix}MaccTotal`],
        ]),
        simpleTable([
            ['魔攻', `statMg${prefix}Matk`],
            ['魔法ダメージ+', `statMg${prefix}Mdmg`],
            ...(opts.mb
                ? ([
                    ['MB.ボーナス', `statMg${prefix}MbBonus`],
                    ['MBダメージ', `statMg${prefix}MbDmg`],
                    ['MBダメージII', `statMg${prefix}MbDmg2`],
                ] as [string, string][])
                : []),
        ]),
        magicCastTable(prefix, opts.recastId),
        simpleTable([
            ['INT', `statMg${prefix}Int`],
            ['MND', `statMg${prefix}Mnd`],
            ['CHR', `statMg${prefix}Chr`],
            ['MP', `statMg${prefix}Mp`],
        ]),
    ],
});

export const TEMPLATE_PROPSET_DEFS: TemplatePropsetDef[] = [
    // ----- 1. 待機/回避/防御 -----
    {
        id: 'subtab-defense',
        label: '待機/回避/防御',
        group: '待機/防御/回避',
        tables: [
            simpleTable([
                ['リジェネ', 'statDefRegen'],
                ['リフレシュ', 'statDefRefresh'],
                ['リゲイン', 'statDefRegain'],
            ]),
            simpleTable([
                ['ファストキャスト', 'statDefFastCast'],
                ['クイックマジック', 'statDefQuickMagic'],
                ['スナップショット', 'statDefSnapshot'],
                ['ラピッドショット', 'statDefRapidShot'],
            ]),
            simpleTable([
                ['火', 'statDefResFire'],
                ['氷', 'statDefResIce'],
                ['風', 'statDefResWind'],
                ['土', 'statDefResEarth'],
                ['雷', 'statDefResLightning'],
                ['水', 'statDefResWater'],
                ['光', 'statDefResLight'],
                ['闇', 'statDefResDark'],
            ], '属性耐性'),
            simpleTable([
                ['睡眠', 'statDefResSleep'],
                ['麻痺', 'statDefResParalysis'],
                ['バインド', 'statDefResBind'],
                ['沈黙', 'statDefResSilence'],
                ['ヘヴィ', 'statDefResGravity'],
                ['スロウ', 'statDefResSlow'],
                ['石化', 'statDefResPetrification'],
                ['スタン', 'statDefResStun'],
            ], '状態異常レジスト'),
            simpleTable([
                ['毒', 'statDefResPoison'],
                ['魅了', 'statDefResCharm'],
                ['暗闇', 'statDefResBlind'],
                ['呪い', 'statDefResCurse'],
                ['病気', 'statDefResVirus'],
                ['アムネジア', 'statDefResAmnesia'],
                ['テラー', 'statDefResTerror'],
                ['デス', 'statDefResDeath'],
            ]),
        ],
    },

    // ----- 2. オートアタック (近接) -----
    {
        id: 'subtab-melee-auto',
        label: 'オートアタック',
        group: 'オートアタック/遠隔攻撃',
        tables: [
            {
                columns: [{ label: '武器スキル' }, { label: '攻撃' }, { label: '命中' }],
                rows: [
                    { label: 'メイン', cells: ['statAaMainSkill', 'statAaMainAtk', 'statAaMainAcc'] },
                    { label: 'サブ', cells: ['statAaSubSkill', 'statAaSubAtk', 'statAaSubAcc'] },
                ],
            },
            simpleTable([
                ['ストアTP', 'statAaStp'],
                ['モクシャ', 'statAaSb'],
                ['モクシャII', 'statAaSb2'],
                ['DA', 'statAaDa'],
                ['TA', 'statAaTa'],
                ['QA', 'statAaQa'],
            ]),
            simpleTable([
                ['DAダメージ', 'statAaDaDmg'],
                ['TAダメージ', 'statAaTaDmg'],
                ['クリティカル率', 'statAaCrit'],
                ['クリティカルダメージ', 'statAaCritDmg'],
                ['物理ダメージ上限', 'statAaPdl'],
            ]),
        ],
    },

    // ----- 3. 遠隔攻撃 -----
    {
        id: 'subtab-ranged-auto',
        label: '遠隔攻撃',
        group: 'オートアタック/遠隔攻撃',
        tables: [
            {
                visibleIf: 'rangedWs',
                columns: [{ label: '武器スキル' }, { label: '攻撃' }, { label: '命中' }],
                rows: [{ label: 'レンジ', cells: ['statRaSkill', 'statRaAtk', 'statRaAcc'] }],
            },
            simpleTable([
                ['ストアTP', 'statRaStp'],
                ['モクシャ', 'statRaSb'],
                ['モクシャII', 'statRaSb2'],
                ['ダブルショット', 'statRaDoubleShot'],
                ['トリプルショット', 'statRaTripleShot'],
            ]),
            simpleTable([
                ['ダブルショットダメージ', 'statRaDoubleShotDmg'],
                ['トリプルショットダメージ', 'statRaTripleShotDmg'],
                ['クリティカル率', 'statRaCrit'],
                ['クリティカルダメージ', 'statRaCritDmg'],
            ]),
            simpleTable([
                ['物理ダメージ上限', 'statRaPdl'],
                ['トゥルーショット', 'statRaTs'],
                ['リサイクル', 'statRaRecycle'],
            ]),
        ],
    },

    // ----- 4. 近接物理 WS -----
    {
        id: 'subtab-melee-ws',
        label: '近接物理WS',
        group: 'ウェポンスキル',
        tables: [
            {
                columns: [
                    { label: '武器スキル' }, { label: '魔命スキル' },
                    { label: '攻撃' }, { label: '命中' }, { label: '魔命' },
                ],
                rows: [
                    { label: 'メイン', cells: [
                        'statMwsSkill', 'statMwsMaccSkill',
                        'statMwsAtk', 'statMwsAcc', 'statMwsMaccTotal',
                    ] },
                    // 魔命合計はメイン/レンジ行のみ (サブ行はダッシュ固定)
                    { label: 'サブ', cells: [
                        'statMwsSubSkill', 'statMwsSubMaccSkill',
                        'statMwsSubAtk', 'statMwsSubAcc', null,
                    ] },
                ],
            },
            simpleTable([
                ['ストアTP', 'statMwsStp'],
                ['モクシャ', 'statMwsSb'],
                ['モクシャII', 'statMwsSb2'],
                ['DA', 'statMwsDa'],
                ['TA', 'statMwsTa'],
                ['QA', 'statMwsQa'],
            ]),
            simpleTable([
                ['クリティカル率', 'statMwsCrit'],
                ['クリティカルダメージ', 'statMwsCritDmg'],
                ['WSD', 'statMwsWsdmg'],
                ['TPボーナス', 'statMwsTpb'],
                ['連携ボーナス', 'statMwsScb'],
                ['物理ダメ上限', 'statMwsPdl'],
            ]),
        ],
    },

    // ----- 5. 遠隔物理 WS -----
    {
        id: 'subtab-ranged-ws',
        label: '遠隔物理WS',
        group: 'ウェポンスキル',
        tables: [
            // レンジ行 (弓術/射撃装備時のみ)。唯一の行のためテーブルごと切替
            {
                visibleIf: 'rangedWs',
                columns: [
                    { label: '武器スキル' }, { label: '魔命スキル' },
                    { label: '攻撃' }, { label: '命中' }, { label: '魔命' },
                ],
                rows: [{ label: 'レンジ', cells: [
                    'statRwsSkill', 'statRwsMaccSkill',
                    'statRwsAtk', 'statRwsAcc', 'statRwsMaccTotal',
                ] }],
            },
            simpleTable([
                ['ストアTP', 'statRwsStp'],
                ['モクシャ', 'statRwsSb'],
                ['モクシャII', 'statRwsSb2'],
                ['クリ率', 'statRwsCrit'],
                ['クリダメ', 'statRwsCritDmg'],
            ]),
            simpleTable([
                ['WSD', 'statRwsWsdmg'],
                ['TPボーナス', 'statRwsTpb'],
                ['連携ボーナス', 'statRwsScb'],
                ['物理ダメ上限', 'statRwsPdl'],
                ['トゥルーショット', 'statRwsTs'],
            ]),
        ],
    },

    // ----- 6. 属性 WS -----
    {
        id: 'subtab-elemental-ws',
        label: '属性WS',
        group: 'ウェポンスキル',
        tables: [
            {
                columns: [
                    { label: '武器スキル' }, { label: '魔命スキル' }, { label: '魔命' },
                ],
                rows: [
                    { label: 'メイン', cells: [
                        'statEwsMainSkill', 'statEwsMainMaccSkill', 'statEwsMainMaccTotal',
                    ] },
                    { label: 'レンジ', visibleIf: 'rangedWs', cells: [
                        'statEwsRangedSkill', null, 'statEwsRangedMaccTotal',
                    ] },
                ],
            },
            simpleTable([
                ['ストアTP', 'statEwsStp'],
                ['モクシャ', 'statEwsSb'],
                ['モクシャII', 'statEwsSb2'],
                ['WSD', 'statEwsWsdmg'],
                ['TPボーナス', 'statEwsTpb'],
                ['連携ボーナス', 'statEwsScb'],
            ]),
            simpleTable([
                ['魔攻', 'statEwsMatk'],
                ['魔法ダメージ+', 'statEwsMdmg'],
                ['アフィニティ', 'statEwsAff'],
                ['魔クリII', 'statEwsMcrit2'],
            ]),
        ],
    },

    // ----- 7. 近接属性物理 WS -----
    {
        id: 'subtab-melee-elemental-ws',
        label: '近接属性物理WS',
        group: 'ウェポンスキル',
        tables: [
            {
                columns: [
                    { label: '武器スキル' }, { label: '魔命スキル' },
                    { label: '攻撃' }, { label: '命中' }, { label: '魔命' },
                ],
                rows: [
                    { label: 'メイン', cells: [
                        'statMewsSkill', 'statMewsMaccSkill',
                        'statMewsAtk', 'statMewsAcc', 'statMewsMaccTotal',
                    ] },
                    { label: 'サブ', cells: [
                        'statMewsSubSkill', 'statMewsSubMaccSkill',
                        'statMewsSubAtk', 'statMewsSubAcc', null,
                    ] },
                ],
            },
            simpleTable([
                ['ストアTP', 'statMewsStp'],
                ['モクシャ', 'statMewsSb'],
                ['モクシャII', 'statMewsSb2'],
                ['DA', 'statMewsDa'],
                ['TA', 'statMewsTa'],
                ['QA', 'statMewsQa'],
            ]),
            simpleTable([
                ['WSD', 'statMewsWsdmg'],
                ['TPボーナス', 'statMewsTpb'],
                ['連携ボーナス', 'statMewsScb'],
                ['物理ダメ上限', 'statMewsPdl'],
            ]),
            simpleTable([
                ['魔攻', 'statMewsMatk'],
                ['魔法ダメージ+', 'statMewsMdmg'],
                ['アフィニティ', 'statMewsAff'],
                ['魔クリII', 'statMewsMcrit2'],
            ]),
        ],
    },

    // ----- 8. 遠隔属性物理 WS -----
    {
        id: 'subtab-ranged-elemental-ws',
        label: '遠隔属性物理WS',
        group: 'ウェポンスキル',
        tables: [
            {
                visibleIf: 'rangedWs',
                columns: [
                    { label: '武器スキル' }, { label: '魔命スキル' },
                    { label: '攻撃' }, { label: '命中' }, { label: '魔命' },
                ],
                rows: [{ label: 'レンジ', cells: [
                    'statRewsSkill', 'statRewsMaccSkill',
                    'statRewsAtk', 'statRewsAcc', 'statRewsMaccTotal',
                ] }],
            },
            simpleTable([
                ['ストアTP', 'statRewsStp'],
                ['モクシャ', 'statRewsSb'],
                ['モクシャII', 'statRewsSb2'],
                ['WSD', 'statRewsWsdmg'],
                ['TPボーナス', 'statRewsTpb'],
                ['連携ボーナス', 'statRewsScb'],
                ['物理ダメ上限', 'statRewsPdl'],
                ['トゥルーショット', 'statRewsTs'],
            ]),
            simpleTable([
                ['魔攻', 'statRewsMatk'],
                ['魔法ダメージ+', 'statRewsMdmg'],
                ['アフィニティ', 'statRewsAff'],
                ['魔クリII', 'statRewsMcrit2'],
            ]),
        ],
    },

    // ----- 9-19. 魔法 (11 種別) -----
    magicTemplate('divine', 'Divine', '神聖魔法', { mb: true }),
    magicTemplate('healing', 'Healing', '回復魔法'),
    magicTemplate('enhancing', 'Enhancing', '強化魔法'),
    magicTemplate('enfeebling', 'Enfeebling', '弱体魔法'),
    magicTemplate('elemental', 'Elemental', '精霊魔法', {
        mb: true, recastId: 'statMgElementalRecast',
    }),
    magicTemplate('dark', 'Dark', '暗黒魔法', { mb: true }),
    magicTemplate('summoning', 'Summoning', '召喚魔法'),
    magicTemplate('ninjutsu', 'Ninjutsu', '忍術', {
        mb: true, recastId: 'statMgNinjutsuRecast',
    }),
    // 呪歌: 楽器スキル列はレンジスロットの楽器種別で切替 (未装備時は歌唱のみ)
    {
        id: 'subtab-magic-song',
        label: '呪歌',
        group: '魔法',
        tables: [
            {
                columns: [
                    { label: '歌唱スキル' },
                    { label: '弦楽器スキル', visibleIf: 'songString' },
                    { label: '管楽器スキル', visibleIf: 'songWind' },
                    { label: '魔命スキル' },
                    { label: '魔命' },
                ],
                rows: [{ cells: [
                    'statMgSongSingingSkill', 'statMgSongStringSkill', 'statMgSongWindSkill',
                    'statMgSongMaccSkill', 'statMgSongMaccTotal',
                ] }],
            },
            simpleTable([
                ['魔攻', 'statMgSongMatk'],
                ['魔法ダメージ+', 'statMgSongMdmg'],
            ]),
            magicCastTable('Song', 'statMgSongRecast'),
            simpleTable([
                ['INT', 'statMgSongInt'],
                ['MND', 'statMgSongMnd'],
                ['CHR', 'statMgSongChr'],
                ['MP', 'statMgSongMp'],
            ]),
        ],
    },
    magicTemplate('blue', 'Blue', '青魔法', { mb: true, recastId: 'statMgBlueRecast' }),
    // 風水魔法: 風水鈴スキル列はレンジスロットに風水鈴装備時のみ表示
    {
        id: 'subtab-magic-geomancy',
        label: '風水魔法',
        group: '魔法',
        tables: [
            {
                columns: [
                    { label: '風水魔法スキル' },
                    { label: '風水鈴スキル', visibleIf: 'geoHandbell' },
                    { label: '魔命スキル' },
                    { label: '魔命' },
                ],
                rows: [{ cells: [
                    'statMgGeomancySkill', 'statMgGeomancyHandbellSkill',
                    'statMgGeomancyMaccSkill', 'statMgGeomancyMaccTotal',
                ] }],
            },
            simpleTable([
                ['魔攻', 'statMgGeomancyMatk'],
                ['魔法ダメージ+', 'statMgGeomancyMdmg'],
            ]),
            magicCastTable('Geomancy'),
            simpleTable([
                ['INT', 'statMgGeomancyInt'],
                ['MND', 'statMgGeomancyMnd'],
                ['CHR', 'statMgGeomancyChr'],
                ['MP', 'statMgGeomancyMp'],
            ]),
        ],
    },
];
