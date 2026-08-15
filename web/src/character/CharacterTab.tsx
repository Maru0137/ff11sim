// キャラクター管理タブ (旧 web/js/character-list.js + character-form.js の React 移行)。
// 一覧・編集フォーム・保存/削除と、ジョブ別メリット / JP / スキルの入力 UI。
//
// 旧実装で currentEditing* のモジュール変数 5 つと DOM に分散していた
// 編集中状態を、1 つのフォーム状態 (FormState) に集約した。
// スキル上限の追従 (ジョブレベル・スキルメリット変更時に、未カスタムの
// スキル値だけ新デフォルトへ更新する) などの挙動は旧実装を踏襲。
import { useEffect, useRef, useState, useSyncExternalStore } from 'react';
import type { ReactNode } from 'react';
import { calculate_default_skills } from '../wasm';
import { loadCharacters, saveCharacters } from '../storage';
import { guard, registerDirtyEditor } from '../dirty-guard';
import {
    JOBS, RACE_NAMES, JP_CATEGORIES, JP_CATEGORY_COUNT, JP_MAX_RANK, JP_MAX_TOTAL,
    JOB_MERIT_GROUP_SIZE, JOB_MERIT_MAX_RANK, JOB_MERIT_GROUP_MAX_TOTAL,
    MERIT_BASE_MAX_RANK, MERIT_SKILL_MAX_RANK, MERIT_OTHER_MAX_RANK,
    MERIT_OTHER_GROUP_MAX_TOTAL,
    SKILL_KEYS_WEAPON, SKILL_KEYS_DEFENSE, SKILL_KEYS_MAGIC,
    ALL_SKILL_KEYS, COMBAT_SKILL_KEYS, MAGIC_SKILL_KEYS,
} from '../constants';
import {
    jpJobTotal, jpDefaultRanks,
    jobMeritDefaultRanks, jobMeritCategoryName, isJobMeritPlaceholder, samStoreTpIndex,
} from '../utils';
import { clampToMax, clampWithinGroup } from './limits';
import { moveItem } from './reorder';
import { charactersStore, reloadCharacterList } from './character-store';
import type { CharacterRecord } from './character-store';

interface JobDef {
    key: string;
    name: string;
}

type SkillKeyPair = [string, string];

const jobs = JOBS as JobDef[];
const allSkillKeys = ALL_SKILL_KEYS as SkillKeyPair[];

const MERIT_BASE_KEYS = [
    ['hp', 'charMeritHp', 'HP'],
    ['mp', 'charMeritMp', 'MP'],
    ['str_', 'charMeritStr', 'STR'],
    ['dex', 'charMeritDex', 'DEX'],
    ['vit', 'charMeritVit', 'VIT'],
    ['agi', 'charMeritAgi', 'AGI'],
    ['int', 'charMeritInt', 'INT'],
    ['mnd', 'charMeritMnd', 'MND'],
    ['chr', 'charMeritChr', 'CHR'],
] as const;

const MERIT_OTHER_KEYS = [
    ['enmity_plus', 'charMeritEnmityPlus', '敵対心+'],
    ['enmity_minus', 'charMeritEnmityMinus', '敵対心-'],
    ['critical_hit_rate', 'charMeritCriticalHitRate', 'クリティカルヒット率'],
    ['enemy_critical_hit_rate', 'charMeritEnemyCriticalHitRate', '被クリティカルヒット率'],
    ['spell_interruption_rate', 'charMeritSpellInterruptionRate', '詠唱中断率'],
] as const;

interface FormState {
    /** 編集中の既存キャラクター名。新規なら null */
    editingName: string | null;
    name: string;
    race: string;
    /** 入力途中の空文字を許すため文字列で保持し、計算・保存時に parse する */
    jobLevels: Record<string, { level: string; masterLv: string }>;
    meritBase: Record<string, string>;
    meritOther: Record<string, string>;
    meritSkills: Record<string, number>;
    jobMerits: Record<string, { group1: number[]; group2: number[] }>;
    jobPoints: Record<string, number[]>;
    skills: Record<string, number>;
    skillDefaults: Record<string, number>;
}

function parseNum(s: string): number {
    const v = parseInt(s, 10);
    return isNaN(v) ? 0 : v;
}

function buildJobLevels(f: FormState) {
    const job_levels: Record<string, { level: number; master_lv: number }> = {};
    jobs.forEach((job) => {
        const jl = f.jobLevels[job.key] || { level: '0', masterLv: '0' };
        job_levels[job.key] = { level: parseNum(jl.level), master_lv: parseNum(jl.masterLv) };
    });
    return job_levels;
}

// 旧 readMeritPointsFromForm の移植
function buildMeritPoints(f: FormState) {
    const combat_skill_merits: Record<string, number> = {};
    (COMBAT_SKILL_KEYS as SkillKeyPair[]).forEach(([k]) => {
        combat_skill_merits[k] = f.meritSkills[k] || 0;
    });
    const magic_skill_merits: Record<string, number> = {};
    (MAGIC_SKILL_KEYS as SkillKeyPair[]).forEach(([k]) => {
        magic_skill_merits[k] = f.meritSkills[k] || 0;
    });
    const job_merits: Record<string, { group1: number[]; group2: number[] }> = {};
    jobs.forEach((job) => {
        const m = f.jobMerits[job.key] || { group1: jobMeritDefaultRanks(), group2: jobMeritDefaultRanks() };
        job_merits[job.key] = { group1: m.group1.slice(), group2: m.group2.slice() };
    });
    return {
        hp: parseNum(f.meritBase.hp),
        mp: parseNum(f.meritBase.mp),
        str_: parseNum(f.meritBase.str_),
        dex: parseNum(f.meritBase.dex),
        vit: parseNum(f.meritBase.vit),
        agi: parseNum(f.meritBase.agi),
        int: parseNum(f.meritBase.int),
        mnd: parseNum(f.meritBase.mnd),
        chr: parseNum(f.meritBase.chr),
        combat_skill_merits,
        magic_skill_merits,
        enmity_plus: parseNum(f.meritOther.enmity_plus),
        enmity_minus: parseNum(f.meritOther.enmity_minus),
        critical_hit_rate: parseNum(f.meritOther.critical_hit_rate),
        enemy_critical_hit_rate: parseNum(f.meritOther.enemy_critical_hit_rate),
        spell_interruption_rate: parseNum(f.meritOther.spell_interruption_rate),
        store_tp: (job_merits.Sam && job_merits.Sam.group1[samStoreTpIndex()]) || 0,
        job_merits,
    };
}

function computeSkillDefaults(characterLike: {
    name?: string;
    race: string;
    job_levels: Record<string, { level: number; master_lv: number }>;
    merit_points?: unknown;
}): Record<string, number> {
    const basicProfile = {
        name: characterLike.name || '_temp',
        race: characterLike.race,
        job_levels: characterLike.job_levels,
        merit_points: characterLike.merit_points || {
            hp: 0, mp: 0, str_: 0, dex: 0, vit: 0, agi: 0, int: 0, mnd: 0, chr: 0,
        },
    };
    return calculate_default_skills(basicProfile);
}

// スキル上限を再計算し、未カスタム (旧デフォルトと同値) のスキルだけ追従させる
function withFollowedDefaults(f: FormState): FormState {
    const oldDefaults = f.skillDefaults;
    const newDefaults = computeSkillDefaults({
        race: f.race,
        job_levels: buildJobLevels(f),
        merit_points: buildMeritPoints(f),
    });
    const skills = { ...f.skills };
    allSkillKeys.forEach(([k]) => {
        if (skills[k] === oldDefaults[k]) {
            skills[k] = newDefaults[k] || 0;
        }
    });
    return { ...f, skills, skillDefaults: newDefaults };
}

// 旧 setFormStateFromCharacter / resetFormStateForNew の移植
function formStateFor(character: CharacterRecord | null): FormState {
    if (character) {
        const mp = (character.merit_points || {}) as Record<string, any>;
        const jobLevels: FormState['jobLevels'] = {};
        jobs.forEach((job) => {
            const jl = character.job_levels[job.key] || { level: 0, master_lv: 0 };
            jobLevels[job.key] = { level: String(jl.level), masterLv: String(jl.master_lv) };
        });
        const meritBase: Record<string, string> = {};
        MERIT_BASE_KEYS.forEach(([key]) => {
            meritBase[key] = String(mp[key] || 0);
        });
        const meritOther: Record<string, string> = {};
        MERIT_OTHER_KEYS.forEach(([key]) => {
            meritOther[key] = String(mp[key] || 0);
        });
        const csm = mp.combat_skill_merits || {};
        const msm = mp.magic_skill_merits || {};
        const meritSkills: Record<string, number> = {};
        (COMBAT_SKILL_KEYS as SkillKeyPair[]).forEach(([k]) => {
            meritSkills[k] = csm[k] != null ? csm[k] : 8;
        });
        (MAGIC_SKILL_KEYS as SkillKeyPair[]).forEach(([k]) => {
            meritSkills[k] = msm[k] != null ? msm[k] : 8;
        });
        const stored = mp.job_merits || {};
        const jobMerits: FormState['jobMerits'] = {};
        jobs.forEach((job) => {
            const s = stored[job.key];
            jobMerits[job.key] = {
                group1: (s && Array.isArray(s.group1) && s.group1.length === JOB_MERIT_GROUP_SIZE) ? s.group1.slice() : jobMeritDefaultRanks(),
                group2: (s && Array.isArray(s.group2) && s.group2.length === JOB_MERIT_GROUP_SIZE) ? s.group2.slice() : jobMeritDefaultRanks(),
            };
        });
        // 旧フォーマットからの移行: merit_points.store_tp を SAM G1 ストアTP に backfill
        if (mp.store_tp && jobMerits.Sam) {
            const idx = samStoreTpIndex();
            if (idx >= 0 && !jobMerits.Sam.group1[idx]) {
                jobMerits.Sam.group1[idx] = mp.store_tp;
            }
        }
        const storedCategories = (character.job_points && character.job_points.categories) || {};
        const jobPoints: FormState['jobPoints'] = {};
        jobs.forEach((job) => {
            const s = storedCategories[job.key];
            if (s && Array.isArray(s.ranks) && s.ranks.length === JP_CATEGORY_COUNT) {
                jobPoints[job.key] = s.ranks.slice();
            } else {
                jobPoints[job.key] = jpDefaultRanks();
            }
        });
        const skillDefaults = computeSkillDefaults(character);
        const skills: Record<string, number> = {};
        const storedValues = (character.skills && character.skills.values) || null;
        allSkillKeys.forEach(([k]) => {
            if (storedValues && storedValues[k] != null) {
                skills[k] = storedValues[k];
            } else {
                skills[k] = skillDefaults[k] || 0;
            }
        });
        return {
            editingName: character.name,
            name: character.name,
            race: character.race,
            jobLevels, meritBase, meritOther, meritSkills, jobMerits, jobPoints,
            skills, skillDefaults,
        };
    }

    // 新規: 全ジョブ 99 / メリット 15 / スキルメリット 8 で開始
    const jobLevels: FormState['jobLevels'] = {};
    const defaultJobLevels: Record<string, { level: number; master_lv: number }> = {};
    jobs.forEach((job) => {
        jobLevels[job.key] = { level: '99', masterLv: '0' };
        defaultJobLevels[job.key] = { level: 99, master_lv: 0 };
    });
    const meritBase: Record<string, string> = {};
    MERIT_BASE_KEYS.forEach(([key]) => {
        meritBase[key] = '15';
    });
    const meritOther: Record<string, string> = {};
    MERIT_OTHER_KEYS.forEach(([key]) => {
        meritOther[key] = '0';
    });
    const meritSkills: Record<string, number> = {};
    allSkillKeys.forEach(([k]) => {
        meritSkills[k] = 8;
    });
    const jobMerits: FormState['jobMerits'] = {};
    jobs.forEach((job) => {
        jobMerits[job.key] = { group1: jobMeritDefaultRanks(), group2: jobMeritDefaultRanks() };
    });
    const jobPoints: FormState['jobPoints'] = {};
    jobs.forEach((job) => {
        jobPoints[job.key] = jpDefaultRanks();
    });
    // フォームが持つスキルメリット (既定 8 = 上限 +16) を含めて上限を算出する。
    // 旧実装はここだけメリット抜きで計算しており、表示上の「上限」が実際のキャップより
    // 16 低かった。入力を上限でクランプするようになった (docs/adr/0021) ため、
    // ずれていると本来入る値が弾かれてしまう。
    const skillDefaults = computeSkillDefaults({
        race: 'Hum',
        job_levels: defaultJobLevels,
        merit_points: {
            hp: 15, mp: 15, str_: 15, dex: 15, vit: 15, agi: 15, int: 15, mnd: 15, chr: 15,
            combat_skill_merits: Object.fromEntries(
                (COMBAT_SKILL_KEYS as SkillKeyPair[]).map(([k]) => [k, MERIT_SKILL_MAX_RANK])),
            magic_skill_merits: Object.fromEntries(
                (MAGIC_SKILL_KEYS as SkillKeyPair[]).map(([k]) => [k, MERIT_SKILL_MAX_RANK])),
        },
    });
    const skills: Record<string, number> = {};
    allSkillKeys.forEach(([k]) => {
        skills[k] = skillDefaults[k] || 0;
    });
    return {
        editingName: null,
        name: '',
        race: 'Hum',
        jobLevels, meritBase, meritOther, meritSkills, jobMerits, jobPoints,
        skills, skillDefaults,
    };
}

export const CHARACTER_EDITOR_ID = 'characters';

// 開閉式セクション (docs/adr/0021)。既定で開くのは基本情報とジョブレベルだけ。
const SECTIONS = [
    { id: 'basic', label: '基本情報' },
    { id: 'joblv', label: 'ジョブレベル' },
    { id: 'merit', label: 'メリットポイント (共通)' },
    { id: 'jobpt', label: 'ジョブ別ポイント' },
    { id: 'skill', label: 'スキル値' },
] as const;

type SectionId = (typeof SECTIONS)[number]['id'];

const OPEN_SECTIONS_KEY = 'ff11sim_char_sections';
const DEFAULT_OPEN: SectionId[] = ['basic', 'joblv'];

function loadOpenSections(): SectionId[] {
    try {
        const raw = localStorage.getItem(OPEN_SECTIONS_KEY);
        if (!raw) return DEFAULT_OPEN;
        const ids = JSON.parse(raw) as unknown;
        if (!Array.isArray(ids)) return DEFAULT_OPEN;
        return SECTIONS.map((s) => s.id).filter((id) => ids.includes(id));
    } catch {
        return DEFAULT_OPEN;
    }
}

function saveOpenSections(ids: SectionId[]) {
    try {
        localStorage.setItem(OPEN_SECTIONS_KEY, JSON.stringify(ids));
    } catch {
        // localStorage が使えなくても開閉自体は動くので握りつぶす
    }
}

export function CharacterTab() {
    const characters = useSyncExternalStore(charactersStore.subscribe, charactersStore.get);
    const [form, setForm] = useState<FormState | null>(null);
    // フォームを開いた / 保存した時点の内容。これとの差が「未保存の変更」
    // (docs/adr/0020)。値を戻せば未保存でなくなる。
    const [baseline, setBaseline] = useState<string | null>(null);
    // 一覧のドラッグ並び替えの進行状態
    const [draggingName, setDraggingName] = useState<string | null>(null);
    const [dragOverName, setDragOverName] = useState<string | null>(null);
    // ジョブ別ポイント (ジョブ別メリット + JP) のジョブセレクタ。
    // 同じ「どのジョブか」を指すので 1 つに統合した (docs/adr/0021)。
    // フォームを閉じても保持する (旧実装踏襲)。
    const [targetJob, setTargetJob] = useState(jobs[0]?.key ?? '');

    const dirty = form !== null && JSON.stringify(form) !== baseline;

    function openForm(next: FormState | null) {
        setForm(next);
        setBaseline(next && JSON.stringify(next));
    }

    async function openEdit(name: string) {
        const list = await loadCharacters();
        const ch = (list as CharacterRecord[]).find((c) => c.name === name);
        if (ch) openForm(formStateFor(ch));
    }

    // 未保存確認ダイアログから保存 / 破棄を呼べるように、最新の実装を渡す。
    // 依存配列を空にできるよう ref 経由にする (毎レンダー登録し直さない)。
    const editorRef = useRef({ dirty, form, save: async (): Promise<true | string> => true });
    editorRef.current = { dirty, form, save };
    useEffect(
        () =>
            registerDirtyEditor(CHARACTER_EDITOR_ID, {
                label: () =>
                    `キャラクター「${editorRef.current.form?.name.trim() || '(名称未設定)'}」`,
                isDirty: () => editorRef.current.dirty,
                save: () => editorRef.current.save(),
                discard: () => openForm(null),
            }),
        []
    );

    async function save(): Promise<true | string> {
        if (!form) return true;
        const name = form.name.trim();
        if (!name) {
            return 'キャラクター名を入力してください。';
        }
        const job_levels = buildJobLevels(form);
        const merit_points = buildMeritPoints(form);
        const job_points_categories: Record<string, { ranks: number[] }> = {};
        jobs.forEach((job) => {
            const ranks = form.jobPoints[job.key] || jpDefaultRanks();
            job_points_categories[job.key] = { ranks: ranks.slice() };
        });
        const skill_values: Record<string, number> = {};
        allSkillKeys.forEach(([k]) => {
            skill_values[k] = form.skills[k] != null ? form.skills[k] : 0;
        });
        const record = {
            name,
            race: form.race,
            job_levels,
            merit_points,
            job_points: { categories: job_points_categories },
            skills: { values: skill_values },
        };

        const list: CharacterRecord[] = await loadCharacters();
        if (form.editingName) {
            const idx = list.findIndex((c) => c.name === form.editingName);
            if (idx >= 0) list[idx] = record;
        } else {
            if (list.some((c) => c.name === name)) {
                return `キャラクター「${name}」は既に存在します。`;
            }
            list.push(record);
        }
        await saveCharacters(list);
        await reloadCharacterList();
        // 旧実装踏襲: 保存したらフォームを閉じる (= 未保存状態も解消)
        openForm(null);
        return true;
    }

    async function remove(name: string) {
        if (!confirm(`キャラクター「${name}」を削除しますか？`)) return;
        const list = ((await loadCharacters()) as CharacterRecord[]).filter((c) => c.name !== name);
        await saveCharacters(list);
        await reloadCharacterList();
        if (form?.editingName === name) openForm(null);
    }

    // ドラッグ並び替え (docs/adr/0022)。配列の順序がそのまま保存順になる。
    // 編集中のフォームには触れないので、未保存ガードは掛けない。
    async function moveCharacter(fromName: string, toName: string) {
        const list = (await loadCharacters()) as CharacterRecord[];
        const next = moveItem(
            list,
            list.findIndex((c) => c.name === fromName),
            list.findIndex((c) => c.name === toName)
        );
        if (next === list) return;
        await saveCharacters(next as CharacterRecord[]);
        await reloadCharacterList();
    }

    return (
        <div className="container">
            <div>
                <h2>登録キャラクター</h2>
                <ul className="char-list" id="charList">
                    {characters.length === 0 ? (
                        <li className="empty-msg">キャラクターが登録されていません</li>
                    ) : (
                        characters.map((ch) => (
                            <li
                                key={ch.name}
                                className={[
                                    ch.name === draggingName ? 'dragging' : '',
                                    ch.name === dragOverName ? 'drag-over' : '',
                                ].filter(Boolean).join(' ') || undefined}
                                onDragOver={(e) => {
                                    if (!draggingName) return;
                                    e.preventDefault();
                                    e.dataTransfer.dropEffect = 'move';
                                    setDragOverName(ch.name);
                                }}
                                onDragLeave={() => setDragOverName(null)}
                                onDrop={(e) => {
                                    e.preventDefault();
                                    const from = e.dataTransfer.getData('text/plain');
                                    if (from) void moveCharacter(from, ch.name);
                                    setDraggingName(null);
                                    setDragOverName(null);
                                }}
                            >
                                {/* 行全体を draggable にすると編集 / 削除ボタンが押しにくくなるため
                                    ハンドルだけを掴めるようにする (PropsetManageModal と同じ) */}
                                <span
                                    className="char-drag-handle"
                                    title="ドラッグで並び替え"
                                    aria-label={`${ch.name} を並び替え`}
                                    draggable
                                    onDragStart={(e) => {
                                        e.dataTransfer.effectAllowed = 'move';
                                        e.dataTransfer.setData('text/plain', ch.name);
                                        setDraggingName(ch.name);
                                    }}
                                    onDragEnd={() => {
                                        setDraggingName(null);
                                        setDragOverName(null);
                                    }}
                                >
                                    ⠿
                                </span>
                                <span className="char-main">
                                    <span className="char-info">{ch.name}</span>
                                    <span className="char-race">{RACE_NAMES[ch.race] || ch.race}</span>
                                </span>
                                <span className="char-actions">
                                    <button
                                        className="btn btn-primary btn-sm"
                                        onClick={() =>
                                            guard(
                                                `キャラクター「${ch.name}」の編集`,
                                                () => void openEdit(ch.name),
                                                { editorId: CHARACTER_EDITOR_ID }
                                            )
                                        }
                                    >
                                        編集
                                    </button>
                                    <button className="btn btn-danger btn-sm" onClick={() => remove(ch.name)}>
                                        削除
                                    </button>
                                </span>
                            </li>
                        ))
                    )}
                </ul>
                <div className="btn-group">
                    <button
                        className="btn btn-primary"
                        id="btnNewChar"
                        onClick={() =>
                            guard('新規キャラクターの作成', () => openForm(formStateFor(null)), {
                                editorId: CHARACTER_EDITOR_ID,
                            })
                        }
                    >
                        新規キャラクター
                    </button>
                </div>
            </div>

            {form && (
                <CharacterForm
                    form={form}
                    setForm={setForm}
                    dirty={dirty}
                    targetJob={targetJob}
                    setTargetJob={setTargetJob}
                    onSave={() => {
                        void save().then((r) => {
                            if (r !== true) alert(r);
                        });
                    }}
                    onClose={() =>
                        guard('編集フォームを閉じる', () => openForm(null), {
                            editorId: CHARACTER_EDITOR_ID,
                        })
                    }
                />
            )}
        </div>
    );
}


interface CharacterFormProps {
    form: FormState;
    setForm: (f: FormState) => void;
    /** 保存済みから変更されているか (保存ボタンの活性と未保存バッジに使う) */
    dirty: boolean;
    /** ジョブ別メリットとジョブポイントで共有する対象ジョブ */
    targetJob: string;
    setTargetJob: (v: string) => void;
    onSave: () => void;
    onClose: () => void;
}

const MERIT_BASE_TOTAL_MAX = MERIT_BASE_KEYS.length * MERIT_BASE_MAX_RANK;
const MERIT_SKILL_TOTAL_MAX = ALL_SKILL_KEYS.length * MERIT_SKILL_MAX_RANK;

/** フォーカスで中身を全選択する。デフォルト値を消してから打ち直す手間を省く (docs/adr/0021) */
function selectAll(e: React.FocusEvent<HTMLInputElement>) {
    e.target.select();
}

interface NumFieldProps {
    id: string;
    label: string;
    /** 文字列で受けるのは、入力途中の空欄を許すフィールドがあるため */
    value: string | number;
    max: number;
    onChange: (raw: string) => void;
    /** merit-item | jp-item | skill-item */
    variant: string;
    disabled?: boolean;
    title?: string;
}

/** ラベル・数値入力・「/ 最大値」を 1 行に並べる共通フィールド (docs/adr/0021) */
function NumField({ id, label, value, max, onChange, variant, disabled, title }: NumFieldProps) {
    const num = typeof value === 'number' ? value : parseNum(value);
    // 上限 0 は「達成」ではないので強調しない
    const atCap = max > 0 && num >= max;
    return (
        <div className={atCap ? `${variant} at-cap` : variant}>
            <label htmlFor={id} title={title || label}>{label}</label>
            <input
                type="number"
                id={id}
                min={0}
                max={max}
                value={value}
                disabled={disabled}
                title={title}
                onFocus={selectAll}
                onChange={(e) => onChange(e.target.value)}
            />
            <span className="field-max">/ {max}</span>
        </div>
    );
}

interface SectionProps {
    id: SectionId;
    label: string;
    summary?: ReactNode;
    open: boolean;
    onToggle: (id: SectionId, open: boolean) => void;
    children: ReactNode;
}

function Section({ id, label, summary, open, onToggle, children }: SectionProps) {
    return (
        <details
            className="char-section"
            data-section={id}
            open={open}
            onToggle={(e) => onToggle(id, e.currentTarget.open)}
        >
            <summary>
                <span className="char-section-caret" aria-hidden="true">▶</span>
                <span className="char-section-title">{label}</span>
                {summary != null && <span className="char-section-summary">{summary}</span>}
            </summary>
            <div className="char-section-body">{children}</div>
        </details>
    );
}

/** 親セクション内の小見出し + 右肩の合計表示 */
function SubGroup({ title, hint, total, children }: {
    title: string;
    hint?: string;
    total?: ReactNode;
    children: ReactNode;
}) {
    return (
        <div className="char-subgroup">
            <div className="char-subgroup-head">
                <span className="char-subgroup-title">{title}</span>
                {hint && <span className="char-subgroup-hint">{hint}</span>}
                {total != null && <span className="char-subgroup-total">{total}</span>}
            </div>
            {children}
        </div>
    );
}

function CharacterForm({ form, setForm, dirty, targetJob, setTargetJob, onSave, onClose }: CharacterFormProps) {
    const [openSections, setOpenSections] = useState<SectionId[]>(loadOpenSections);

    function toggleSection(id: SectionId, open: boolean) {
        setOpenSections((prev) => {
            const has = prev.includes(id);
            if (open === has) return prev;
            const next = open ? [...prev, id] : prev.filter((s) => s !== id);
            saveOpenSections(next);
            return next;
        });
    }
    const isOpen = (id: SectionId) => openSections.includes(id);

    const jmData = form.jobMerits[targetJob] || { group1: jobMeritDefaultRanks(), group2: jobMeritDefaultRanks() };
    const jpNames: string[] =
        (JP_CATEGORIES as Record<string, string[]>)[targetJob] ||
        new Array(JP_CATEGORY_COUNT).fill('').map((_, i) => `カテゴリ${i + 1}`);
    const jpRanks = form.jobPoints[targetJob] || jpDefaultRanks();

    function setJobMerit(group: 'group1' | 'group2', idx: number, raw: string) {
        const ranks = jmData[group];
        const v = clampWithinGroup(ranks, idx, parseNum(raw), JOB_MERIT_MAX_RANK, JOB_MERIT_GROUP_MAX_TOTAL);
        const next = { ...jmData, [group]: ranks.map((r, i) => (i === idx ? v : r)) };
        setForm({ ...form, jobMerits: { ...form.jobMerits, [targetJob]: next } });
    }

    function setJpRank(idx: number, raw: string) {
        const v = clampToMax(parseNum(raw), JP_MAX_RANK);
        setForm({
            ...form,
            jobPoints: { ...form.jobPoints, [targetJob]: jpRanks.map((r, i) => (i === idx ? v : r)) },
        });
    }

    function setMeritBase(key: string, raw: string) {
        // 入力途中の空欄を潰さないよう、空文字はそのまま保持する
        const v = raw === '' ? '' : String(clampToMax(parseNum(raw), MERIT_BASE_MAX_RANK));
        setForm({ ...form, meritBase: { ...form.meritBase, [key]: v } });
    }

    function setMeritOther(key: string, raw: string) {
        if (raw === '') {
            setForm({ ...form, meritOther: { ...form.meritOther, [key]: '' } });
            return;
        }
        // 各項目 0-5 に加えてグループ合計にも上限がある (docs/knowledge/status/merit_points.md)
        const ranks = MERIT_OTHER_KEYS.map(([k]) => parseNum(form.meritOther[k]));
        const idx = MERIT_OTHER_KEYS.findIndex(([k]) => k === key);
        const v = clampWithinGroup(ranks, idx, parseNum(raw), MERIT_OTHER_MAX_RANK, MERIT_OTHER_GROUP_MAX_TOTAL);
        setForm({ ...form, meritOther: { ...form.meritOther, [key]: String(v) } });
    }

    function setMeritSkill(key: string, raw: string) {
        const v = clampToMax(parseNum(raw), MERIT_SKILL_MAX_RANK);
        // スキルメリットはスキル上限に影響するため、上限を再計算して追従させる
        setForm(withFollowedDefaults({
            ...form,
            meritSkills: { ...form.meritSkills, [key]: v },
        }));
    }

    function setJobLevel(jobKey: string, field: 'level' | 'masterLv', raw: string) {
        const max = field === 'level' ? 99 : 50;
        const v = raw === '' ? '' : String(clampToMax(parseNum(raw), max));
        // ジョブレベル / ML はスキル上限に影響するため、上限を再計算して追従させる
        setForm(withFollowedDefaults({
            ...form,
            jobLevels: {
                ...form.jobLevels,
                [jobKey]: { ...form.jobLevels[jobKey], [field]: v },
            },
        }));
    }

    function setSkill(key: string, raw: string) {
        // 上限を超える値は入力させない。上限はジョブレベル由来なので欄ごとに違う
        const v = clampToMax(parseNum(raw), form.skillDefaults[key] || 0);
        setForm({ ...form, skills: { ...form.skills, [key]: v } });
    }

    /** 33 欄すべてを現在の上限で埋める */
    function setSkillsToCap() {
        const next = withFollowedDefaults(form);
        const skills = { ...next.skills };
        allSkillKeys.forEach(([k]) => {
            skills[k] = next.skillDefaults[k] || 0;
        });
        setForm({ ...next, skills });
    }

    function meritSkillGrid(keys: SkillKeyPair[], groupLabel: string) {
        const groupMax = keys.length * MERIT_SKILL_MAX_RANK;
        const total = keys.reduce((s, [k]) => s + (form.meritSkills[k] ?? MERIT_SKILL_MAX_RANK), 0);
        return (
            <>
                <div className="char-group-row">
                    <span className="char-group-title">{groupLabel}</span>
                    <span className="char-group-total">計 {total} / {groupMax}pt</span>
                </div>
                <div className="merit-grid merit-grid-narrow">
                    {keys.map(([key, ja]) => (
                        <NumField
                            key={key}
                            variant="merit-item"
                            id={`meritSkill_${key}`}
                            label={ja}
                            max={MERIT_SKILL_MAX_RANK}
                            value={form.meritSkills[key] ?? MERIT_SKILL_MAX_RANK}
                            title={`${ja}スキル上限アップ (ランク × 2 が上限に加算される)`}
                            onChange={(raw) => setMeritSkill(key, raw)}
                        />
                    ))}
                </div>
            </>
        );
    }

    function skillGrid(keys: SkillKeyPair[], groupLabel: string) {
        return (
            <>
                <div className="char-group-row">
                    <span className="char-group-title">{groupLabel}</span>
                </div>
                <div className="skill-grid">
                    {keys.map(([key, ja]) => {
                        const cap = form.skillDefaults[key] || 0;
                        const value = form.skills[key] ?? cap;
                        return (
                            <NumField
                                key={key}
                                variant={cap === 0 ? 'skill-item skill-item-zero' : 'skill-item'}
                                id={`skill_${key}`}
                                label={ja}
                                max={cap}
                                value={value}
                                // 上限 0 = Lv>0 のジョブに該当スキルのランクが無い。0 しか入らないので閉じる
                                disabled={cap === 0}
                                title={cap === 0
                                    ? `${ja}: Lv1 以上のジョブに該当スキルがないため上限 0`
                                    : `${ja}: 上限 ${cap} (Lv1 以上のジョブのうち最大のキャップ + スキルメリット × 2)`}
                                onChange={(raw) => setSkill(key, raw)}
                            />
                        );
                    })}
                </div>
            </>
        );
    }

    function jobMeritGrid(group: 'group1' | 'group2') {
        const items = [];
        for (let i = 0; i < JOB_MERIT_GROUP_SIZE; i++) {
            if (isJobMeritPlaceholder(targetJob, group, i)) continue;
            const name = jobMeritCategoryName(targetJob, group, i);
            items.push(
                <NumField
                    key={i}
                    variant="jp-item"
                    id={`jm_${group}_${i}`}
                    label={name}
                    max={JOB_MERIT_MAX_RANK}
                    value={jmData[group][i]}
                    onChange={(raw) => setJobMerit(group, i, raw)}
                />
            );
        }
        if (items.length === 0) {
            return <div className="char-empty-note">(項目なし)</div>;
        }
        return <div className="jp-grid">{items}</div>;
    }

    const groupTotal = (group: 'group1' | 'group2') =>
        jmData[group].reduce((s, v) => s + (v || 0), 0);

    const meritBaseTotal = MERIT_BASE_KEYS.reduce((s, [key]) => s + parseNum(form.meritBase[key]), 0);
    const meritSkillTotal = allSkillKeys.reduce(
        (s, [k]) => s + (form.meritSkills[k] ?? MERIT_SKILL_MAX_RANK), 0);
    const meritOtherTotal = MERIT_OTHER_KEYS.reduce((s, [key]) => s + parseNum(form.meritOther[key]), 0);
    const jpTotal = jpJobTotal(jpRanks);
    const zeroCapCount = allSkillKeys.filter(([k]) => !form.skillDefaults[k]).length;
    const targetJobName = jobs.find((j) => j.key === targetJob)?.name ?? targetJob;

    const jobSelect = (
        <select
            id="targetJobSelector"
            className="char-job-select"
            value={targetJob}
            onChange={(e) => setTargetJob(e.target.value)}
        >
            {jobs.map((j) => (
                <option key={j.key} value={j.key}>{j.name}</option>
            ))}
        </select>
    );

    return (
        <div id="charEditSection">
            {/* スクロールしても保存できるよう上端に固定する (docs/adr/0021) */}
            <div className="char-action-bar">
                <span className="char-action-title" id="charEditTitle">
                    {form.editingName ? 'キャラクター編集' : '新規キャラクター'}
                </span>
                {dirty && <span className="unsaved-badge">未保存</span>}
                <span className="char-action-spacer" />
                <button
                    className="btn btn-primary"
                    id="btnSaveChar"
                    disabled={!dirty}
                    title={dirty ? undefined : '変更がありません'}
                    onClick={onSave}
                >
                    保存
                </button>
                {/* 「破棄」ではなくフォームを閉じる操作。未保存ならガードが確認する */}
                <button className="btn btn-neutral" id="btnCancelEdit" onClick={onClose}>閉じる</button>
            </div>

            <Section
                id="basic"
                label="基本情報"
                summary={`${form.name.trim() || '(名称未設定)'} / ${RACE_NAMES[form.race] || form.race}`}
                open={isOpen('basic')}
                onToggle={toggleSection}
            >
                <div className="char-basic-row">
                    <div className="form-group">
                        <label htmlFor="charName">名前</label>
                        <input
                            type="text"
                            id="charName"
                            className="char-field-name"
                            placeholder="キャラクター名"
                            value={form.name}
                            onChange={(e) => setForm({ ...form, name: e.target.value })}
                        />
                    </div>
                    <div className="form-group">
                        <label htmlFor="charRace">種族</label>
                        <select
                            id="charRace"
                            className="char-field-race"
                            value={form.race}
                            onChange={(e) => setForm(withFollowedDefaults({ ...form, race: e.target.value }))}
                        >
                            <option value="Hum">ヒューム</option>
                            <option value="Elv">エルヴァーン</option>
                            <option value="Tar">タルタル</option>
                            <option value="Mit">ミスラ</option>
                            <option value="Gal">ガルカ</option>
                        </select>
                    </div>
                </div>
            </Section>

            <Section id="joblv" label="ジョブレベル" open={isOpen('joblv')} onToggle={toggleSection}>
                <div className="job-grid" id="jobLevelGrid">
                    {jobs.map((job) => {
                        const jl = form.jobLevels[job.key];
                        const lv = jl?.level ?? '0';
                        return (
                            <div className={parseNum(lv) > 0 ? 'job-chip is-leveled' : 'job-chip'} key={job.key}>
                                <span className="job-chip-name" title={job.name}>{job.name}</span>
                                <label className="job-chip-tag" htmlFor={`jl_${job.key}_lv`}>Lv</label>
                                <input
                                    type="number"
                                    id={`jl_${job.key}_lv`}
                                    min={0}
                                    max={99}
                                    value={lv}
                                    onFocus={selectAll}
                                    onChange={(e) => setJobLevel(job.key, 'level', e.target.value)}
                                />
                                <label className="job-chip-tag" htmlFor={`jl_${job.key}_mlv`}>ML</label>
                                <input
                                    type="number"
                                    id={`jl_${job.key}_mlv`}
                                    min={0}
                                    max={50}
                                    value={jl?.masterLv ?? '0'}
                                    onFocus={selectAll}
                                    onChange={(e) => setJobLevel(job.key, 'masterLv', e.target.value)}
                                />
                            </div>
                        );
                    })}
                </div>
            </Section>

            <Section
                id="merit"
                label="メリットポイント (共通)"
                summary={
                    <>
                        基礎 <b>{meritBaseTotal}</b>/{MERIT_BASE_TOTAL_MAX}pt ・
                        スキル <b>{meritSkillTotal}</b>/{MERIT_SKILL_TOTAL_MAX}pt ・
                        その他 <b>{meritOtherTotal}</b>/{MERIT_OTHER_GROUP_MAX_TOTAL}pt
                    </>
                }
                open={isOpen('merit')}
                onToggle={toggleSection}
            >
                <SubGroup
                    title="基礎ステータス"
                    hint={`各 0-${MERIT_BASE_MAX_RANK}`}
                    total={<>計 <b>{meritBaseTotal}</b> / {MERIT_BASE_TOTAL_MAX}pt</>}
                >
                    <div className="merit-grid merit-grid-narrow">
                        {MERIT_BASE_KEYS.map(([key, id, label]) => (
                            <NumField
                                key={key}
                                variant="merit-item"
                                id={id}
                                label={label}
                                max={MERIT_BASE_MAX_RANK}
                                value={form.meritBase[key]}
                                onChange={(raw) => setMeritBase(key, raw)}
                            />
                        ))}
                    </div>
                </SubGroup>

                <SubGroup
                    title="スキル"
                    hint={`各 0-${MERIT_SKILL_MAX_RANK}`}
                    total={<>計 <b>{meritSkillTotal}</b> / {MERIT_SKILL_TOTAL_MAX}pt</>}
                >
                    {meritSkillGrid(SKILL_KEYS_WEAPON as SkillKeyPair[], '武器スキル')}
                    {meritSkillGrid(SKILL_KEYS_DEFENSE as SkillKeyPair[], '防御スキル')}
                    {meritSkillGrid(MAGIC_SKILL_KEYS as SkillKeyPair[], '魔法スキル')}
                </SubGroup>

                <SubGroup
                    title="その他"
                    hint={`各 0-${MERIT_OTHER_MAX_RANK} ・ 合計 ${MERIT_OTHER_GROUP_MAX_TOTAL}pt まで`}
                    total={<>計 <b>{meritOtherTotal}</b> / {MERIT_OTHER_GROUP_MAX_TOTAL}pt</>}
                >
                    <div className="merit-grid merit-grid-wide">
                        {MERIT_OTHER_KEYS.map(([key, id, label]) => (
                            <NumField
                                key={key}
                                variant="merit-item"
                                id={id}
                                label={label}
                                max={MERIT_OTHER_MAX_RANK}
                                value={form.meritOther[key]}
                                onChange={(raw) => setMeritOther(key, raw)}
                            />
                        ))}
                    </div>
                </SubGroup>
            </Section>

            <Section
                id="jobpt"
                label="ジョブ別ポイント"
                summary={
                    <>
                        {targetJobName} ・ G1 <b>{groupTotal('group1')}</b>/{JOB_MERIT_GROUP_MAX_TOTAL}{' '}
                        G2 <b>{groupTotal('group2')}</b>/{JOB_MERIT_GROUP_MAX_TOTAL} ・
                        JP <b>{jpTotal}</b>/{JP_MAX_TOTAL}
                    </>
                }
                open={isOpen('jobpt')}
                onToggle={toggleSection}
            >
                {/* ジョブ別メリットと JP は同じ「どのジョブか」を指すのでセレクタを共有する */}
                <div className="char-job-picker">
                    <label htmlFor="targetJobSelector">対象ジョブ</label>
                    {jobSelect}
                    <span className="char-job-picker-hint">
                        ジョブ別メリットとジョブポイントの両方に効きます
                    </span>
                </div>

                <SubGroup title="ジョブ別メリットポイント" hint={`各項目 0-${JOB_MERIT_MAX_RANK} ・ グループ合計 ${JOB_MERIT_GROUP_MAX_TOTAL}pt まで`}>
                    <div className="char-group-row">
                        <span className="char-group-title">グループ1</span>
                        <span className="char-group-total">
                            計 <span id="jobMeritGroup1Total">{groupTotal('group1')}</span> / {JOB_MERIT_GROUP_MAX_TOTAL}pt
                        </span>
                    </div>
                    {jobMeritGrid('group1')}
                    <div className="char-group-row">
                        <span className="char-group-title">グループ2</span>
                        <span className="char-group-total">
                            計 <span id="jobMeritGroup2Total">{groupTotal('group2')}</span> / {JOB_MERIT_GROUP_MAX_TOTAL}pt
                        </span>
                    </div>
                    {jobMeritGrid('group2')}
                </SubGroup>

                <SubGroup
                    title="ジョブポイント"
                    hint={`各 0-${JP_MAX_RANK}`}
                    total={<>計 <span id="jpTotalDisplay">{jpTotal}</span> / {JP_MAX_TOTAL}pt</>}
                >
                    <div className="jp-grid" id="jpCategoryGrid">
                        {jpRanks.map((rank, i) => (
                            <NumField
                                key={i}
                                variant="jp-item"
                                id={`jp_cat_${i}`}
                                label={jpNames[i] || `カテゴリ${i + 1}`}
                                max={JP_MAX_RANK}
                                value={rank}
                                onChange={(raw) => setJpRank(i, raw)}
                            />
                        ))}
                    </div>
                </SubGroup>
            </Section>

            <Section
                id="skill"
                label="スキル値"
                summary={
                    <>
                        {allSkillKeys.length} 項目
                        {zeroCapCount > 0 && <> ・ 上限 0 が <b>{zeroCapCount}</b> 件</>}
                    </>
                }
                open={isOpen('skill')}
                onToggle={toggleSection}
            >
                <div className="char-subgroup-head">
                    <button
                        className="btn btn-primary btn-sm"
                        id="btnResetSkillsToDefault"
                        type="button"
                        onClick={setSkillsToCap}
                    >
                        全て上限値にする
                    </button>
                    <span className="char-subgroup-hint">上限を超える値は入力できません</span>
                </div>
                {skillGrid(SKILL_KEYS_WEAPON as SkillKeyPair[], '武器スキル')}
                {skillGrid(SKILL_KEYS_DEFENSE as SkillKeyPair[], '防御スキル')}
                {skillGrid(SKILL_KEYS_MAGIC as SkillKeyPair[], '魔法スキル')}
            </Section>
        </div>
    );
}
