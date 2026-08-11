// キャラクター編集フォームの部品層。ジョブ別メリット・ジョブポイント (JP)・
// スキル/スキルメリットの入力 UI と、その編集中状態を持つ。
//
// currentEditing* の編集中状態はこのモジュールに閉じる。呼び出し側
// (キャラクター一覧・保存) は 3 つの seam:
//   setFormStateFromCharacter() / resetFormStateForNew() / collectFormStateForSave()
// と readMeritPointsFromForm() を通じてのみ状態を受け渡す。
import { calculate_default_skills } from './wasm.js';
import {
    JOBS, JP_CATEGORIES, JP_CATEGORY_COUNT, JP_MAX_RANK,
    JOB_MERIT_GROUP_SIZE, JOB_MERIT_MAX_RANK, JOB_MERIT_GROUP_MAX_TOTAL,
    SKILL_KEYS_WEAPON, SKILL_KEYS_DEFENSE, SKILL_KEYS_MAGIC,
    ALL_SKILL_KEYS, COMBAT_SKILL_KEYS, MAGIC_SKILL_KEYS,
} from './constants.js';
import {
    jpJobTotal, jpDefaultRanks,
    jobMeritDefaultRanks, jobMeritCategoryName, isJobMeritPlaceholder, samStoreTpIndex,
} from './utils.js';

/// 現在編集中キャラクターのジョブ別メリット
let currentEditingJobMerits = {};

/// 現在編集中キャラクターの JP データ（ジョブキー → ranks 配列）
let currentEditingJobPoints = {};

let currentEditingSkills = {}; // skill_key -> value
let currentEditingSkillDefaults = {}; // skill_key -> default (最大キャップ)
let currentEditingMeritSkills = {}; // skill_key -> merit rank (0-8)

function loadJobMeritsFromCharacter(character) {
    const result = {};
    const stored = (character && character.merit_points && character.merit_points.job_merits) || {};
    JOBS.forEach(job => {
        const s = stored[job.key];
        result[job.key] = {
            group1: (s && Array.isArray(s.group1) && s.group1.length === JOB_MERIT_GROUP_SIZE) ? s.group1.slice() : jobMeritDefaultRanks(),
            group2: (s && Array.isArray(s.group2) && s.group2.length === JOB_MERIT_GROUP_SIZE) ? s.group2.slice() : jobMeritDefaultRanks(),
        };
    });
    return result;
}

function jobMeritGroupTotal(jobKey, group) {
    const data = currentEditingJobMerits[jobKey];
    if (!data || !data[group]) return 0;
    return data[group].reduce((s, v) => s + (v || 0), 0);
}
function updateJobMeritGroupTotalDisplay() {
    const jobKey = document.getElementById('jobMeritJobSelector').value;
    for (const group of ['group1', 'group2']) {
        const id = group === 'group1' ? 'jobMeritGroup1Total' : 'jobMeritGroup2Total';
        document.getElementById(id).textContent = jobMeritGroupTotal(jobKey, group);
    }
}
export function renderJobMeritInputs() {
    const jobKey = document.getElementById('jobMeritJobSelector').value;
    const data = currentEditingJobMerits[jobKey] || { group1: jobMeritDefaultRanks(), group2: jobMeritDefaultRanks() };
    for (const group of ['group1', 'group2']) {
        const grid = document.getElementById(group === 'group1' ? 'jobMeritGroup1Grid' : 'jobMeritGroup2Grid');
        grid.innerHTML = '';
        for (let i = 0; i < JOB_MERIT_GROUP_SIZE; i++) {
            if (isJobMeritPlaceholder(jobKey, group, i)) continue;
            const item = document.createElement('div');
            item.className = 'jp-item';
            item.innerHTML = `
                <label for="jm_${group}_${i}">${jobMeritCategoryName(jobKey, group, i)}</label>
                <input type="number" id="jm_${group}_${i}" min="0" max="${JOB_MERIT_MAX_RANK}" value="${data[group][i]}" data-merit-group="${group}" data-merit-index="${i}">
            `;
            grid.appendChild(item);
        }
        if (grid.innerHTML === '') {
            grid.innerHTML = '<div style="color:#666;font-size:12px;">(項目なし)</div>';
        }
    }
    updateJobMeritGroupTotalDisplay();
}

export function ensureJobMeritSelectorInitialized() {
    const sel = document.getElementById('jobMeritJobSelector');
    if (sel.options.length === 0) {
        JOBS.forEach(job => {
            const opt = document.createElement('option');
            opt.value = job.key;
            opt.textContent = job.name;
            sel.appendChild(opt);
        });
        sel.addEventListener('change', renderJobMeritInputs);
        ['jobMeritGroup1Grid', 'jobMeritGroup2Grid'].forEach(gridId => {
            document.getElementById(gridId).addEventListener('input', (e) => {
                if (!e.target.matches('input[data-merit-group]')) return;
                const group = e.target.dataset.meritGroup;
                const idx = parseInt(e.target.dataset.meritIndex, 10);
                let v = parseInt(e.target.value, 10);
                if (isNaN(v)) v = 0;
                v = Math.max(0, Math.min(JOB_MERIT_MAX_RANK, v));
                const jobKey = document.getElementById('jobMeritJobSelector').value;
                if (!currentEditingJobMerits[jobKey]) {
                    currentEditingJobMerits[jobKey] = {
                        group1: jobMeritDefaultRanks(),
                        group2: jobMeritDefaultRanks(),
                    };
                }
                // グループ合計が JOB_MERIT_GROUP_MAX_TOTAL を超えないようにクランプ
                const ranks = currentEditingJobMerits[jobKey][group];
                const otherSum = ranks.reduce((s, r, i) => s + (i === idx ? 0 : (r || 0)), 0);
                const allowed = Math.max(0, JOB_MERIT_GROUP_MAX_TOTAL - otherSum);
                if (v > allowed) v = allowed;
                ranks[idx] = v;
                if (parseInt(e.target.value, 10) !== v) {
                    e.target.value = v;
                }
                updateJobMeritGroupTotalDisplay();
            });
        });
    }
}

// キャラクターから JP データをロード（未定義の場合は全振りデフォルト）
function loadJobPointsFromCharacter(character) {
    const result = {};
    const storedCategories = character && character.job_points && character.job_points.categories || {};
    JOBS.forEach(job => {
        const stored = storedCategories[job.key];
        if (stored && Array.isArray(stored.ranks) && stored.ranks.length === JP_CATEGORY_COUNT) {
            result[job.key] = stored.ranks.slice();
        } else {
            result[job.key] = jpDefaultRanks();
        }
    });
    return result;
}

// 現在選択中のジョブに対応するカテゴリ入力を再描画
export function renderJpCategoryInputs() {
    const jobKey = document.getElementById('jpJobSelector').value;
    const grid = document.getElementById('jpCategoryGrid');
    const names = JP_CATEGORIES[jobKey] || new Array(JP_CATEGORY_COUNT).fill('').map((_, i) => `カテゴリ${i + 1}`);
    const ranks = currentEditingJobPoints[jobKey] || jpDefaultRanks();
    grid.innerHTML = '';
    for (let i = 0; i < JP_CATEGORY_COUNT; i++) {
        const item = document.createElement('div');
        item.className = 'jp-item';
        item.innerHTML = `
            <label for="jp_cat_${i}">${names[i] || `カテゴリ${i + 1}`}</label>
            <input type="number" id="jp_cat_${i}" min="0" max="${JP_MAX_RANK}" value="${ranks[i]}" data-cat-index="${i}">
        `;
        grid.appendChild(item);
    }
    updateJpTotalDisplay();
}

function updateJpTotalDisplay() {
    const jobKey = document.getElementById('jpJobSelector').value;
    const ranks = currentEditingJobPoints[jobKey] || jpDefaultRanks();
    document.getElementById('jpTotalDisplay').textContent = jpJobTotal(ranks);
}

// ジョブセレクタ初期化（1 回のみ）
export function ensureJpJobSelectorInitialized() {
    const sel = document.getElementById('jpJobSelector');
    if (sel.options.length === 0) {
        JOBS.forEach(job => {
            const opt = document.createElement('option');
            opt.value = job.key;
            opt.textContent = job.name;
            sel.appendChild(opt);
        });
        sel.addEventListener('change', renderJpCategoryInputs);
        // カテゴリ入力の変更を委任でハンドル
        document.getElementById('jpCategoryGrid').addEventListener('input', (e) => {
            if (e.target.matches('input[data-cat-index]')) {
                const idx = parseInt(e.target.dataset.catIndex, 10);
                let v = parseInt(e.target.value, 10);
                if (isNaN(v)) v = 0;
                v = Math.max(0, Math.min(JP_MAX_RANK, v));
                const jobKey = document.getElementById('jpJobSelector').value;
                if (!currentEditingJobPoints[jobKey]) {
                    currentEditingJobPoints[jobKey] = jpDefaultRanks();
                }
                currentEditingJobPoints[jobKey][idx] = v;
                updateJpTotalDisplay();
            }
        });
    }
}

function computeSkillDefaults(characterLike) {
    // characterLike: { race, job_levels, merit_points } で十分
    const basicProfile = {
        name: characterLike.name || '_temp',
        race: characterLike.race,
        job_levels: characterLike.job_levels,
        merit_points: characterLike.merit_points || {
            hp: 0, mp: 0, str_: 0, dex: 0, vit: 0, agi: 0, int: 0, mnd: 0, chr: 0,
        }
    };
    return calculate_default_skills(basicProfile);
}

export function renderMeritSkillInputs() {
    const groups = [
        { gridId: 'meritSkillGridWeapon', keys: SKILL_KEYS_WEAPON },
        { gridId: 'meritSkillGridDefense', keys: SKILL_KEYS_DEFENSE },
        { gridId: 'meritSkillGridMagic', keys: MAGIC_SKILL_KEYS },
    ];
    groups.forEach(({ gridId, keys }) => {
        const grid = document.getElementById(gridId);
        grid.innerHTML = '';
        keys.forEach(([key, ja]) => {
            const value = currentEditingMeritSkills[key] != null ? currentEditingMeritSkills[key] : 8;
            const item = document.createElement('div');
            item.className = 'merit-item';
            item.innerHTML = `
                <label for="meritSkill_${key}">${ja}</label>
                <input type="number" id="meritSkill_${key}" min="0" max="8" value="${value}" data-merit-skill-key="${key}">
            `;
            grid.appendChild(item);
        });
    });
}

export function renderSkillInputs() {
    const groups = [
        { gridId: 'skillGridWeapon', keys: SKILL_KEYS_WEAPON },
        { gridId: 'skillGridDefense', keys: SKILL_KEYS_DEFENSE },
        { gridId: 'skillGridMagic', keys: SKILL_KEYS_MAGIC },
    ];
    groups.forEach(({ gridId, keys }) => {
        const grid = document.getElementById(gridId);
        grid.innerHTML = '';
        keys.forEach(([key, ja]) => {
            const cap = currentEditingSkillDefaults[key] || 0;
            const value = currentEditingSkills[key] != null ? currentEditingSkills[key] : cap;
            const item = document.createElement('div');
            item.className = 'skill-item';
            item.innerHTML = `
                <label for="skill_${key}">
                    <span>${ja}</span>
                    <span class="skill-cap">上限 ${cap}</span>
                </label>
                <input type="number" id="skill_${key}" min="0" value="${value}" data-skill-key="${key}">
            `;
            grid.appendChild(item);
        });
    });
}

export function ensureSkillInputListenerInitialized() {
    const btn = document.getElementById('btnResetSkillsToDefault');
    if (!btn.dataset.bound) {
        btn.dataset.bound = '1';
        btn.addEventListener('click', () => {
            recomputeSkillDefaultsFromForm();
            ALL_SKILL_KEYS.forEach(([k]) => {
                currentEditingSkills[k] = currentEditingSkillDefaults[k] || 0;
            });
            renderSkillInputs();
        });
    }
    const containers = ['skillGridWeapon', 'skillGridDefense', 'skillGridMagic'];
    containers.forEach(id => {
        const el = document.getElementById(id);
        if (!el.dataset.bound) {
            el.dataset.bound = '1';
            el.addEventListener('input', (e) => {
                if (e.target.matches('input[data-skill-key]')) {
                    const k = e.target.dataset.skillKey;
                    let v = parseInt(e.target.value, 10);
                    if (isNaN(v) || v < 0) v = 0;
                    currentEditingSkills[k] = v;
                }
            });
        }
    });
    // ジョブレベル/マスターレベルの変更時にスキル上限を再計算し、
    // 旧デフォルトと同値のスキルは新デフォルトに追従させる
    const jobLevelTable = document.getElementById('jobLevelTable');
    if (jobLevelTable && !jobLevelTable.dataset.skillBound) {
        jobLevelTable.dataset.skillBound = '1';
        jobLevelTable.addEventListener('input', (e) => {
            if (e.target.matches('input[id^="jl_"]')) {
                const oldDefaults = { ...currentEditingSkillDefaults };
                recomputeSkillDefaultsFromForm();
                ALL_SKILL_KEYS.forEach(([k]) => {
                    // ユーザー未カスタム（旧デフォルトと同値）のみ追従
                    if (currentEditingSkills[k] === oldDefaults[k]) {
                        currentEditingSkills[k] = currentEditingSkillDefaults[k] || 0;
                    }
                });
                renderSkillInputs();
            }
        });
    }
    // スキルメリット変更時にスキル上限を再計算
    ['meritSkillGridWeapon', 'meritSkillGridDefense', 'meritSkillGridMagic'].forEach(id => {
        const el = document.getElementById(id);
        if (el && !el.dataset.skillBound) {
            el.dataset.skillBound = '1';
            el.addEventListener('input', (e) => {
                if (e.target.matches('input[data-merit-skill-key]')) {
                    const k = e.target.dataset.meritSkillKey;
                    let v = parseInt(e.target.value, 10);
                    if (isNaN(v) || v < 0) v = 0;
                    if (v > 8) v = 8;
                    currentEditingMeritSkills[k] = v;
                    const oldDefaults = { ...currentEditingSkillDefaults };
                    recomputeSkillDefaultsFromForm();
                    ALL_SKILL_KEYS.forEach(([sk]) => {
                        if (currentEditingSkills[sk] === oldDefaults[sk]) {
                            currentEditingSkills[sk] = currentEditingSkillDefaults[sk] || 0;
                        }
                    });
                    renderSkillInputs();
                }
            });
        }
    });
}

export function readMeritPointsFromForm() {
    const combat_skill_merits = {};
    COMBAT_SKILL_KEYS.forEach(([k]) => { combat_skill_merits[k] = currentEditingMeritSkills[k] || 0; });
    const magic_skill_merits = {};
    MAGIC_SKILL_KEYS.forEach(([k]) => { magic_skill_merits[k] = currentEditingMeritSkills[k] || 0; });
    const job_merits = {};
    JOBS.forEach(job => {
        const m = currentEditingJobMerits[job.key] || { group1: jobMeritDefaultRanks(), group2: jobMeritDefaultRanks() };
        job_merits[job.key] = { group1: m.group1.slice(), group2: m.group2.slice() };
    });
    return {
        hp: parseInt(document.getElementById('charMeritHp').value) || 0,
        mp: parseInt(document.getElementById('charMeritMp').value) || 0,
        str_: parseInt(document.getElementById('charMeritStr').value) || 0,
        dex: parseInt(document.getElementById('charMeritDex').value) || 0,
        vit: parseInt(document.getElementById('charMeritVit').value) || 0,
        agi: parseInt(document.getElementById('charMeritAgi').value) || 0,
        int: parseInt(document.getElementById('charMeritInt').value) || 0,
        mnd: parseInt(document.getElementById('charMeritMnd').value) || 0,
        chr: parseInt(document.getElementById('charMeritChr').value) || 0,
        combat_skill_merits,
        magic_skill_merits,
        enmity_plus: parseInt(document.getElementById('charMeritEnmityPlus').value) || 0,
        enmity_minus: parseInt(document.getElementById('charMeritEnmityMinus').value) || 0,
        critical_hit_rate: parseInt(document.getElementById('charMeritCriticalHitRate').value) || 0,
        enemy_critical_hit_rate: parseInt(document.getElementById('charMeritEnemyCriticalHitRate').value) || 0,
        spell_interruption_rate: parseInt(document.getElementById('charMeritSpellInterruptionRate').value) || 0,
        store_tp: (job_merits.Sam && job_merits.Sam.group1[samStoreTpIndex()]) || 0,
        job_merits,
    };
}

// 編集フォームの現状ジョブレベル / ML から skill defaults を再計算
function recomputeSkillDefaultsFromForm() {
    const job_levels = {};
    JOBS.forEach(job => {
        const lv = parseInt(document.getElementById(`jl_${job.key}_lv`).value) || 0;
        const mlv = parseInt(document.getElementById(`jl_${job.key}_mlv`).value) || 0;
        job_levels[job.key] = { level: lv, master_lv: mlv };
    });
    const race = document.getElementById('charRace').value;
    const merit_points = readMeritPointsFromForm();
    currentEditingSkillDefaults = computeSkillDefaults({ race, job_levels, merit_points });
}

// ===== seam: 呼び出し側 (キャラクター一覧・保存) との受け渡し =====

// 既存キャラクターの内容を編集中状態へ読み込む (フォームのメリット系 DOM 反映含む)
export function setFormStateFromCharacter(character) {
    const mp = character.merit_points || {};
    // スキルメリット: 既存値を読み込み、未設定は 8
    const csm = mp.combat_skill_merits || {};
    const msm = mp.magic_skill_merits || {};
    currentEditingMeritSkills = {};
    COMBAT_SKILL_KEYS.forEach(([k]) => { currentEditingMeritSkills[k] = csm[k] != null ? csm[k] : 8; });
    MAGIC_SKILL_KEYS.forEach(([k]) => { currentEditingMeritSkills[k] = msm[k] != null ? msm[k] : 8; });
    document.getElementById('charMeritEnmityPlus').value = mp.enmity_plus || 0;
    document.getElementById('charMeritEnmityMinus').value = mp.enmity_minus || 0;
    document.getElementById('charMeritCriticalHitRate').value = mp.critical_hit_rate || 0;
    document.getElementById('charMeritEnemyCriticalHitRate').value = mp.enemy_critical_hit_rate || 0;
    document.getElementById('charMeritSpellInterruptionRate').value = mp.spell_interruption_rate || 0;
    currentEditingJobMerits = loadJobMeritsFromCharacter(character);
    // 旧フォーマットからの移行: merit_points.store_tp を SAM G1 ストアTP に backfill
    if (mp.store_tp && currentEditingJobMerits.Sam) {
        const idx = samStoreTpIndex();
        if (idx >= 0 && !currentEditingJobMerits.Sam.group1[idx]) {
            currentEditingJobMerits.Sam.group1[idx] = mp.store_tp;
        }
    }
    currentEditingJobPoints = loadJobPointsFromCharacter(character);
    // スキル: デフォルト（キャップ最大）を算出して、既存値があればマージ
    currentEditingSkillDefaults = computeSkillDefaults(character);
    currentEditingSkills = {};
    const storedValues = (character.skills && character.skills.values) || null;
    ALL_SKILL_KEYS.forEach(([k]) => {
        if (storedValues && storedValues[k] != null) {
            currentEditingSkills[k] = storedValues[k];
        } else {
            currentEditingSkills[k] = currentEditingSkillDefaults[k] || 0;
        }
    });
}

// 新規キャラクター向けの編集中状態を作る (フォームのメリット系 DOM 反映含む)
export function resetFormStateForNew(defaultJobLevels) {
    currentEditingMeritSkills = {};
    ALL_SKILL_KEYS.forEach(([k]) => { currentEditingMeritSkills[k] = 8; });
    document.getElementById('charMeritEnmityPlus').value = 0;
    document.getElementById('charMeritEnmityMinus').value = 0;
    document.getElementById('charMeritCriticalHitRate').value = 0;
    document.getElementById('charMeritEnemyCriticalHitRate').value = 0;
    document.getElementById('charMeritSpellInterruptionRate').value = 0;
    currentEditingJobMerits = {};
    JOBS.forEach(job => { currentEditingJobMerits[job.key] = { group1: jobMeritDefaultRanks(), group2: jobMeritDefaultRanks() }; });
    currentEditingJobPoints = {};
    JOBS.forEach(job => { currentEditingJobPoints[job.key] = jpDefaultRanks(); });
    // 新規キャラ: 全ジョブ 99 の前提でデフォルトスキル値を算出
    currentEditingSkillDefaults = computeSkillDefaults({
        race: 'Hum',
        job_levels: defaultJobLevels,
    });
    currentEditingSkills = {};
    ALL_SKILL_KEYS.forEach(([k]) => {
        currentEditingSkills[k] = currentEditingSkillDefaults[k] || 0;
    });
}

// 保存用に編集中状態から job_points / skills を組み立てる
export function collectFormStateForSave() {
    // JP: 全ジョブの振り分け状況を保存
    const job_points_categories = {};
    JOBS.forEach(job => {
        const ranks = currentEditingJobPoints[job.key] || jpDefaultRanks();
        job_points_categories[job.key] = { ranks: ranks.slice() };
    });
    const job_points = { categories: job_points_categories };

    // スキル: 全33スキルの現在値を保存
    const skill_values = {};
    ALL_SKILL_KEYS.forEach(([k]) => {
        skill_values[k] = currentEditingSkills[k] != null ? currentEditingSkills[k] : 0;
    });
    const skills = { values: skill_values };

    return { job_points, skills };
}
