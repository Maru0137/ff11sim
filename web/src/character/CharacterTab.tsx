// キャラクター管理タブ (旧 web/js/character-list.js + character-form.js の React 移行)。
// 一覧・編集フォーム・保存/削除と、ジョブ別メリット / JP / スキルの入力 UI。
//
// 旧実装で currentEditing* のモジュール変数 5 つと DOM に分散していた
// 編集中状態を、1 つのフォーム状態 (FormState) に集約した。
// スキル上限の追従 (ジョブレベル・スキルメリット変更時に、未カスタムの
// スキル値だけ新デフォルトへ更新する) などの挙動は旧実装を踏襲。
import { useState, useSyncExternalStore } from 'react';
import { createRoot } from 'react-dom/client';
import { calculate_default_skills } from '../../js/wasm.js';
import { loadCharacters, saveCharacters } from '../../js/storage.js';
import {
    JOBS, RACE_NAMES, JP_CATEGORIES, JP_CATEGORY_COUNT, JP_MAX_RANK,
    JOB_MERIT_GROUP_SIZE, JOB_MERIT_MAX_RANK, JOB_MERIT_GROUP_MAX_TOTAL,
    SKILL_KEYS_WEAPON, SKILL_KEYS_DEFENSE, SKILL_KEYS_MAGIC,
    ALL_SKILL_KEYS, COMBAT_SKILL_KEYS, MAGIC_SKILL_KEYS,
} from '../../js/constants.js';
import {
    jpJobTotal, jpDefaultRanks,
    jobMeritDefaultRanks, jobMeritCategoryName, isJobMeritPlaceholder, samStoreTpIndex,
} from '../../js/utils.js';
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
    // 旧実装踏襲: 新規時のスキル上限はメリットポイント抜き (ゼロ) で算出する
    const skillDefaults = computeSkillDefaults({ race: 'Hum', job_levels: defaultJobLevels });
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

function CharacterTab() {
    const characters = useSyncExternalStore(charactersStore.subscribe, charactersStore.get);
    const [form, setForm] = useState<FormState | null>(null);
    // ジョブ別メリット / JP のジョブセレクタはフォームを閉じても保持 (旧実装踏襲)
    const [jmJob, setJmJob] = useState(jobs[0]?.key ?? '');
    const [jpJob, setJpJob] = useState(jobs[0]?.key ?? '');

    async function openEdit(name: string) {
        const list = await loadCharacters();
        const ch = (list as CharacterRecord[]).find((c) => c.name === name);
        if (ch) setForm(formStateFor(ch));
    }

    async function save() {
        if (!form) return;
        const name = form.name.trim();
        if (!name) {
            alert('キャラクター名を入力してください。');
            return;
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
                alert(`キャラクター「${name}」は既に存在します。`);
                return;
            }
            list.push(record);
        }
        await saveCharacters(list);
        await reloadCharacterList();
        setForm(null);
    }

    async function remove(name: string) {
        if (!confirm(`キャラクター「${name}」を削除しますか？`)) return;
        const list = ((await loadCharacters()) as CharacterRecord[]).filter((c) => c.name !== name);
        await saveCharacters(list);
        await reloadCharacterList();
        if (form?.editingName === name) setForm(null);
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
                            <li key={ch.name}>
                                <span>
                                    <span className="char-info">{ch.name}</span>
                                    <span className="char-race">{RACE_NAMES[ch.race] || ch.race}</span>
                                </span>
                                <span className="char-actions">
                                    <button className="btn btn-primary btn-sm" onClick={() => openEdit(ch.name)}>
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
                    <button className="btn btn-primary" id="btnNewChar" onClick={() => setForm(formStateFor(null))}>
                        新規キャラクター
                    </button>
                </div>
            </div>

            {form && (
                <CharacterForm
                    form={form}
                    setForm={setForm}
                    jmJob={jmJob}
                    setJmJob={setJmJob}
                    jpJob={jpJob}
                    setJpJob={setJpJob}
                    onSave={save}
                    onCancel={() => setForm(null)}
                />
            )}
        </div>
    );
}

interface CharacterFormProps {
    form: FormState;
    setForm: (f: FormState) => void;
    jmJob: string;
    setJmJob: (v: string) => void;
    jpJob: string;
    setJpJob: (v: string) => void;
    onSave: () => void;
    onCancel: () => void;
}

function CharacterForm({ form, setForm, jmJob, setJmJob, jpJob, setJpJob, onSave, onCancel }: CharacterFormProps) {
    const jmData = form.jobMerits[jmJob] || { group1: jobMeritDefaultRanks(), group2: jobMeritDefaultRanks() };
    const jpNames: string[] =
        (JP_CATEGORIES as Record<string, string[]>)[jpJob] ||
        new Array(JP_CATEGORY_COUNT).fill('').map((_, i) => `カテゴリ${i + 1}`);
    const jpRanks = form.jobPoints[jpJob] || jpDefaultRanks();

    function setJobMerit(group: 'group1' | 'group2', idx: number, raw: string) {
        let v = parseNum(raw);
        v = Math.max(0, Math.min(JOB_MERIT_MAX_RANK, v));
        // グループ合計が JOB_MERIT_GROUP_MAX_TOTAL を超えないようにクランプ
        const ranks = jmData[group];
        const otherSum = ranks.reduce((s, r, i) => s + (i === idx ? 0 : r || 0), 0);
        const allowed = Math.max(0, JOB_MERIT_GROUP_MAX_TOTAL - otherSum);
        if (v > allowed) v = allowed;
        const next = {
            ...jmData,
            [group]: ranks.map((r, i) => (i === idx ? v : r)),
        };
        setForm({ ...form, jobMerits: { ...form.jobMerits, [jmJob]: next } });
    }

    function setJpRank(idx: number, raw: string) {
        let v = parseNum(raw);
        v = Math.max(0, Math.min(JP_MAX_RANK, v));
        setForm({
            ...form,
            jobPoints: { ...form.jobPoints, [jpJob]: jpRanks.map((r, i) => (i === idx ? v : r)) },
        });
    }

    function setMeritSkill(key: string, raw: string) {
        let v = parseNum(raw);
        if (v < 0) v = 0;
        if (v > 8) v = 8;
        // スキルメリットはスキル上限に影響するため、上限を再計算して追従させる
        setForm(withFollowedDefaults({
            ...form,
            meritSkills: { ...form.meritSkills, [key]: v },
        }));
    }

    function setJobLevel(jobKey: string, field: 'level' | 'masterLv', raw: string) {
        // ジョブレベル / ML はスキル上限に影響するため、上限を再計算して追従させる
        setForm(withFollowedDefaults({
            ...form,
            jobLevels: {
                ...form.jobLevels,
                [jobKey]: { ...form.jobLevels[jobKey], [field]: raw },
            },
        }));
    }

    function setSkill(key: string, raw: string) {
        let v = parseNum(raw);
        if (v < 0) v = 0;
        setForm({ ...form, skills: { ...form.skills, [key]: v } });
    }

    function resetSkillsToDefault() {
        const next = withFollowedDefaults(form);
        const skills = { ...next.skills };
        allSkillKeys.forEach(([k]) => {
            skills[k] = next.skillDefaults[k] || 0;
        });
        setForm({ ...next, skills });
    }

    function meritSkillGrid(keys: SkillKeyPair[]) {
        return (
            <div className="merit-grid">
                {keys.map(([key, ja]) => (
                    <div className="merit-item" key={key}>
                        <label htmlFor={`meritSkill_${key}`}>{ja}</label>
                        <input
                            type="number"
                            id={`meritSkill_${key}`}
                            min={0}
                            max={8}
                            value={form.meritSkills[key] != null ? form.meritSkills[key] : 8}
                            onChange={(e) => setMeritSkill(key, e.target.value)}
                        />
                    </div>
                ))}
            </div>
        );
    }

    function skillGrid(keys: SkillKeyPair[]) {
        return (
            <div className="skill-grid">
                {keys.map(([key, ja]) => {
                    const cap = form.skillDefaults[key] || 0;
                    const value = form.skills[key] != null ? form.skills[key] : cap;
                    return (
                        <div className="skill-item" key={key}>
                            <label htmlFor={`skill_${key}`}>
                                <span>{ja}</span>
                                <span className="skill-cap">上限 {cap}</span>
                            </label>
                            <input
                                type="number"
                                id={`skill_${key}`}
                                min={0}
                                value={value}
                                onChange={(e) => setSkill(key, e.target.value)}
                            />
                        </div>
                    );
                })}
            </div>
        );
    }

    function jobMeritGrid(group: 'group1' | 'group2') {
        const items = [];
        for (let i = 0; i < JOB_MERIT_GROUP_SIZE; i++) {
            if (isJobMeritPlaceholder(jmJob, group, i)) continue;
            items.push(
                <div className="jp-item" key={i}>
                    <label htmlFor={`jm_${group}_${i}`}>{jobMeritCategoryName(jmJob, group, i)}</label>
                    <input
                        type="number"
                        id={`jm_${group}_${i}`}
                        min={0}
                        max={JOB_MERIT_MAX_RANK}
                        value={jmData[group][i]}
                        onChange={(e) => setJobMerit(group, i, e.target.value)}
                    />
                </div>
            );
        }
        if (items.length === 0) {
            return <div style={{ color: '#666', fontSize: 12 }}>(項目なし)</div>;
        }
        return items;
    }

    const groupTotal = (group: 'group1' | 'group2') =>
        jmData[group].reduce((s, v) => s + (v || 0), 0);

    return (
        <div id="charEditSection">
            <h2 id="charEditTitle">{form.editingName ? 'キャラクター編集' : '新規キャラクター'}</h2>
            <div className="form-group">
                <label htmlFor="charName">名前</label>
                <input
                    type="text"
                    id="charName"
                    placeholder="キャラクター名"
                    value={form.name}
                    onChange={(e) => setForm({ ...form, name: e.target.value })}
                />
            </div>
            <div className="form-group">
                <label htmlFor="charRace">種族</label>
                <select
                    id="charRace"
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

            <h3>ジョブレベル</h3>
            <table className="job-level-table">
                <thead>
                    <tr>
                        <th>ジョブ</th>
                        <th>Lv</th>
                        <th>MLv</th>
                    </tr>
                </thead>
                <tbody id="jobLevelTable">
                    {jobs.map((job) => (
                        <tr key={job.key}>
                            <td className="job-name">{job.key}</td>
                            <td>
                                <input
                                    type="number"
                                    id={`jl_${job.key}_lv`}
                                    min={0}
                                    max={99}
                                    value={form.jobLevels[job.key]?.level ?? '0'}
                                    onChange={(e) => setJobLevel(job.key, 'level', e.target.value)}
                                />
                            </td>
                            <td>
                                <input
                                    type="number"
                                    id={`jl_${job.key}_mlv`}
                                    min={0}
                                    max={50}
                                    value={form.jobLevels[job.key]?.masterLv ?? '0'}
                                    onChange={(e) => setJobLevel(job.key, 'masterLv', e.target.value)}
                                />
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>

            <div className="merit-section">
                <h3>メリットポイント (0-15)</h3>
                <div className="merit-grid">
                    {MERIT_BASE_KEYS.map(([key, id, label]) => (
                        <div className="merit-item" key={key}>
                            <label htmlFor={id}>{label}</label>
                            <input
                                type="number"
                                id={id}
                                min={0}
                                max={15}
                                value={form.meritBase[key]}
                                onChange={(e) =>
                                    setForm({ ...form, meritBase: { ...form.meritBase, [key]: e.target.value } })
                                }
                            />
                        </div>
                    ))}
                </div>
            </div>

            <div className="merit-section">
                <h3>メリットポイント (スキル)</h3>
                <div className="skill-group-title">武器スキル (0-8)</div>
                {meritSkillGrid(SKILL_KEYS_WEAPON as SkillKeyPair[])}
                <div className="skill-group-title">防御スキル (0-8)</div>
                {meritSkillGrid(SKILL_KEYS_DEFENSE as SkillKeyPair[])}
                <div className="skill-group-title">魔法スキル (0-8)</div>
                {meritSkillGrid(MAGIC_SKILL_KEYS as SkillKeyPair[])}
            </div>

            <div className="merit-section">
                <h3>メリットポイント (その他)</h3>
                <div className="merit-grid">
                    {MERIT_OTHER_KEYS.map(([key, id, label]) => (
                        <div className="merit-item" key={key}>
                            <label htmlFor={id}>{label}</label>
                            <input
                                type="number"
                                id={id}
                                min={0}
                                max={5}
                                value={form.meritOther[key]}
                                onChange={(e) =>
                                    setForm({ ...form, meritOther: { ...form.meritOther, [key]: e.target.value } })
                                }
                            />
                        </div>
                    ))}
                </div>
            </div>

            <div className="jp-section">
                <div className="jp-section-header">
                    <h3>ジョブ別メリットポイント (各項目0-5)</h3>
                    <select id="jobMeritJobSelector" value={jmJob} onChange={(e) => setJmJob(e.target.value)}>
                        {jobs.map((j) => (
                            <option key={j.key} value={j.key}>{j.name}</option>
                        ))}
                    </select>
                </div>
                <div className="skill-group-title">
                    グループ1{' '}
                    <span style={{ color: '#888', fontSize: 12 }}>
                        (<span id="jobMeritGroup1Total">{groupTotal('group1')}</span> / 10)
                    </span>
                </div>
                <div className="jp-grid" id="jobMeritGroup1Grid">{jobMeritGrid('group1')}</div>
                <div className="skill-group-title">
                    グループ2{' '}
                    <span style={{ color: '#888', fontSize: 12 }}>
                        (<span id="jobMeritGroup2Total">{groupTotal('group2')}</span> / 10)
                    </span>
                </div>
                <div className="jp-grid" id="jobMeritGroup2Grid">{jobMeritGrid('group2')}</div>
            </div>

            <div className="jp-section">
                <div className="jp-section-header">
                    <h3>ジョブポイント (0-20)</h3>
                    <select id="jpJobSelector" value={jpJob} onChange={(e) => setJpJob(e.target.value)}>
                        {jobs.map((j) => (
                            <option key={j.key} value={j.key}>{j.name}</option>
                        ))}
                    </select>
                    <span className="jp-total-display">
                        合計: <span id="jpTotalDisplay">{jpJobTotal(jpRanks)}</span> / 2100
                    </span>
                </div>
                <div className="jp-grid" id="jpCategoryGrid">
                    {jpRanks.map((rank, i) => (
                        <div className="jp-item" key={i}>
                            <label htmlFor={`jp_cat_${i}`}>{jpNames[i] || `カテゴリ${i + 1}`}</label>
                            <input
                                type="number"
                                id={`jp_cat_${i}`}
                                min={0}
                                max={JP_MAX_RANK}
                                value={rank}
                                onChange={(e) => setJpRank(i, e.target.value)}
                            />
                        </div>
                    ))}
                </div>
            </div>

            <div className="skill-section">
                <div className="skill-section-header">
                    <h3>スキル値</h3>
                    <button
                        className="btn btn-primary btn-sm"
                        id="btnResetSkillsToDefault"
                        type="button"
                        onClick={resetSkillsToDefault}
                    >
                        全て最大値に戻す
                    </button>
                </div>
                <div className="skill-group-title">武器スキル</div>
                {skillGrid(SKILL_KEYS_WEAPON as SkillKeyPair[])}
                <div className="skill-group-title">防御スキル</div>
                {skillGrid(SKILL_KEYS_DEFENSE as SkillKeyPair[])}
                <div className="skill-group-title">魔法スキル</div>
                {skillGrid(SKILL_KEYS_MAGIC as SkillKeyPair[])}
            </div>

            <div className="btn-group">
                <button className="btn btn-primary" id="btnSaveChar" onClick={onSave}>保存</button>
                <button className="btn btn-danger" id="btnCancelEdit" onClick={onCancel}>キャンセル</button>
            </div>
        </div>
    );
}

export function mountCharacterTab(container: HTMLElement) {
    createRoot(container).render(<CharacterTab />);
}
