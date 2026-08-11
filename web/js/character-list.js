// キャラクター管理タブ。一覧表示・編集フォームの開閉・保存・削除を持つ。
// フォーム部品と編集中状態は js/character-form.js、永続化は js/storage.js。
import { JOBS, RACE_NAMES } from './constants.js';
import { loadCharacters, saveCharacters } from './storage.js';
import {
    ensureJobMeritSelectorInitialized, ensureJpJobSelectorInitialized,
    ensureSkillInputListenerInitialized,
    renderJobMeritInputs, renderJpCategoryInputs,
    renderMeritSkillInputs, renderSkillInputs,
    readMeritPointsFromForm,
    setFormStateFromCharacter, resetFormStateForNew, collectFormStateForSave,
} from './character-form.js';
import { reloadCharacters } from '../src/equip/equip-sets-store';

let editingCharName = null;

export function buildJobLevelTable() {
    const tbody = document.getElementById('jobLevelTable');
    tbody.innerHTML = '';
    JOBS.forEach(job => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
            <td class="job-name">${job.key}</td>
            <td><input type="number" id="jl_${job.key}_lv" min="0" max="99" value="99"></td>
            <td><input type="number" id="jl_${job.key}_mlv" min="0" max="50" value="0"></td>
        `;
        tbody.appendChild(tr);
    });
}

export async function renderCharList() {
    const characters = await loadCharacters();
    const list = document.getElementById('charList');
    if (characters.length === 0) {
        list.innerHTML = '<li class="empty-msg">キャラクターが登録されていません</li>';
    } else {
        list.innerHTML = '';
        characters.forEach(ch => {
            const li = document.createElement('li');
            li.innerHTML = `
                <span>
                    <span class="char-info">${ch.name}</span>
                    <span class="char-race">${RACE_NAMES[ch.race] || ch.race}</span>
                </span>
                <span class="char-actions">
                    <button class="btn btn-primary btn-sm" data-edit="${ch.name}">編集</button>
                    <button class="btn btn-danger btn-sm" data-delete="${ch.name}">削除</button>
                </span>
            `;
            list.appendChild(li);
        });
    }
    // Update equipment set character selector
    await reloadCharacters();
}

function showEditForm(character) {
    const section = document.getElementById('charEditSection');
    section.classList.remove('hidden');
    ensureJpJobSelectorInitialized();
    ensureJobMeritSelectorInitialized();
    ensureSkillInputListenerInitialized();

    if (character) {
        editingCharName = character.name;
        document.getElementById('charEditTitle').textContent = 'キャラクター編集';
        document.getElementById('charName').value = character.name;
        document.getElementById('charRace').value = character.race;
        JOBS.forEach(job => {
            const jl = character.job_levels[job.key] || { level: 0, master_lv: 0 };
            document.getElementById(`jl_${job.key}_lv`).value = jl.level;
            document.getElementById(`jl_${job.key}_mlv`).value = jl.master_lv;
        });
        const mp = character.merit_points || {};
        document.getElementById('charMeritHp').value = mp.hp || 0;
        document.getElementById('charMeritMp').value = mp.mp || 0;
        document.getElementById('charMeritStr').value = mp.str_ || 0;
        document.getElementById('charMeritDex').value = mp.dex || 0;
        document.getElementById('charMeritVit').value = mp.vit || 0;
        document.getElementById('charMeritAgi').value = mp.agi || 0;
        document.getElementById('charMeritInt').value = mp.int || 0;
        document.getElementById('charMeritMnd').value = mp.mnd || 0;
        document.getElementById('charMeritChr').value = mp.chr || 0;
        setFormStateFromCharacter(character);
    } else {
        editingCharName = null;
        document.getElementById('charEditTitle').textContent = '新規キャラクター';
        document.getElementById('charName').value = '';
        document.getElementById('charRace').value = 'Hum';
        const defaultJobLevels = {};
        JOBS.forEach(job => {
            document.getElementById(`jl_${job.key}_lv`).value = 99;
            document.getElementById(`jl_${job.key}_mlv`).value = 0;
            defaultJobLevels[job.key] = { level: 99, master_lv: 0 };
        });
        ['Hp', 'Mp', 'Str', 'Dex', 'Vit', 'Agi', 'Int', 'Mnd', 'Chr'].forEach(s => {
            document.getElementById(`charMerit${s}`).value = 15;
        });
        resetFormStateForNew(defaultJobLevels);
    }
    renderMeritSkillInputs();
    renderJobMeritInputs();
    renderJpCategoryInputs();
    renderSkillInputs();
}

function hideEditForm() {
    document.getElementById('charEditSection').classList.add('hidden');
    editingCharName = null;
}

async function saveCharacter() {
    const name = document.getElementById('charName').value.trim();
    if (!name) {
        alert('キャラクター名を入力してください。');
        return;
    }

    const race = document.getElementById('charRace').value;
    const job_levels = {};
    JOBS.forEach(job => {
        const lv = parseInt(document.getElementById(`jl_${job.key}_lv`).value) || 0;
        const mlv = parseInt(document.getElementById(`jl_${job.key}_mlv`).value) || 0;
        job_levels[job.key] = { level: lv, master_lv: mlv };
    });

    const merit_points = readMeritPointsFromForm();
    const { job_points, skills } = collectFormStateForSave();

    const characters = await loadCharacters();

    if (editingCharName) {
        const idx = characters.findIndex(c => c.name === editingCharName);
        if (idx >= 0) {
            characters[idx] = { name, race, job_levels, merit_points, job_points, skills };
        }
    } else {
        if (characters.some(c => c.name === name)) {
            alert(`キャラクター「${name}」は既に存在します。`);
            return;
        }
        characters.push({ name, race, job_levels, merit_points, job_points, skills });
    }

    await saveCharacters(characters);
    await renderCharList();
    hideEditForm();
}

async function deleteCharacter(name) {
    if (!confirm(`キャラクター「${name}」を削除しますか？`)) return;
    const characters = (await loadCharacters()).filter(c => c.name !== name);
    await saveCharacters(characters);
    await renderCharList();
    if (editingCharName === name) hideEditForm();
}

export function initCharacterTab() {
    document.getElementById('charList').addEventListener('click', async (e) => {
        const editBtn = e.target.closest('[data-edit]');
        const deleteBtn = e.target.closest('[data-delete]');
        if (editBtn) {
            const name = editBtn.dataset.edit;
            const characters = await loadCharacters();
            const ch = characters.find(c => c.name === name);
            if (ch) showEditForm(ch);
        }
        if (deleteBtn) {
            await deleteCharacter(deleteBtn.dataset.delete);
        }
    });

    document.getElementById('btnNewChar').addEventListener('click', () => showEditForm(null));
    document.getElementById('btnSaveChar').addEventListener('click', saveCharacter);
    document.getElementById('btnCancelEdit').addEventListener('click', hideEditForm);
}
