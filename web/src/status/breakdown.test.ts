// 内訳モデル (breakdown.ts) のテスト。
// equip-store / format 経由で constants (top-level await fetch) に届くため
// 必要な定数のみモックする (equip-bonuses.test.ts と同じ理由)。
import { describe, it, expect, vi } from 'vitest';

const SLOT_KEYS = [
    'main', 'sub', 'range', 'ammo', 'head', 'body', 'hands', 'legs',
    'feet', 'neck', 'waist', 'ear1', 'ear2', 'ring1', 'ring2', 'back',
];

vi.mock('../constants', () => ({
    EQUIPMENT_SLOTS: [
        ['main', 'メイン'], ['sub', 'サブ'], ['range', 'レンジ'], ['ammo', '矢弾'],
        ['head', '頭'], ['body', '胴'], ['hands', '両手'], ['legs', '両脚'],
        ['feet', '両足'], ['neck', '首'], ['waist', '腰'], ['ear1', '左耳'],
        ['ear2', '右耳'], ['ring1', '左指'], ['ring2', '右指'], ['back', '背'],
    ].map(([key, label]) => ({ key, label })),
    ALL_SKILL_KEYS: [],
}));

const { STATUS_BREAKDOWN_COLUMNS, buildPropsetColumns, buildBreakdownModel } =
    await import('./breakdown');

describe('buildPropsetColumns', () => {
    it('カタログの breakdown メタから列を組み立てる', () => {
        const cols = buildPropsetColumns(['store_tp', 'fast_cast', 'quad_attack']);
        expect(cols.map((c) => c.key)).toEqual(['store_tp', 'fast_cast', 'quad_attack']);
        expect(cols[0]).toMatchObject({
            label: 'ストアTP', equipKey: 'store_tp', charKey: 'store_tp', totalId: 'store_tp',
        });
        // 装備のみの項目は charKey なし
        expect(cols[2].charKey).toBeUndefined();
        expect(cols[2].equipKey).toBe('quad_attack_pct');
    });

    it('内訳非対応の項目 (武器スキル) と未知 id は列から除外する', () => {
        const cols = buildPropsetColumns(['main_weapon_skill', 'store_tp', 'no_such_id']);
        expect(cols.map((c) => c.key)).toEqual(['store_tp']);
    });

    it('ユーザー定義項目は装備行のみ (userItemId 経由) の列になる', () => {
        const cols = buildPropsetColumns(['user:二刀流']);
        expect(cols[0]).toMatchObject({
            key: 'user:二刀流', label: '二刀流',
            totalId: 'user:二刀流', userItemId: 'user:二刀流',
        });
        expect(cols[0].equipKey).toBeUndefined();
        expect(cols[0].charKey).toBeUndefined();
    });

    it('状態異常レジストはデスのみ tenacity 行なし', () => {
        const cols = buildPropsetColumns(['resist_sleep', 'resist_death']);
        expect(cols[0].charKey).toBe('tenacity');
        expect(cols[1].charKey).toBeUndefined();
    });
});

describe('buildBreakdownModel', () => {
    const baseInputs = {
        columns: STATUS_BREAKDOWN_COLUMNS,
        slots: {
            head: { item_id: 0, name_ja: 'テスト兜', aug_rank: 15 },
        },
        perSlotStats: {
            head: { hp: 145, def: 145, physical_damage_taken_pct: -11 },
        },
        perSlotUserValues: {},
        charRows: {
            race: { hp: 485, str: 37.5 },
            main_job: { hp: 855, str: 45 },
            support_job: { str: 15.25 },
            base: { def: 272 },
            master_level: { hp: 350 },
        },
        totals: {
            equipTotalHp: 1945,
            equipTotalStr: 117,
            equipTotalDef: 430,
        },
        charItemLabels: {
            race: 'ヒューム',
            main_job: '戦士99 (ジョブ+特性)',
            base: 'Lv・ステータス・スキル由来',
        },
    };

    it('装備 16 部位 + キャラ由来 8 行を列順のセルで組み立てる', () => {
        const model = buildBreakdownModel(baseInputs);
        expect(model.rows).toHaveLength(16 + 8);
        expect(model.rows.slice(0, 16).map((r) => r.key)).toEqual(SLOT_KEYS);
        expect(model.rows.slice(16).map((r) => r.key)).toEqual([
            'base', 'race', 'main_job', 'support_job',
            'merit', 'job_points', 'gift', 'master_level',
        ]);

        const col = (key: string) => STATUS_BREAKDOWN_COLUMNS.findIndex((c) => c.key === key);
        const row = (key: string) => model.rows.find((r) => r.key === key)!;

        // 装備行: per_slot_stats から。0 は null、負値は保持
        expect(row('head').cells[col('hp')]).toBe(145);
        expect(row('head').cells[col('str')]).toBeNull();
        expect(row('head').cells[col('pdt')]).toBe(-11);
        expect(row('body').cells.every((c) => c === null)).toBe(true);

        // キャラ行: charRows から。f32 由来の小数は保持 (表示桁で丸め)
        expect(row('race').cells[col('str')]).toBe(37.5);
        expect(row('support_job').cells[col('str')]).toBe(15.25);
        expect(row('base').cells[col('def')]).toBe(272);
        expect(row('merit').cells.every((c) => c === null)).toBe(true);

        // ラベル: 装備行は名前 + オーグメントランク、キャラ行は補足ラベル
        expect(row('head').itemLabel).toBe('テスト兜 R15');
        expect(row('race').itemLabel).toBe('ヒューム');
        expect(row('sub').itemLabel).toBe('');
    });

    it('合計行はパネル表示値をそのまま参照し、未定義は "-"', () => {
        const model = buildBreakdownModel(baseInputs);
        const col = (key: string) => STATUS_BREAKDOWN_COLUMNS.findIndex((c) => c.key === key);
        expect(model.totals[col('hp')]).toBe(1945);
        expect(model.totals[col('def')]).toBe(430);
        // totals に無い列 (ヘイスト等) は '-'
        expect(model.totals[col('haste')]).toBe('-');
    });

    it('ユーザー定義項目の列はスロット別ユーザー値から装備行を埋める', () => {
        const cols = buildPropsetColumns(['user:二刀流']);
        const model = buildBreakdownModel({
            ...baseInputs,
            columns: cols,
            perSlotUserValues: { body: { 'user:二刀流': 5 } },
            totals: { 'user:二刀流': 5 },
        });
        const row = (key: string) => model.rows.find((r) => r.key === key)!;
        expect(row('body').cells[0]).toBe(5);
        expect(row('head').cells[0]).toBeNull();
        // キャラ行は全て null
        expect(row('main_job').cells[0]).toBeNull();
        expect(model.totals[0]).toBe(5);
    });
});
