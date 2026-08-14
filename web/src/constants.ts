// プロジェクトの純粋データ定数。状態 / 副作用を持たない。
// すべての参照は ES module の named import で行う。
// (旧 web/js/constants.js の TS 化)
//
// Tier 1 (共有メタデータ: JOBS / RACE_NAMES / SKILL_KEYS_* / EQUIPMENT_SLOTS) は
// `/data/*.json` から top-level await で取得し、Rust 側 (data_loader.rs) と
// 同一のソースを参照する。詳細は ADR/Plan: テーブルデータの JSON 外出し。
//
// 注意: top-level await の fetch は node (Vitest) では動かない。ユニット
// テストからは vi.mock で必要な定数のみ差し替えること (web/src/utils.test.ts 参照)。

export interface JobDef {
    key: string;
    name: string;
}

export interface SlotDef {
    key: string;
    label: string;
}

/** [key, name_ja] のタプル。既存 UI コードがこの形式に依存している */
export type SkillKeyPair = [string, string];

const fetchData = async (name: string): Promise<any[]> =>
    (await (await fetch(`./data/${name}.json`)).json()).data;

const [_jobs, _races, _skills, _slots, _jpCats, _meritCats] = await Promise.all([
    fetchData('jobs'),
    fetchData('races'),
    fetchData('skills'),
    fetchData('equipment_slots'),
    fetchData('job_categories'),
    fetchData('job_merit_categories'),
]);

// 既存コードとの互換のため、JS 側では `name` プロパティで参照
export const JOBS: JobDef[] = _jobs.map((j) => ({ key: j.key, name: j.name_ja }));

export const RACE_NAMES: Record<string, string> =
    Object.fromEntries(_races.map((r) => [r.key, r.name_ja]));

// ジョブポイントのカテゴリ名 (各ジョブ 10 カテゴリ)。data/job_categories.json から読み込み。
export const JP_CATEGORIES: Record<string, string[]> = _jpCats as never;

export const JP_MAX_TOTAL = 2100;
export const JP_CATEGORY_COUNT = 10;
export const JP_MAX_RANK = 20;

// ジョブ別メリットポイントのカテゴリ名 (Group 1 / Group 2、固定 8 枠でパディング)
// data/job_merit_categories.json から読み込み。"カテゴリ N" は wiki に項目がない枠 (描画しない)
export const JOB_MERIT_GROUP_SIZE = 8;
export const JOB_MERIT_MAX_RANK = 5;
export const JOB_MERIT_GROUP_MAX_TOTAL = 10;
export const JOB_MERIT_CATEGORIES: Record<string, Record<string, string[]>> = _meritCats as never;
export const JOB_MERIT_PLACEHOLDER_RE = /^カテゴリ\s*\d+$/;

// WASM の SkillKind と対応するキー順 (Rust 側 SkillKind enum の宣言順)
const _toSkillTuples = (cat: string): SkillKeyPair[] =>
    _skills.filter((s) => s.category === cat).map((s) => [s.key, s.name_ja]);
export const SKILL_KEYS_WEAPON = _toSkillTuples('Weapon');
export const SKILL_KEYS_DEFENSE = _toSkillTuples('Defense');
export const SKILL_KEYS_MAGIC = _toSkillTuples('Magic');
export const ALL_SKILL_KEYS = [...SKILL_KEYS_WEAPON, ...SKILL_KEYS_DEFENSE, ...SKILL_KEYS_MAGIC];
export const COMBAT_SKILL_KEYS = [...SKILL_KEYS_WEAPON, ...SKILL_KEYS_DEFENSE];
export const MAGIC_SKILL_KEYS = SKILL_KEYS_MAGIC;

export const EQUIPMENT_SLOTS: SlotDef[] = _slots.map((s) => ({ key: s.key, label: s.label_ja }));

export const STORAGE_KEY = 'ff11sim_characters';
export const EQUIP_STORAGE_KEY = 'ff11sim_equipsets';
export const PROPSET_STORAGE_KEY = 'ff11sim_property_sets';

// 装備編集ステータス画面の左側パネル用テーブル定義
export const BASE_STATS = [
    { key: 'Hp', resultKey: 'hp', equipKey: 'hp', pctKey: 'hp_pct' },
    { key: 'Mp', resultKey: 'mp', equipKey: 'mp', pctKey: 'mp_pct' },
    { key: 'Str', resultKey: 'str_', equipKey: 'str' },
    { key: 'Dex', resultKey: 'dex', equipKey: 'dex' },
    { key: 'Vit', resultKey: 'vit', equipKey: 'vit' },
    { key: 'Agi', resultKey: 'agi', equipKey: 'agi' },
    { key: 'Int', resultKey: 'int', equipKey: 'int' },
    { key: 'Mnd', resultKey: 'mnd', equipKey: 'mnd' },
    { key: 'Chr', resultKey: 'chr', equipKey: 'chr' },
];

export const COMBAT_STATS = [
    { id: 'Def', key: 'def' },
    { id: 'Attack', key: 'attack', pctKey: 'attack_pct' },
    { id: 'Accuracy', key: 'accuracy' },
    { id: 'Evasion', key: 'evasion' },
    { id: 'RangedAttack', key: 'ranged_attack' },
    { id: 'RangedAccuracy', key: 'ranged_accuracy' },
    { id: 'MagicAttack', key: 'magic_attack' },
    { id: 'MagicAccuracy', key: 'magic_accuracy' },
    { id: 'MagicEvasion', key: 'magic_evasion' },
    { id: 'MagicDamage', key: 'magic_damage' },
    { id: 'Haste', key: 'haste_pct', isPct: true },
    { id: 'StoreTp', key: 'store_tp' },
    { id: 'DoubleAttack', key: 'double_attack_pct', isPct: true },
    { id: 'TripleAttack', key: 'triple_attack_pct', isPct: true },
    { id: 'CritRate', key: 'critical_hit_rate_pct', isPct: true },
    { id: 'WsDamage', key: 'weapon_skill_damage_pct', isPct: true },
];
