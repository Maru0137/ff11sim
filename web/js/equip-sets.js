// 装備セット単位の管理 UI。タブ表示 (ドラッグ&ドロップ並べ替え含む)・
// 編集フォームへの読み込み・保存・複製・削除と、キャラクター/ジョブ/
// サポートジョブのセレクタ配線を持つ。
//
// saveEquipSet → renderEquipSetTabs → selectEquipSetTab → showEquipSetEditForm
// が相互に呼び合うため、この一連は 1 モジュールに保つこと。
import { get_item_by_id } from './wasm.js';
import { JOBS, RACE_NAMES, EQUIPMENT_SLOTS } from './constants.js';
import { loadCharacters, loadEquipSets, saveEquipSets } from './storage.js';
import { equipState } from './equip-state.js';
import { createEmptySlots } from './equip-slots.js';
import { updateAugPathOptions } from './augments.js';
import { updateEquipEditStatus } from '../src/status/status-store';

// Render equipment set tabs for selected character+job
export async function renderEquipSetTabs() {
    const tabBar = document.getElementById('equipSetTabBar');
    tabBar.innerHTML = '';

    if (!equipState.currentEquipChar || !equipState.currentEquipJob) {
        tabBar.classList.add('hidden');
        hideEquipSetEditForm();
        return;
    }

    tabBar.classList.remove('hidden');

    const sets = (await loadEquipSets()).filter(
        s => s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob
    );

    sets.forEach(es => {
        const tab = document.createElement('button');
        tab.className = 'equipset-tab';
        tab.textContent = es.name;
        tab.dataset.equipsetName = es.name;
        tab.draggable = true;
        tab.addEventListener('click', () => {
            selectEquipSetTab(es.name);
        });
        tab.addEventListener('dragstart', (e) => {
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', es.name);
            tab.classList.add('dragging');
        });
        tab.addEventListener('dragend', () => {
            tab.classList.remove('dragging');
        });
        tab.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = 'move';
            tab.classList.add('drag-over');
        });
        tab.addEventListener('dragleave', () => {
            tab.classList.remove('drag-over');
        });
        tab.addEventListener('drop', async (e) => {
            e.preventDefault();
            tab.classList.remove('drag-over');
            const fromName = e.dataTransfer.getData('text/plain');
            const toName = es.name;
            if (fromName && fromName !== toName) {
                await reorderEquipSetTabs(fromName, toName);
            }
        });
        tabBar.appendChild(tab);
    });

    // "+" tab for new set
    const addTab = document.createElement('button');
    addTab.className = 'equipset-tab-add';
    addTab.textContent = '+';
    addTab.addEventListener('click', () => {
        showNewEquipSetForm();
    });
    tabBar.appendChild(addTab);

    // Auto-select first tab if any sets exist
    if (sets.length > 0) {
        await selectEquipSetTab(sets[0].name);
    } else {
        hideEquipSetEditForm();
    }
}

export async function selectEquipSetTab(name) {
    equipState.isNewEquipSet = false;

    // Update tab active state
    document.querySelectorAll('.equipset-tab').forEach(t => {
        t.classList.toggle('active', t.dataset.equipsetName === name);
    });

    const sets = await loadEquipSets();
    const es = sets.find(
        s => s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob && s.name === name
    );
    if (es) {
        showEquipSetEditForm(es);
    }
}

function showNewEquipSetForm() {
    equipState.isNewEquipSet = true;

    // Deactivate all tabs
    document.querySelectorAll('.equipset-tab').forEach(t => t.classList.remove('active'));

    showEquipSetEditForm(null);
}

export function showEquipSetEditForm(equipSet) {
    const section = document.getElementById('equipEditSection');
    section.classList.remove('hidden');

    const deleteBtn = document.getElementById('btnDeleteEquipSet');

    if (equipSet) {
        equipState.editingEquipSetName = equipSet.name;
        equipState.currentEquipSlots = { ...equipSet.slots };
        // 旧データは skill フィールドを持たないので item_id から補完
        Object.keys(equipState.currentEquipSlots).forEach(k => {
            const d = equipState.currentEquipSlots[k];
            if (d && d.skill == null && d.item_id != null) {
                const item = get_item_by_id(d.item_id);
                if (item && item.skill != null) d.skill = item.skill;
            }
        });
        document.getElementById('equipSetName').value = equipSet.name;
        deleteBtn.classList.remove('hidden');

        EQUIPMENT_SLOTS.forEach(slot => {
            const data = equipState.currentEquipSlots[slot.key];
            const input = document.querySelector(`.equip-slot-search[data-slot="${slot.key}"]`);
            const descDiv = document.querySelector(`.equip-slot-description[data-slot="${slot.key}"]`);
            const customDescInput = document.querySelector(`.equip-slot-custom-desc[data-slot="${slot.key}"]`);
            if (input) {
                input.value = data ? (data.name_ja || data.name_en || '') : '';
            }
            if (descDiv) {
                const raw = data ? (data.description_ja || '') : '';
                descDiv.innerHTML = raw.replace(/\\n/g, '<br>');
            }
            if (customDescInput) {
                customDescInput.value = data ? (data.custom_description || '') : '';
            }
            updateAugPathOptions(slot.key);
        });
    } else {
        equipState.editingEquipSetName = null;
        equipState.currentEquipSlots = createEmptySlots();
        document.getElementById('equipSetName').value = '';
        deleteBtn.classList.add('hidden');

        EQUIPMENT_SLOTS.forEach(slot => {
            const input = document.querySelector(`.equip-slot-search[data-slot="${slot.key}"]`);
            const descDiv = document.querySelector(`.equip-slot-description[data-slot="${slot.key}"]`);
            const customDescInput = document.querySelector(`.equip-slot-custom-desc[data-slot="${slot.key}"]`);
            if (input) input.value = '';
            if (descDiv) descDiv.textContent = '';
            if (customDescInput) customDescInput.value = '';
            updateAugPathOptions(slot.key);
        });
    }

    updateEquipEditStatus();
}

export function hideEquipSetEditForm() {
    document.getElementById('equipEditSection').classList.add('hidden');
    equipState.editingEquipSetName = null;
    equipState.isNewEquipSet = false;
}

async function saveEquipSet() {
    const name = document.getElementById('equipSetName').value.trim();
    if (!name) {
        alert('装備セット名を入力してください。');
        return;
    }

    const sets = await loadEquipSets();
    const character = equipState.currentEquipChar;
    const job = equipState.currentEquipJob;

    if (equipState.editingEquipSetName) {
        // Editing existing set
        const idx = sets.findIndex(
            s => s.character === character && s.job === job && s.name === equipState.editingEquipSetName
        );
        if (idx >= 0) {
            // Check name uniqueness if renamed
            if (name !== equipState.editingEquipSetName) {
                if (sets.some(s => s.character === character && s.job === job && s.name === name)) {
                    alert(`装備セット「${name}」は既に存在します。`);
                    return;
                }
            }
            sets[idx] = { name, job, character, slots: { ...equipState.currentEquipSlots } };
        }
    } else {
        // Creating new set
        if (sets.some(s => s.character === character && s.job === job && s.name === name)) {
            alert(`装備セット「${name}」は既に存在します。`);
            return;
        }
        sets.push({ name, job, character, slots: { ...equipState.currentEquipSlots } });
    }

    await saveEquipSets(sets);
    await renderEquipSetTabs();
    // Select the saved set
    await selectEquipSetTab(name);
}

async function reorderEquipSetTabs(fromName, toName) {
    // 現在の (character, job) 内で fromName を toName の位置に移動。
    // 他の (character, job) のセットには触れずに save。
    const allSets = await loadEquipSets();
    const inScope = (s) => s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob;
    const tabs = allSets.filter(inScope);
    const others = allSets.filter((s) => !inScope(s));

    const fromIdx = tabs.findIndex((s) => s.name === fromName);
    const toIdx = tabs.findIndex((s) => s.name === toName);
    if (fromIdx < 0 || toIdx < 0) return;

    const [moved] = tabs.splice(fromIdx, 1);
    tabs.splice(toIdx, 0, moved);

    // 順序統合 (other は元の位置を維持できないので末尾に集める。
    //  Local 側は順序を厳密維持しないため許容、Supabase 側は (char, job) 単位で position 管理)
    await saveEquipSets([...others, ...tabs]);
    await renderEquipSetTabs();
    await selectEquipSetTab(moved.name);
}

async function copyEquipSet() {
    if (!equipState.currentEquipChar || !equipState.currentEquipJob) return;
    // 編集中のセット名をベースに、未使用の名前を生成
    const baseName = equipState.editingEquipSetName || document.getElementById('equipSetName').value.trim();
    if (!baseName) {
        alert('複製元の装備セットがありません。');
        return;
    }
    const sets = await loadEquipSets();
    const inScope = sets.filter(s => s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob);
    const usedNames = new Set(inScope.map(s => s.name));
    let newName = `${baseName} のコピー`;
    let i = 2;
    while (usedNames.has(newName)) {
        newName = `${baseName} のコピー ${i++}`;
    }
    sets.push({
        name: newName,
        job: equipState.currentEquipJob,
        character: equipState.currentEquipChar,
        slots: { ...equipState.currentEquipSlots },
    });
    await saveEquipSets(sets);
    await renderEquipSetTabs();
    await selectEquipSetTab(newName);
}

async function deleteEquipSet() {
    if (!equipState.editingEquipSetName) return;
    if (!confirm(`装備セット「${equipState.editingEquipSetName}」を削除しますか？`)) return;

    const sets = (await loadEquipSets()).filter(
        s => !(s.character === equipState.currentEquipChar && s.job === equipState.currentEquipJob && s.name === equipState.editingEquipSetName)
    );
    await saveEquipSets(sets);
    equipState.editingEquipSetName = null;
    await renderEquipSetTabs();
}

function updateSupportJobOptions() {
    const supportJobSel = document.getElementById('equipSelectSupportJob');
    if (!equipState.currentEquipJob) {
        supportJobSel.disabled = true;
        supportJobSel.innerHTML = '<option value="">-- なし --</option>';
        equipState.currentEquipSupportJob = '';
        return;
    }
    supportJobSel.disabled = false;
    supportJobSel.innerHTML = '<option value="">-- なし --</option>';
    for (const job of JOBS) {
        if (job.key.toLowerCase() === equipState.currentEquipJob) continue;
        const opt = document.createElement('option');
        opt.value = job.key.toLowerCase();
        opt.textContent = job.name;
        supportJobSel.appendChild(opt);
    }
    equipState.currentEquipSupportJob = '';
}

// キャラクターセレクタの選択肢を保存済みキャラクター一覧から再構築
export async function updateEquipCharSelector() {
    const sel = document.getElementById('equipSelectChar');
    const currentVal = sel.value;
    sel.innerHTML = '<option value="">-- キャラクターを選択 --</option>';
    const characters = await loadCharacters();
    characters.forEach(ch => {
        const opt = document.createElement('option');
        opt.value = ch.name;
        opt.textContent = `${ch.name} (${RACE_NAMES[ch.race] || ch.race})`;
        sel.appendChild(opt);
    });
    if (currentVal && [...sel.options].some(o => o.value === currentVal)) {
        sel.value = currentVal;
    }
}

// セレクタと保存/複製/削除ボタンの配線
export function initEquipSetControls() {
    // Character selector change
    document.getElementById('equipSelectChar').addEventListener('change', (e) => {
        equipState.currentEquipChar = e.target.value;
        const jobSel = document.getElementById('equipSelectJob');
        const supportJobSel = document.getElementById('equipSelectSupportJob');
        if (equipState.currentEquipChar) {
            jobSel.disabled = false;
        } else {
            jobSel.disabled = true;
            jobSel.value = '';
            equipState.currentEquipJob = '';
            supportJobSel.disabled = true;
            supportJobSel.innerHTML = '<option value="">-- なし --</option>';
            equipState.currentEquipSupportJob = '';
        }
        renderEquipSetTabs();
    });

    // Job selector change
    document.getElementById('equipSelectJob').addEventListener('change', (e) => {
        equipState.currentEquipJob = e.target.value;
        updateSupportJobOptions();
        renderEquipSetTabs();
    });

    // Support job selector change
    document.getElementById('equipSelectSupportJob').addEventListener('change', (e) => {
        equipState.currentEquipSupportJob = e.target.value;
        updateEquipEditStatus();
    });

    document.getElementById('btnSaveEquipSet').addEventListener('click', saveEquipSet);
    document.getElementById('btnCopyEquipSet').addEventListener('click', copyEquipSet);
    document.getElementById('btnDeleteEquipSet').addEventListener('click', deleteEquipSet);
}
