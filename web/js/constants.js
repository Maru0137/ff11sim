// プロジェクトの純粋データ定数。状態 / 副作用を持たない。
// すべての参照は ES module の named import で行う。
//
// Tier 1 (共有メタデータ: JOBS / RACE_NAMES / SKILL_KEYS_* / EQUIPMENT_SLOTS) は
// `/data/*.json` から top-level await で取得し、Rust 側 (data_loader.rs) と
// 同一のソースを参照する。詳細は ADR/Plan: テーブルデータの JSON 外出し。

const fetchData = async (name) =>
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
export const JOBS = _jobs.map((j) => ({ key: j.key, name: j.name_ja }));

export const RACE_NAMES = Object.fromEntries(_races.map((r) => [r.key, r.name_ja]));

// ジョブポイントのカテゴリ名 (各ジョブ 10 カテゴリ)。data/job_categories.json から読み込み。
export const JP_CATEGORIES = _jpCats;

export const JP_MAX_TOTAL = 2100;
export const JP_CATEGORY_COUNT = 10;
export const JP_MAX_RANK = 20;

// ジョブ別メリットポイントのカテゴリ名 (Group 1 / Group 2、固定 8 枠でパディング)
// data/job_merit_categories.json から読み込み。"カテゴリ N" は wiki に項目がない枠 (描画しない)
export const JOB_MERIT_GROUP_SIZE = 8;
export const JOB_MERIT_MAX_RANK = 5;
export const JOB_MERIT_GROUP_MAX_TOTAL = 10;
export const JOB_MERIT_CATEGORIES = _meritCats;
export const JOB_MERIT_PLACEHOLDER_RE = /^カテゴリ\s*\d+$/;

// WASM の SkillKind と対応するキー順 (Rust 側 SkillKind enum の宣言順)
// `[key, name_ja]` のタプル配列形式は既存 UI コードに依存しているため維持
const _toSkillTuples = (cat) =>
    _skills.filter((s) => s.category === cat).map((s) => [s.key, s.name_ja]);
export const SKILL_KEYS_WEAPON = _toSkillTuples('Weapon');
export const SKILL_KEYS_DEFENSE = _toSkillTuples('Defense');
export const SKILL_KEYS_MAGIC = _toSkillTuples('Magic');
export const ALL_SKILL_KEYS = [...SKILL_KEYS_WEAPON, ...SKILL_KEYS_DEFENSE, ...SKILL_KEYS_MAGIC];
export const COMBAT_SKILL_KEYS = [...SKILL_KEYS_WEAPON, ...SKILL_KEYS_DEFENSE];
export const MAGIC_SKILL_KEYS = SKILL_KEYS_MAGIC;

export const EQUIPMENT_SLOTS = _slots.map((s) => ({ key: s.key, label: s.label_ja }));

export const STORAGE_KEY = 'ff11sim_characters';
export const EQUIP_STORAGE_KEY = 'ff11sim_equipsets';

// オーグメント JA→EN 変換テーブル。長いパターンを先に置くこと（部分一致回避）。
export const AUGMENT_JA_TO_EN = [
    ['ウェポンスキルのダメージ', 'Weapon skill damage'],
    ['マジックバーストダメージ', 'Magic burst damage'],
    ['魔法クリティカルヒットII', 'Magic Crit. Hit Rate II'],
    ['魔法クリティカルヒット率', 'Magic Critical hit rate'],
    ['物理ダメージ上限', 'Physical damage limit'],
    ['被物理ダメージ', 'Physical damage taken'],
    ['被魔法ダメージ', 'Magic damage taken'],
    ['クリティカルヒットダメージ', 'Critical hit damage'],
    ['クリティカルヒット率', 'Critical hit rate'],
    ['トリプルアタックダメージ', 'Triple Attack damage'],
    ['トリプルアタック', '"Triple Attack"'],
    ['ダブルアタックダメージ', 'Double Attack damage'],
    ['ダブルアタック', '"Double Attack"'],
    ['クワッドアタック', '"Quadruple Attack"'],
    ['モクシャII', '"Subtle Blow II"'],
    ['モクシャ', '"Subtle Blow"'],
    ['魔法ダメージ', 'Magic Damage'],
    // 個別魔法スキル名 (「魔法スキル」より先に置換しないと「強化魔法スキル」→「強化Magic skills」になり
    //                   汎用パターンで全 14 スキルに加算される regression が発生する)
    ['神聖魔法スキル', 'Divine magic skill'],
    ['回復魔法スキル', 'Healing magic skill'],
    ['強化魔法スキル', 'Enhancing magic skill'],
    ['弱体魔法スキル', 'Enfeebling magic skill'],
    ['精霊魔法スキル', 'Elemental magic skill'],
    ['暗黒魔法スキル', 'Dark magic skill'],
    ['召喚魔法スキル', 'Summoning magic skill'],
    ['青魔法スキル', 'Blue magic skill'],
    ['風水魔法スキル', 'Geomancy skill'],
    ['忍術スキル', 'Ninjutsu skill'],
    ['歌唱スキル', 'Singing skill'],
    ['弦楽器スキル', 'String instrument skill'],
    ['管楽器スキル', 'Wind instrument skill'],
    ['風水鈴スキル', 'Handbell skill'],
    // 全魔法スキル一括加算 (extractSkillBonuses で 14 種に展開される)
    ['魔法スキル', 'Magic skills'],
    ['被ダメージ', 'Damage taken'],
    ['ストアTP', '"Store TP"'],
    ['TPボーナス', '"TP Bonus"'],
    ['連携ボーナス', '"Skillchain Bonus"'],
    // 連携ダメージ +N% (Mpaca 系オーグメント等) は内部的に Skillchain Bonus と同種扱い
    ['連携ダメージ', '"Skillchain Bonus"'],
    ['トゥルーショット', '"True Shot"'],
    ['アフィニティ', 'Affinity'],
    ['ヘイスト', 'Haste'],
    ['魔回避', 'Magic Evasion'],
    ['飛攻', 'Ranged Attack'],
    ['飛命', 'Ranged Accuracy'],
    ['魔命', 'Magic Accuracy'],
    ['魔攻', '"Magic Atk. Bonus"'],
    ['回避', 'Evasion'],
    ['命中', 'Accuracy'],
    ['攻', 'Attack'],
];

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
