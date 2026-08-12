// 組み込みプロパティ項目カタログ (docs/adr/0015)。
// 値はリゾルバ関数で {equip, totalStats, derived} から算出する。
// StatusView.values の DOM id を参照しないのは、同一プロパティが
// サブタブごとに別 id (statAaStp / statMwsStp ...) で重複しており、
// テンプレート描画の内部実装に結合してしまうため。
//
// 収録範囲: 基本 9 ステ (HP〜CHR) と左の常時表示テーブル既出の項目
// (防御力/回避/魔防/魔回避/ヘイスト/被ダメージ系) は含めない。
import { numOrDash, pctOrDash, fmtPct, formatWeaponSkill } from '../status/format';

// equip/totalStats は WASM 由来で実質 any (web/src/wasm.ts の型付け方針参照)
export interface PropertyValueContext {
    /** calculateEquipSetBonuses の結果 (装備抽出値のみ) */
    equip: any;
    /** calculate_status_from_profile(profile, ..., bonusStats) の結果 */
    totalStats: any;
    /** compute.ts が算出済みの合成値 */
    derived: {
        magicAttackTotal: number;
        magicAccuracyTotal: number;
        magicDamageTotal: number;
        wsDamagePct: number;
        skillchainBonusTotal: number;
        /** combineStatusResist の結果 (テナシティ込み) */
        statusResists: Record<string, number>;
    };
}

/** 内訳モーダル (docs/adr/0016) 用のソース分解メタ */
export interface PropertyBreakdownMeta {
    /** 装備スロット行: per_slot_stats (extract_all_stats) のキー */
    equipKey?: string;
    /** キャラ由来行: calculate_status_breakdown の rows 列キー */
    charKey?: string;
}

export interface BuiltinPropertyItem {
    id: string;
    label: string;
    category: string;
    resolve: (ctx: PropertyValueContext) => string | number;
    /**
     * 内訳の分解方法。undefined = 内訳非対応 (武器スキルのような文字列値)。
     * resolve が参照する値と同じ定義になるように保つこと
     * (equipKey は resolve の c.equip.* と、charKey は totalStats 側の
     *  キャラ寄与と対応する。齟齬があると内訳の合計がパネル表示と食い違う)。
     */
    breakdown?: PropertyBreakdownMeta;
}

export const PROPERTY_CATEGORIES = [
    '近接',
    '遠隔',
    'WS',
    '魔法',
    '待機/回復',
    '武器スキル',
    '属性耐性',
    '状態異常レジスト',
] as const;

const ELEMENT_RESISTS: [string, string][] = [
    ['fire', '火'], ['ice', '氷'], ['wind', '風'], ['earth', '土'],
    ['lightning', '雷'], ['water', '水'], ['light', '光'], ['dark', '闇'],
];

const STATUS_RESISTS: [string, string][] = [
    ['sleep', '睡眠'], ['paralysis', '麻痺'], ['bind', 'バインド'], ['silence', '沈黙'],
    ['gravity', 'ヘヴィ'], ['slow', 'スロウ'], ['petrification', '石化'], ['stun', 'スタン'],
    ['poison', '毒'], ['charm', '魅了'], ['blind', '暗闇'], ['curse', '呪い'],
    ['virus', '病気'], ['amnesia', 'アムネジア'], ['terror', 'テラー'], ['death', 'デス'],
];

export const BUILTIN_PROPERTY_ITEMS: BuiltinPropertyItem[] = [
    // ----- 近接 -----
    { id: 'attack', label: '攻撃', category: '近接',
      resolve: (c) => numOrDash(c.totalStats.main_attack),
      breakdown: { equipKey: 'attack', charKey: 'attack' } },
    { id: 'accuracy', label: '命中', category: '近接',
      resolve: (c) => numOrDash(c.totalStats.main_accuracy),
      breakdown: { equipKey: 'accuracy', charKey: 'accuracy' } },
    { id: 'store_tp', label: 'ストアTP', category: '近接',
      resolve: (c) => numOrDash(c.totalStats.store_tp),
      breakdown: { equipKey: 'store_tp', charKey: 'store_tp' } },
    { id: 'subtle_blow', label: 'モクシャ', category: '近接',
      resolve: (c) => numOrDash(c.totalStats.subtle_blow),
      breakdown: { equipKey: 'subtle_blow', charKey: 'subtle_blow' } },
    { id: 'subtle_blow_2', label: 'モクシャII', category: '近接',
      resolve: (c) => numOrDash(c.equip.subtle_blow_2),
      breakdown: { equipKey: 'subtle_blow_2' } },
    { id: 'double_attack', label: 'DA', category: '近接',
      resolve: (c) => pctOrDash(c.totalStats.double_attack_pct),
      breakdown: { equipKey: 'double_attack_pct', charKey: 'double_attack' } },
    { id: 'triple_attack', label: 'TA', category: '近接',
      resolve: (c) => pctOrDash(c.totalStats.triple_attack_pct),
      breakdown: { equipKey: 'triple_attack_pct', charKey: 'triple_attack' } },
    { id: 'quad_attack', label: 'QA', category: '近接',
      resolve: (c) => pctOrDash(c.equip.quad_attack_pct),
      breakdown: { equipKey: 'quad_attack_pct' } },
    { id: 'da_damage', label: 'DAダメージ', category: '近接',
      resolve: (c) => pctOrDash(c.equip.double_attack_damage_pct),
      breakdown: { equipKey: 'double_attack_damage_pct' } },
    { id: 'ta_damage', label: 'TAダメージ', category: '近接',
      resolve: (c) => pctOrDash(c.equip.triple_attack_damage_pct),
      breakdown: { equipKey: 'triple_attack_damage_pct' } },
    { id: 'crit_rate', label: 'クリティカル率', category: '近接',
      resolve: (c) => pctOrDash(c.equip.critical_hit_rate_pct),
      breakdown: { equipKey: 'critical_hit_rate_pct' } },
    { id: 'crit_damage', label: 'クリティカルダメージ', category: '近接',
      resolve: (c) => pctOrDash(c.equip.critical_hit_damage_pct),
      breakdown: { equipKey: 'critical_hit_damage_pct' } },

    // ----- 遠隔 -----
    { id: 'ranged_attack', label: '飛攻', category: '遠隔',
      resolve: (c) => numOrDash(c.totalStats.ranged_attack),
      breakdown: { equipKey: 'ranged_attack', charKey: 'ranged_attack' } },
    { id: 'ranged_accuracy', label: '飛命', category: '遠隔',
      resolve: (c) => numOrDash(c.totalStats.ranged_accuracy),
      breakdown: { equipKey: 'ranged_accuracy', charKey: 'ranged_accuracy' } },
    // Snapshot/Rapid Shot は装備テキストでも単位無し表記が標準 (compute.ts と同じ)
    { id: 'snapshot', label: 'スナップショット', category: '遠隔',
      resolve: (c) => numOrDash(c.equip.snapshot_pct),
      breakdown: { equipKey: 'snapshot_pct' } },
    { id: 'rapid_shot', label: 'ラピッドショット', category: '遠隔',
      resolve: (c) => numOrDash(c.totalStats.rapid_shot_pct),
      breakdown: { equipKey: 'rapid_shot_pct', charKey: 'rapid_shot' } },
    { id: 'true_shot', label: 'トゥルーショット', category: '遠隔',
      resolve: (c) => numOrDash(c.equip.true_shot),
      breakdown: { equipKey: 'true_shot' } },

    // ----- WS -----
    { id: 'ws_damage', label: 'WSダメージ', category: 'WS',
      resolve: (c) => pctOrDash(c.derived.wsDamagePct),
      breakdown: { equipKey: 'weapon_skill_damage_pct' } },
    { id: 'tp_bonus', label: 'TPボーナス', category: 'WS',
      resolve: (c) => numOrDash(c.equip.tp_bonus),
      breakdown: { equipKey: 'tp_bonus' } },
    { id: 'skillchain_bonus', label: '連携ボーナス', category: 'WS',
      resolve: (c) => numOrDash(c.derived.skillchainBonusTotal),
      breakdown: { equipKey: 'skillchain_bonus', charKey: 'skillchain_bonus' } },
    { id: 'pdl', label: '物理ダメージ上限', category: 'WS',
      resolve: (c) => pctOrDash(c.equip.physical_damage_limit_pct),
      breakdown: { equipKey: 'physical_damage_limit_pct' } },

    // ----- 魔法 -----
    { id: 'magic_attack', label: '魔攻', category: '魔法',
      resolve: (c) => numOrDash(c.derived.magicAttackTotal),
      breakdown: { equipKey: 'magic_attack', charKey: 'magic_attack' } },
    { id: 'magic_accuracy', label: '魔命', category: '魔法',
      resolve: (c) => numOrDash(c.derived.magicAccuracyTotal),
      breakdown: { equipKey: 'magic_accuracy', charKey: 'magic_accuracy' } },
    { id: 'magic_damage', label: '魔法ダメージ', category: '魔法',
      resolve: (c) => numOrDash(c.derived.magicDamageTotal),
      breakdown: { equipKey: 'magic_damage' } },
    { id: 'magic_affinity', label: 'アフィニティ', category: '魔法',
      resolve: (c) => numOrDash(c.equip.magic_affinity),
      breakdown: { equipKey: 'magic_affinity' } },
    { id: 'magic_crit_2', label: '魔クリII', category: '魔法',
      resolve: (c) => pctOrDash(c.equip.magic_critical_hit_2_pct),
      breakdown: { equipKey: 'magic_critical_hit_2_pct' } },
    { id: 'fast_cast', label: 'ファストキャスト', category: '魔法',
      resolve: (c) => pctOrDash(c.totalStats.fast_cast_pct),
      breakdown: { equipKey: 'fast_cast_pct', charKey: 'fast_cast' } },
    { id: 'quick_magic', label: 'クイックマジック', category: '魔法',
      resolve: (c) => pctOrDash(c.equip.quick_magic_pct),
      breakdown: { equipKey: 'quick_magic_pct' } },

    // ----- 待機/回復 -----
    { id: 'regen', label: 'リジェネ', category: '待機/回復',
      resolve: (c) => numOrDash(c.totalStats.regen),
      breakdown: { equipKey: 'regen', charKey: 'regen' } },
    { id: 'refresh', label: 'リフレシュ', category: '待機/回復',
      resolve: (c) => numOrDash(c.totalStats.refresh),
      breakdown: { equipKey: 'refresh', charKey: 'refresh' } },
    { id: 'regain', label: 'リゲイン', category: '待機/回復',
      resolve: (c) => numOrDash(c.equip.regain),
      breakdown: { equipKey: 'regain' } },

    // ----- 武器スキル (テンプレート複製時の引き継ぎ用) -----
    { id: 'main_weapon_skill', label: 'メイン武器スキル', category: '武器スキル',
      resolve: (c) =>
          formatWeaponSkill(c.totalStats.main_weapon_skill, c.totalStats.main_weapon_skill_value) },
    { id: 'sub_weapon_skill', label: 'サブ武器スキル', category: '武器スキル',
      resolve: (c) =>
          formatWeaponSkill(c.totalStats.sub_weapon_skill, c.totalStats.sub_weapon_skill_value) },
    { id: 'ranged_weapon_skill', label: 'レンジ武器スキル', category: '武器スキル',
      resolve: (c) =>
          formatWeaponSkill(c.totalStats.ranged_weapon_skill, c.totalStats.ranged_weapon_skill_value) },

    // ----- 属性耐性 -----
    ...ELEMENT_RESISTS.map(([key, ja]): BuiltinPropertyItem => ({
        id: `resist_${key}`,
        label: `${ja}耐性`,
        category: '属性耐性',
        resolve: (c) => numOrDash(c.equip[`resist_${key}`]),
        breakdown: { equipKey: `resist_${key}` },
    })),

    // ----- 状態異常レジスト (テナシティ込みの合算値) -----
    // デスのみテナシティ非適用 (combineStatusResist と同じ規則)
    ...STATUS_RESISTS.map(([key, ja]): BuiltinPropertyItem => ({
        id: `resist_${key}`,
        label: `${ja}レジスト`,
        category: '状態異常レジスト',
        resolve: (c) => numOrDash(c.derived.statusResists[key]),
        breakdown: {
            equipKey: `resist_${key}`,
            ...(key === 'death' ? {} : { charKey: 'tenacity' }),
        },
    })),
];

/**
 * 内訳非対応のカタログ項目 id (breakdown メタなし)。
 * 新項目を追加するときは breakdown メタを付けるか、非対応と判断して
 * このリストに明示すること (catalog.test.ts が網羅を検証する)。
 */
export const BREAKDOWN_UNSUPPORTED_IDS: readonly string[] = [
    // 武器スキルは "片手剣 (400)" 形式の文字列値で行方向の加算が成立しない
    'main_weapon_skill',
    'sub_weapon_skill',
    'ranged_weapon_skill',
];

export const BUILTIN_ITEM_BY_ID: Map<string, BuiltinPropertyItem> = new Map(
    BUILTIN_PROPERTY_ITEMS.map((item) => [item.id, item])
);

// テンプレート (現行サブタブ) → カタログ項目 id の対応。「複製して編集」の雛形。
// テンプレートは表形式のためカタログに対応項目が無い行 (基本 9 ステ、魔回避、
// 魔法スキル値など) は落ちる。
export const TEMPLATE_ITEM_IDS: Record<string, string[]> = {
    'subtab-defense': [
        'regen', 'refresh', 'regain',
        'fast_cast', 'quick_magic', 'snapshot', 'rapid_shot',
        ...ELEMENT_RESISTS.map(([key]) => `resist_${key}`),
        ...STATUS_RESISTS.map(([key]) => `resist_${key}`),
    ],
    'subtab-melee-auto': [
        'main_weapon_skill', 'sub_weapon_skill', 'attack', 'accuracy',
        'store_tp', 'subtle_blow', 'subtle_blow_2',
        'double_attack', 'triple_attack', 'quad_attack',
        'da_damage', 'ta_damage', 'crit_rate', 'crit_damage',
    ],
    'subtab-ranged-auto': [
        'ranged_weapon_skill', 'ranged_attack', 'ranged_accuracy',
        'store_tp', 'snapshot', 'rapid_shot',
    ],
    'subtab-melee-ws': [
        'main_weapon_skill', 'sub_weapon_skill', 'attack', 'accuracy',
        'store_tp', 'subtle_blow', 'subtle_blow_2',
        'double_attack', 'triple_attack', 'quad_attack', 'crit_rate', 'crit_damage',
        'ws_damage', 'tp_bonus', 'skillchain_bonus', 'pdl',
    ],
    'subtab-ranged-ws': [
        'ranged_weapon_skill', 'ranged_attack', 'ranged_accuracy',
        'store_tp', 'subtle_blow', 'subtle_blow_2', 'crit_rate', 'crit_damage',
        'ws_damage', 'tp_bonus', 'skillchain_bonus', 'pdl', 'true_shot',
    ],
    'subtab-elemental-ws': [
        'main_weapon_skill', 'ranged_weapon_skill',
        'magic_attack', 'magic_accuracy', 'store_tp', 'subtle_blow', 'subtle_blow_2',
        'magic_damage', 'magic_affinity', 'magic_crit_2',
        'ws_damage', 'tp_bonus', 'skillchain_bonus',
    ],
    'subtab-melee-elemental-ws': [
        'main_weapon_skill', 'sub_weapon_skill', 'attack', 'accuracy',
        'store_tp', 'subtle_blow', 'subtle_blow_2',
        'double_attack', 'triple_attack', 'quad_attack', 'crit_rate', 'crit_damage',
        'ws_damage', 'tp_bonus', 'skillchain_bonus', 'pdl',
        'magic_attack', 'magic_accuracy', 'magic_damage', 'magic_affinity', 'magic_crit_2',
    ],
    'subtab-ranged-elemental-ws': [
        'ranged_weapon_skill', 'ranged_attack', 'ranged_accuracy',
        'store_tp', 'subtle_blow', 'subtle_blow_2', 'crit_rate', 'crit_damage',
        'ws_damage', 'tp_bonus', 'skillchain_bonus', 'pdl', 'true_shot',
        'magic_attack', 'magic_accuracy', 'magic_damage', 'magic_affinity', 'magic_crit_2',
    ],
    // 魔法系: スキル値・魔回避・INT/MND/CHR/MP はカタログ対象外のため魔攻/魔命/魔法ダメのみ
    ...Object.fromEntries(
        [
            'divine', 'healing', 'enhancing', 'enfeebling', 'elemental', 'dark',
            'summoning', 'ninjutsu', 'song', 'blue', 'geomancy',
        ].map((kind) => [
            `subtab-magic-${kind}`,
            ['magic_attack', 'magic_accuracy', 'magic_damage', 'fast_cast'],
        ])
    ),
};
