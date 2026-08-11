// 装備スロット行 UI。16 部位 (data/equipment_slots.json) それぞれの
// 検索入力・候補ドロップダウン・選択/クリア・オーグメント選択・カスタム説明を
// 生成して配線する。行の DOM は data-slot 属性で外部 (equip-sets 等) から参照される。
import { search_items, isItemsLoaded } from './wasm.js';
import { EQUIPMENT_SLOTS } from './constants.js';
import { equipState } from './equip-state.js';
import { updateAugPathOptions, updateAugTextDisplay } from './augments.js';
import { updateEquipEditStatus } from './equip-status.js';
import { openCustomAugHelp } from '../src/modals/modal-store';

// 装備データと検索は Rust 側に移した (docs/adr/0009, docs/adr/0010)。
// items.json は WASM に埋め込まれているので JS からは読まない。
const itemSearch = { search: (o) => search_items(o) };

export function createEmptySlots() {
    const slots = {};
    EQUIPMENT_SLOTS.forEach(s => { slots[s.key] = null; });
    return slots;
}

export function buildEquipSlotsUI() {
    const container = document.getElementById('equipSlotsContainer');
    container.innerHTML = '';

    const headerEl = document.createElement('div');
    headerEl.className = 'equip-slot-header';
    const headerLabels = ['スロット', '装備名', '', '説明', 'オーグメント', 'Aug説明', 'カスタム'];
    headerLabels.forEach(text => {
        const span = document.createElement('span');
        span.textContent = text;
        if (text === 'カスタム') {
            const helpBtn = document.createElement('button');
            helpBtn.type = 'button';
            helpBtn.className = 'aug-help-btn';
            helpBtn.textContent = '?';
            helpBtn.title = 'カスタムオーグメント欄の書き方を表示';
            helpBtn.addEventListener('click', () => {
                openCustomAugHelp();
            });
            span.appendChild(helpBtn);
        }
        headerEl.appendChild(span);
    });
    container.appendChild(headerEl);

    EQUIPMENT_SLOTS.forEach(slot => {
        const row = document.createElement('div');
        row.className = 'equip-slot-row';

        const label = document.createElement('span');
        label.className = 'equip-slot-label';
        label.textContent = slot.label;

        const inputWrapper = document.createElement('div');
        inputWrapper.className = 'equip-slot-input-wrapper';

        const input = document.createElement('input');
        input.type = 'text';
        input.className = 'equip-slot-search';
        input.dataset.slot = slot.key;
        input.placeholder = '検索...';

        const dropdown = document.createElement('div');
        dropdown.className = 'equip-slot-dropdown hidden';
        dropdown.dataset.slot = slot.key;

        const arrow = document.createElement('span');
        arrow.className = 'equip-slot-arrow';
        arrow.textContent = '▼';

        inputWrapper.appendChild(input);
        inputWrapper.appendChild(arrow);
        inputWrapper.appendChild(dropdown);

        const selected = document.createElement('span');
        selected.className = 'equip-slot-selected';
        selected.dataset.slot = slot.key;
        selected.textContent = '';

        const clearBtn = document.createElement('button');
        clearBtn.className = 'equip-slot-clear';
        clearBtn.dataset.slot = slot.key;
        clearBtn.textContent = '×';

        const description = document.createElement('div');
        description.className = 'equip-slot-description';
        description.dataset.slot = slot.key;
        description.textContent = '';

        const customDescInput = document.createElement('input');
        customDescInput.type = 'text';
        customDescInput.className = 'equip-slot-custom-desc';
        customDescInput.dataset.slot = slot.key;
        customDescInput.placeholder = '例: STR+5 命中+10 ヘイスト+3%';

        const augContainer = document.createElement('div');
        augContainer.className = 'equip-slot-aug-container';
        augContainer.dataset.slot = slot.key;

        const augPathSelect = document.createElement('select');
        augPathSelect.className = 'equip-slot-aug-path';
        augPathSelect.dataset.slot = slot.key;

        const augTextDiv = document.createElement('div');
        augTextDiv.className = 'equip-slot-aug-text';
        augTextDiv.dataset.slot = slot.key;

        augContainer.appendChild(augPathSelect);
        augContainer.appendChild(augTextDiv);

        row.appendChild(label);
        row.appendChild(inputWrapper);
        row.appendChild(selected);
        row.appendChild(clearBtn);
        row.appendChild(description);
        row.appendChild(augContainer);
        row.appendChild(customDescInput);
        container.appendChild(row);

        setupSlotSearch(slot.key, input, dropdown);

        augPathSelect.addEventListener('change', () => {
            if (equipState.currentEquipSlots[slot.key]) {
                const val = augPathSelect.value;
                if (val) {
                    const [pathIdx, rank] = val.split('-').map(Number);
                    equipState.currentEquipSlots[slot.key].aug_path = pathIdx;
                    equipState.currentEquipSlots[slot.key].aug_rank = rank;
                } else {
                    equipState.currentEquipSlots[slot.key].aug_path = null;
                    equipState.currentEquipSlots[slot.key].aug_rank = null;
                }
                updateAugTextDisplay(slot.key);
                updateEquipEditStatus();
            }
        });

        customDescInput.addEventListener('input', () => {
            if (equipState.currentEquipSlots[slot.key]) {
                equipState.currentEquipSlots[slot.key].custom_description = customDescInput.value;
                updateEquipEditStatus();
            }
        });

        clearBtn.addEventListener('click', () => {
            equipState.currentEquipSlots[slot.key] = null;
            selected.textContent = '';
            input.value = '';
            description.textContent = '';
            customDescInput.value = '';
            updateAugPathOptions(slot.key);
            updateEquipEditStatus();
        });
    });

}

function setupSlotSearch(slotKey, input, dropdown) {
    let debounceTimer = null;

    function doSearch() {
        if (!isItemsLoaded()) {
            dropdown.classList.add('hidden');
            return;
        }
        const query = input.value.trim();
        const results = itemSearch.search({
            query: query,
            slot: slotKey,
            job: equipState.currentEquipJob ? equipState.currentEquipJob.toUpperCase() : '',
            sortBy: 'id',
            sortOrder: 'desc',
            limit: 30,
        });
        renderSlotDropdown(dropdown, results.items, slotKey, input);
    }

    input.addEventListener('input', () => {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(doSearch, 150);
    });

    input.addEventListener('blur', () => {
        setTimeout(() => dropdown.classList.add('hidden'), 200);
    });

    input.addEventListener('focus', () => {
        doSearch();
    });
}

function renderSlotDropdown(dropdown, items, slotKey, input) {
    dropdown.innerHTML = '';
    if (items.length === 0) {
        dropdown.classList.add('hidden');
        return;
    }
    items.forEach(item => {
        const div = document.createElement('div');
        div.className = 'equip-slot-dropdown-item';
        div.textContent = item.ja || item.en;
        div.addEventListener('mousedown', (e) => {
            e.preventDefault();
            selectSlotItem(slotKey, item);
            dropdown.classList.add('hidden');
        });
        dropdown.appendChild(div);
    });
    dropdown.classList.remove('hidden');
}

function selectSlotItem(slotKey, item) {
    equipState.currentEquipSlots[slotKey] = {
        item_id: item.id,
        name_en: item.en,
        name_ja: item.ja,
        description_ja: item.description_ja || '',
        skill: item.skill != null ? item.skill : null,
        aug_path: null,
        aug_rank: null,
    };
    const input = document.querySelector(`.equip-slot-search[data-slot="${slotKey}"]`);
    if (input) {
        input.value = item.ja || item.en;
    }
    const descDiv = document.querySelector(`.equip-slot-description[data-slot="${slotKey}"]`);
    if (descDiv) {
        const raw = item.description_ja || item.description_en || '';
        descDiv.innerHTML = raw.replace(/\\n/g, '<br>');
    }
    const customDescInput = document.querySelector(`.equip-slot-custom-desc[data-slot="${slotKey}"]`);
    if (customDescInput) {
        customDescInput.value = '';
    }
    updateAugPathOptions(slotKey);
    updateEquipEditStatus();
}
