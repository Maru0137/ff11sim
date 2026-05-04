// 装備編集タブの右ペイン (キャラクター × 装備セット の合計ステータス表示) を司る。
// updateEquipEditStatus は WASM 関数 / 装備合計計算関数 / 現在の編集状態 を引数で受け取る。

import { JOBS, JP_CATEGORY_COUNT, ALL_SKILL_KEYS } from './constants.js';
import { jpDefaultRanks } from './utils.js';
import { loadCharacters } from './storage.js';

// === 表示用ヘルパー (純粋) ===
const numOrDash = (v) => (v != null && v !== 0) ? v : '-';
const pctOrDash = (v) => v ? `${v}%` : '-';
const formatStatBonus = (val) => val > 0 ? `+${val}` : val < 0 ? `${val}` : '-';
const fmtPct = (v) => (v != null && v !== 0) ? `${v}%` : '-';

const SKILL_JA_MAP = Object.fromEntries(ALL_SKILL_KEYS);
function formatWeaponSkill(kind, value) {
    if (!kind || !value) return '-';
    return `${SKILL_JA_MAP[kind] || kind} (${value})`;
}

function setText(id, v) {
    const el = document.getElementById(id);
    if (el) el.textContent = v;
}

/**
 * キャラクター + 装備セットの合計ステータスを編集パネルに反映。
 * @param {object} deps
 * @param {boolean} deps.wasmReady
 * @param {boolean} deps.itemsLoaded
 * @param {string}  deps.charName              選択中キャラ名 (空文字の場合はクリア)
 * @param {string}  deps.jobKey                選択中メインジョブキー (空文字の場合はクリア)
 * @param {string|null} deps.supportJob        サブジョブキー (任意)
 * @param {object}  deps.currentEquipSlots     { slotKey: { item_id, skill, ... } | null }
 * @param {Function} deps.calculate_status_from_profile  WASM
 * @param {Function} deps.calculate_default_skills       WASM
 * @param {Function} deps.calculateEquipSetBonuses       装備合計計算 (index.html で定義)
 */
export async function updateEquipEditStatus(deps) {
    const {
        wasmReady, itemsLoaded,
        charName, jobKey, supportJob, currentEquipSlots,
        calculate_status_from_profile,
        calculate_default_skills,
        calculateEquipSetBonuses,
        // 共有閲覧モード用: 通常は loadCharacters() で charName を引くが、
        // 共有装備セットを開いた閲覧者は元のキャラクターを所有していないため、
        // 呼び出し側が直接 character オブジェクトを渡せるようにする。
        characterOverride,
    } = deps;

    if (!wasmReady || !itemsLoaded) {
        clearAllEquipStats();
        return;
    }
    if (!charName || !jobKey) {
        clearAllEquipStats();
        return;
    }

    let ch;
    if (characterOverride) {
        ch = characterOverride;
    } else {
        const characters = await loadCharacters();
        ch = characters.find(c => c.name === charName);
    }
    if (!ch) {
        clearAllEquipStats();
        return;
    }

    const jobLevel = ch.job_levels[jobKey];
    if (!jobLevel || jobLevel.level === 0) {
        clearAllEquipStats();
        return;
    }

    try {
        // JP データが未定義の場合は全振り（全 20）をデフォルトとして補完
        const jpCategories = {};
        const storedJp = (ch.job_points && ch.job_points.categories) || {};
        JOBS.forEach(job => {
            const stored = storedJp[job.key];
            if (stored && Array.isArray(stored.ranks) && stored.ranks.length === JP_CATEGORY_COUNT) {
                jpCategories[job.key] = { ranks: stored.ranks.slice() };
            } else {
                jpCategories[job.key] = { ranks: jpDefaultRanks() };
            }
        });

        // skills: 未定義時はデフォルト（全ジョブ最大）を WASM で計算
        let charSkills;
        if (ch.skills && ch.skills.values) {
            charSkills = ch.skills;
        } else {
            const basicProfile = {
                name: ch.name, race: ch.race,
                job_levels: ch.job_levels,
                merit_points: ch.merit_points,
            };
            const defaults = calculate_default_skills(basicProfile);
            charSkills = { values: defaults };
        }

        const profile = {
            name: ch.name,
            race: ch.race,
            job_levels: ch.job_levels,
            merit_points: ch.merit_points,
            job_points: { categories: jpCategories },
            skills: charSkills,
        };

        const baseStats = calculate_status_from_profile(profile, jobKey, supportJob, null);

        const tempEquipSet = { name: '_temp', slots: currentEquipSlots };
        const equip = calculateEquipSetBonuses(tempEquipSet);

        const getSlotSkillId = (slotKey) => {
            const s = currentEquipSlots && currentEquipSlots[slotKey];
            return s && s.skill != null ? s.skill : null;
        };
        const mainWeaponSkillId = getSlotSkillId('main');
        const subWeaponSkillId = getSlotSkillId('sub');
        const rangedWeaponSkillId = getSlotSkillId('range');

        const bonusStats = {
            hp: equip.hp, mp: equip.mp,
            str_: equip.str, dex: equip.dex, vit: equip.vit,
            agi: equip.agi, int: equip.int, mnd: equip.mnd, chr: equip.chr,
            def: equip.def,
            magic_def_bonus: equip.magic_def_bonus,
            evasion: equip.evasion || 0,
            magic_attack: equip.magic_attack || 0,
            attack: equip.attack || 0,
            accuracy: equip.accuracy || 0,
            ranged_attack: equip.ranged_attack || 0,
            ranged_accuracy: equip.ranged_accuracy || 0,
            store_tp: equip.store_tp || 0,
            double_attack_pct: equip.double_attack_pct || 0,
            skillchain_bonus: equip.skillchain_bonus || 0,
            triple_attack_pct: equip.triple_attack_pct || 0,
            regen: equip.regen || 0,
            refresh: equip.refresh || 0,
            main_weapon_skill_id: mainWeaponSkillId,
            sub_weapon_skill_id: subWeaponSkillId,
            ranged_weapon_skill_id: rangedWeaponSkillId,
            skill_bonus_main: equip.skill_bonus_main || {},
            skill_bonus_sub: equip.skill_bonus_sub || {},
            skill_bonus_ranged: equip.skill_bonus_ranged || {},
            skill_bonus_global: equip.skill_bonus_global || {},
        };
        const totalStats = calculate_status_from_profile(profile, jobKey, supportJob, bonusStats);

        // === 左パネル: 基本 9 ステ (素 / 装備 / 合計) ===
        document.getElementById('equipBaseHp').textContent = baseStats.hp || 0;
        document.getElementById('equipBaseMp').textContent = baseStats.mp || 0;
        document.getElementById('equipBaseStr').textContent = baseStats.str_ || 0;
        document.getElementById('equipBaseDex').textContent = baseStats.dex || 0;
        document.getElementById('equipBaseVit').textContent = baseStats.vit || 0;
        document.getElementById('equipBaseAgi').textContent = baseStats.agi || 0;
        document.getElementById('equipBaseInt').textContent = baseStats.int || 0;
        document.getElementById('equipBaseMnd').textContent = baseStats.mnd || 0;
        document.getElementById('equipBaseChr').textContent = baseStats.chr || 0;

        document.getElementById('equipEquipHp').textContent = formatStatBonus(equip.hp);
        document.getElementById('equipEquipMp').textContent = formatStatBonus(equip.mp);
        document.getElementById('equipEquipStr').textContent = formatStatBonus(equip.str);
        document.getElementById('equipEquipDex').textContent = formatStatBonus(equip.dex);
        document.getElementById('equipEquipVit').textContent = formatStatBonus(equip.vit);
        document.getElementById('equipEquipAgi').textContent = formatStatBonus(equip.agi);
        document.getElementById('equipEquipInt').textContent = formatStatBonus(equip.int);
        document.getElementById('equipEquipMnd').textContent = formatStatBonus(equip.mnd);
        document.getElementById('equipEquipChr').textContent = formatStatBonus(equip.chr);

        document.getElementById('equipTotalHp').textContent = totalStats.hp || 0;
        document.getElementById('equipTotalMp').textContent = totalStats.mp || 0;
        document.getElementById('equipTotalStr').textContent = totalStats.str_ || 0;
        document.getElementById('equipTotalDex').textContent = totalStats.dex || 0;
        document.getElementById('equipTotalVit').textContent = totalStats.vit || 0;
        document.getElementById('equipTotalAgi').textContent = totalStats.agi || 0;
        document.getElementById('equipTotalInt').textContent = totalStats.int || 0;
        document.getElementById('equipTotalMnd').textContent = totalStats.mnd || 0;
        document.getElementById('equipTotalChr').textContent = totalStats.chr || 0;

        // 防御系ステータス（素 = 合計 - 装備）
        const defEquip = equip.def || 0;
        const defTotal = totalStats.def || 0;
        const evaEquip = equip.evasion || 0;
        const evaTotal = totalStats.evasion || 0;
        const mdefEquip = equip.magic_def_bonus || 0;
        const mdefTotal = totalStats.mdef || 0;
        const mevaEquip = equip.magic_evasion || 0;
        const mevaBonus = totalStats.magic_evasion_bonus || 0;
        const mevaTotal = mevaEquip + mevaBonus;
        document.getElementById('equipBaseDef').textContent = defTotal - defEquip;
        document.getElementById('equipEquipDef').textContent = defEquip;
        document.getElementById('equipTotalDef').textContent = defTotal;
        document.getElementById('equipBaseEva').textContent = evaTotal - evaEquip;
        document.getElementById('equipEquipEva').textContent = evaEquip;
        document.getElementById('equipTotalEva').textContent = evaTotal;
        document.getElementById('equipBaseMdef').textContent = mdefTotal - mdefEquip;
        document.getElementById('equipEquipMdef').textContent = mdefEquip;
        document.getElementById('equipTotalMdef').textContent = mdefTotal;
        document.getElementById('equipBaseMeva').textContent = mevaBonus;
        document.getElementById('equipEquipMeva').textContent = mevaEquip;
        document.getElementById('equipTotalMeva').textContent = mevaTotal;
        document.getElementById('equipEquipHaste').textContent = fmtPct(equip.haste_pct);
        document.getElementById('equipTotalHaste').textContent = fmtPct(equip.haste_pct);
        document.getElementById('equipEquipDt').textContent = fmtPct(equip.damage_taken_pct);
        document.getElementById('equipTotalDt').textContent = fmtPct(equip.damage_taken_pct);
        document.getElementById('equipEquipPdt').textContent = fmtPct(equip.physical_damage_taken_pct);
        document.getElementById('equipTotalPdt').textContent = fmtPct(equip.physical_damage_taken_pct);
        document.getElementById('equipEquipMdt').textContent = fmtPct(equip.magic_damage_taken_pct);
        document.getElementById('equipTotalMdt').textContent = fmtPct(equip.magic_damage_taken_pct);

        const evasionTotal = totalStats.evasion != null ? totalStats.evasion : 0;
        const magicAttackTotal = totalStats.magic_attack != null ? totalStats.magic_attack : 0;
        const magicAccuracyTotal = (equip.magic_accuracy || 0) + (totalStats.magic_accuracy_bonus || 0);
        const magicEvasionTotal = (equip.magic_evasion || 0) + (totalStats.magic_evasion_bonus || 0);
        const magicDamageTotal = equip.magic_damage || 0;
        const wsDamagePct = equip.weapon_skill_damage_pct || 0;
        // 連携ボーナス総合 (装備 + ジョブ特性 + ギフト) は WASM 側で算出済み。
        const skillchainBonusTotal = totalStats.skillchain_bonus || 0;

        // ----- Tab 1: 待機/回避/防御 -----
        // オートリジェネ/オートリフレシュは装備 + ジョブ特性の合計を表示
        setText('statDefRegen', numOrDash(totalStats.regen));
        setText('statDefRefresh', numOrDash(totalStats.refresh));
        setText('statDefRegain', numOrDash(equip.regain));
        setText('statDefFastCast', pctOrDash(equip.fast_cast_pct));
        setText('statDefQuickMagic', pctOrDash(equip.quick_magic_pct));
        // Snapshot/Rapid Shot は装備テキストでも単位無し表記が標準
        setText('statDefSnapshot', numOrDash(equip.snapshot_pct));
        setText('statDefRapidShot', numOrDash(equip.rapid_shot_pct));
        // 属性レジスト
        for (const elem of ['fire', 'ice', 'wind', 'earth', 'lightning', 'water', 'light', 'dark']) {
            const id = 'statDefRes' + elem.charAt(0).toUpperCase() + elem.slice(1);
            setText(id, numOrDash(equip['resist_' + elem]));
        }
        // 状態異常レジスト
        for (const st of ['sleep', 'paralysis', 'bind', 'silence', 'gravity', 'slow',
                          'petrification', 'stun', 'poison', 'charm', 'blind', 'curse',
                          'virus', 'amnesia', 'terror', 'death']) {
            const id = 'statDefRes' + st.charAt(0).toUpperCase() + st.slice(1);
            setText(id, numOrDash(equip['resist_' + st]));
        }

        // ----- Tab 2: オートアタック (近接) -----
        setText('statAaMainSkill', formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value));
        setText('statAaMainAtk', numOrDash(totalStats.main_attack));
        setText('statAaMainAcc', numOrDash(totalStats.main_accuracy));
        setText('statAaSubSkill', formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value));
        setText('statAaSubAtk', totalStats.sub_attack != null ? totalStats.sub_attack : '-');
        setText('statAaSubAcc', totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-');
        setText('statAaStp', numOrDash(totalStats.store_tp));
        setText('statAaDa', pctOrDash(totalStats.double_attack_pct));
        setText('statAaTa', pctOrDash(totalStats.triple_attack_pct));
        setText('statAaQa', pctOrDash(equip.quad_attack_pct));
        setText('statAaCrit', pctOrDash(equip.critical_hit_rate_pct));
        setText('statAaDaDmg', pctOrDash(equip.double_attack_damage_pct));
        setText('statAaTaDmg', pctOrDash(equip.triple_attack_damage_pct));
        setText('statAaCritDmg', pctOrDash(equip.critical_hit_damage_pct));
        setText('statAaSb', numOrDash(equip.subtle_blow));
        setText('statAaSb2', numOrDash(equip.subtle_blow_2));

        // ----- Tab 3: 遠隔攻撃 -----
        setText('statRaSkill', formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value));
        setText('statRaAtk', totalStats.ranged_attack != null ? totalStats.ranged_attack : '-');
        setText('statRaAcc', totalStats.ranged_accuracy != null ? totalStats.ranged_accuracy : '-');
        setText('statRaStr', totalStats.str_ || '-');
        setText('statRaAgi', totalStats.agi || '-');
        setText('statRaStp', numOrDash(totalStats.store_tp));

        // ----- Tab 4: 近接物理 WS -----
        setText('statMwsSkill', formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value));
        setText('statMwsAtk', numOrDash(totalStats.main_attack));
        setText('statMwsAcc', numOrDash(totalStats.main_accuracy));
        setText('statMwsSubSkill', formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value));
        setText('statMwsSubAtk', totalStats.sub_attack != null ? totalStats.sub_attack : '-');
        setText('statMwsSubAcc', totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-');
        setText('statMwsStp', numOrDash(totalStats.store_tp));
        setText('statMwsSb', numOrDash(equip.subtle_blow));
        setText('statMwsSb2', numOrDash(equip.subtle_blow_2));
        setText('statMwsDa', pctOrDash(totalStats.double_attack_pct));
        setText('statMwsTa', pctOrDash(totalStats.triple_attack_pct));
        setText('statMwsQa', pctOrDash(equip.quad_attack_pct));
        setText('statMwsCrit', pctOrDash(equip.critical_hit_rate_pct));
        setText('statMwsCritDmg', pctOrDash(equip.critical_hit_damage_pct));
        setText('statMwsWsdmg', pctOrDash(wsDamagePct));
        setText('statMwsTpb', numOrDash(equip.tp_bonus));
        setText('statMwsScb', numOrDash(skillchainBonusTotal));
        setText('statMwsPdl', pctOrDash(equip.physical_damage_limit_pct));

        // ----- Tab 5: 遠隔物理 WS -----
        setText('statRwsSkill', formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value));
        setText('statRwsAtk', totalStats.ranged_attack != null ? totalStats.ranged_attack : '-');
        setText('statRwsAcc', totalStats.ranged_accuracy != null ? totalStats.ranged_accuracy : '-');
        setText('statRwsStp', numOrDash(totalStats.store_tp));
        setText('statRwsSb', numOrDash(equip.subtle_blow));
        setText('statRwsSb2', numOrDash(equip.subtle_blow_2));
        setText('statRwsCrit', pctOrDash(equip.critical_hit_rate_pct));
        setText('statRwsCritDmg', pctOrDash(equip.critical_hit_damage_pct));
        setText('statRwsWsdmg', pctOrDash(wsDamagePct));
        setText('statRwsTpb', numOrDash(equip.tp_bonus));
        setText('statRwsScb', numOrDash(skillchainBonusTotal));
        setText('statRwsPdl', pctOrDash(equip.physical_damage_limit_pct));
        setText('statRwsTs', numOrDash(equip.true_shot));

        // ----- Tab 6: 属性 WS -----
        // 魔命スキルはメイン武器スロットの装備分のみ表示 (サブ/レンジは UI 上ダッシュ固定)。
        // 魔命合計 = 各行の武器スキル値 + メインの魔命スキル + 魔命総合 (装備 + ボーナス)。
        const mainMaccSkill = equip.slot_stats?.main?.magic_accuracy_skill || 0;
        const mainSkillValue = totalStats.main_weapon_skill_value || 0;
        const rangedSkillValue = totalStats.ranged_weapon_skill_value || 0;
        setText('statEwsMainSkill', formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value));
        setText('statEwsMainMaccSkill', numOrDash(mainMaccSkill));
        setText('statEwsMainMaccTotal', numOrDash(mainSkillValue + mainMaccSkill + magicAccuracyTotal));
        setText('statEwsRangedSkill', formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value));
        setText('statEwsRangedMaccTotal', numOrDash(rangedSkillValue + mainMaccSkill + magicAccuracyTotal));
        setText('statEwsMatk', numOrDash(magicAttackTotal));
        setText('statEwsMacc', numOrDash(magicAccuracyTotal));
        setText('statEwsStp', numOrDash(totalStats.store_tp));
        setText('statEwsSb', numOrDash(equip.subtle_blow));
        setText('statEwsSb2', numOrDash(equip.subtle_blow_2));
        setText('statEwsMdmg', numOrDash(magicDamageTotal));
        setText('statEwsAff', numOrDash(equip.magic_affinity));
        setText('statEwsMcrit2', pctOrDash(equip.magic_critical_hit_2_pct));
        setText('statEwsWsdmg', pctOrDash(wsDamagePct));
        setText('statEwsTpb', numOrDash(equip.tp_bonus));
        setText('statEwsScb', numOrDash(skillchainBonusTotal));

        // ----- Tab 7: 近接属性物理 WS -----
        // 物理 WS 系の攻撃/命中/DA 等と、属性 WS の魔攻/魔命/魔法ダメ等を併記。
        // 魔命合計は各行の武器スキル値 + その枠の魔命スキル + 魔命総合の合算。
        const subMaccSkill = equip.slot_stats?.sub?.magic_accuracy_skill || 0;
        const subSkillValue = totalStats.sub_weapon_skill_value || 0;
        setText('statMewsSkill', formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value));
        setText('statMewsAtk', numOrDash(totalStats.main_attack));
        setText('statMewsAcc', numOrDash(totalStats.main_accuracy));
        setText('statMewsMaccSkill', numOrDash(mainMaccSkill));
        setText('statMewsMaccTotal', numOrDash(mainSkillValue + mainMaccSkill + magicAccuracyTotal));
        setText('statMewsSubSkill', formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value));
        setText('statMewsSubAtk', totalStats.sub_attack != null ? totalStats.sub_attack : '-');
        setText('statMewsSubAcc', totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-');
        setText('statMewsSubMaccSkill', numOrDash(subMaccSkill));
        setText('statMewsSubMaccTotal', numOrDash(subSkillValue + subMaccSkill + magicAccuracyTotal));
        setText('statMewsMatk', numOrDash(magicAttackTotal));
        setText('statMewsMacc', numOrDash(magicAccuracyTotal));
        setText('statMewsMdmg', numOrDash(magicDamageTotal));
        setText('statMewsAff', numOrDash(equip.magic_affinity));
        setText('statMewsMcrit2', pctOrDash(equip.magic_critical_hit_2_pct));
        setText('statMewsStp', numOrDash(totalStats.store_tp));
        setText('statMewsSb', numOrDash(equip.subtle_blow));
        setText('statMewsSb2', numOrDash(equip.subtle_blow_2));
        setText('statMewsDa', pctOrDash(totalStats.double_attack_pct));
        setText('statMewsTa', pctOrDash(totalStats.triple_attack_pct));
        setText('statMewsQa', pctOrDash(equip.quad_attack_pct));
        setText('statMewsCrit', pctOrDash(equip.critical_hit_rate_pct));
        setText('statMewsCritDmg', pctOrDash(equip.critical_hit_damage_pct));
        setText('statMewsWsdmg', pctOrDash(wsDamagePct));
        setText('statMewsTpb', numOrDash(equip.tp_bonus));
        setText('statMewsScb', numOrDash(skillchainBonusTotal));
        setText('statMewsPdl', pctOrDash(equip.physical_damage_limit_pct));

        // ----- Tab 8: 遠隔属性物理 WS -----
        // レンジ枠基準: 武器スキルはレンジ、攻撃/命中は飛攻/飛命。
        // 魔命合計は レンジ武器スキル値 + メイン魔命スキル + 魔命総合 の合算。
        setText('statRewsSkill', formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value));
        setText('statRewsAtk', numOrDash(totalStats.ranged_attack));
        setText('statRewsAcc', numOrDash(totalStats.ranged_accuracy));
        setText('statRewsMaccSkill', numOrDash(mainMaccSkill));
        setText('statRewsMaccTotal', numOrDash(rangedSkillValue + mainMaccSkill + magicAccuracyTotal));
        setText('statRewsMatk', numOrDash(magicAttackTotal));
        setText('statRewsMacc', numOrDash(magicAccuracyTotal));
        setText('statRewsMdmg', numOrDash(magicDamageTotal));
        setText('statRewsAff', numOrDash(equip.magic_affinity));
        setText('statRewsMcrit2', pctOrDash(equip.magic_critical_hit_2_pct));
        setText('statRewsStp', numOrDash(totalStats.store_tp));
        setText('statRewsSb', numOrDash(equip.subtle_blow));
        setText('statRewsSb2', numOrDash(equip.subtle_blow_2));
        setText('statRewsCrit', pctOrDash(equip.critical_hit_rate_pct));
        setText('statRewsCritDmg', pctOrDash(equip.critical_hit_damage_pct));
        setText('statRewsWsdmg', pctOrDash(wsDamagePct));
        setText('statRewsTpb', numOrDash(equip.tp_bonus));
        setText('statRewsScb', numOrDash(skillchainBonusTotal));
        setText('statRewsPdl', pctOrDash(equip.physical_damage_limit_pct));
        setText('statRewsTs', numOrDash(equip.true_shot));

        // ----- Tab 9: 魔法 -----
        setText('statMgMatk', numOrDash(magicAttackTotal));
        setText('statMgMacc', numOrDash(magicAccuracyTotal));
        setText('statMgMeva', numOrDash(magicEvasionTotal));
        setText('statMgMdmg', numOrDash(magicDamageTotal));
        setText('statMgInt', totalStats.int || '-');
        setText('statMgMnd', totalStats.mnd || '-');
        setText('statMgChr', totalStats.chr || '-');
        setText('statMgMp', totalStats.mp || '-');

        // 有効スキル値の表示（値が 0 のスキルは非表示）
        const skillsContainer = document.getElementById('equipEffectiveSkills');
        skillsContainer.innerHTML = '';
        const effSkills = totalStats.effective_skills || {};
        ALL_SKILL_KEYS.forEach(([k, ja]) => {
            const v = effSkills[k] || 0;
            if (v <= 0) return;
            const badge = (k === totalStats.main_weapon_skill) ? ' <span style="color:#8ab4f8;">(主武器)</span>' : '';
            const div = document.createElement('div');
            div.innerHTML = `<span style="color:#888;">${ja}:</span> <strong>${v}</strong>${badge}`;
            skillsContainer.appendChild(div);
        });
        if (skillsContainer.innerHTML === '') {
            skillsContainer.innerHTML = '<div style="color:#666;">(表示できるスキルなし)</div>';
        }
    } catch (e) {
        console.error('Error calculating equipment edit status:', e);
        clearAllEquipStats();
    }
}

export function clearAllEquipStats() {
    ['Hp', 'Mp', 'Str', 'Dex', 'Vit', 'Agi', 'Int', 'Mnd', 'Chr'].forEach(stat => {
        document.getElementById(`equipBase${stat}`).textContent = '-';
        document.getElementById(`equipEquip${stat}`).textContent = '-';
        document.getElementById(`equipTotal${stat}`).textContent = '-';
    });
    ['Def', 'Eva', 'Mdef', 'Meva'].forEach(stat => {
        document.getElementById(`equipBase${stat}`).textContent = '-';
        document.getElementById(`equipEquip${stat}`).textContent = '-';
        document.getElementById(`equipTotal${stat}`).textContent = '-';
    });
    ['Haste', 'Dt', 'Pdt', 'Mdt'].forEach(stat => {
        document.getElementById(`equipEquip${stat}`).textContent = '-';
        document.getElementById(`equipTotal${stat}`).textContent = '-';
    });
    // 9 サブタブ内の全表示要素をクリア
    const tabIds = [
        // Tab 1: 防御
        'statDefHp', 'statDefMp', 'statDefDef', 'statDefMdef', 'statDefEva',
        'statDefMeva', 'statDefDt', 'statDefPdt', 'statDefMdt',
        // Tab 2: オートアタック
        'statAaMainSkill', 'statAaMainAtk', 'statAaMainAcc',
        'statAaSubSkill', 'statAaSubAtk', 'statAaSubAcc',
        'statAaStp', 'statAaDa', 'statAaTa', 'statAaQa', 'statAaCrit',
        'statAaDaDmg', 'statAaTaDmg', 'statAaCritDmg', 'statAaSb', 'statAaSb2',
        // Tab 3: 遠隔
        'statRaSkill', 'statRaAtk', 'statRaAcc', 'statRaStr', 'statRaAgi', 'statRaStp',
        // Tab 4: 近接 WS
        'statMwsSkill', 'statMwsAtk', 'statMwsAcc',
        'statMwsSubSkill', 'statMwsSubAtk', 'statMwsSubAcc',
        'statMwsStp', 'statMwsSb', 'statMwsSb2', 'statMwsDa', 'statMwsTa', 'statMwsQa',
        'statMwsCrit', 'statMwsCritDmg', 'statMwsWsdmg', 'statMwsTpb', 'statMwsScb', 'statMwsPdl',
        // Tab 5: 遠隔 WS
        'statRwsSkill', 'statRwsAtk', 'statRwsAcc',
        'statRwsStp', 'statRwsSb', 'statRwsSb2', 'statRwsCrit', 'statRwsCritDmg',
        'statRwsWsdmg', 'statRwsTpb', 'statRwsScb', 'statRwsPdl', 'statRwsTs',
        // Tab 6: 属性 WS
        'statEwsMainSkill', 'statEwsMainMaccSkill', 'statEwsMainMaccTotal',
        'statEwsRangedSkill', 'statEwsRangedMaccTotal',
        'statEwsMatk', 'statEwsMacc', 'statEwsStp', 'statEwsSb', 'statEwsSb2', 'statEwsMdmg',
        'statEwsAff', 'statEwsMcrit2', 'statEwsWsdmg', 'statEwsTpb', 'statEwsScb',
        // Tab 7: 近接属性物理 WS
        'statMewsSkill', 'statMewsAtk', 'statMewsAcc', 'statMewsMaccSkill', 'statMewsMaccTotal',
        'statMewsSubSkill', 'statMewsSubAtk', 'statMewsSubAcc', 'statMewsSubMaccSkill', 'statMewsSubMaccTotal',
        'statMewsMatk', 'statMewsMacc', 'statMewsMdmg', 'statMewsAff', 'statMewsMcrit2',
        'statMewsStp', 'statMewsSb', 'statMewsSb2', 'statMewsDa', 'statMewsTa', 'statMewsQa',
        'statMewsCrit', 'statMewsCritDmg', 'statMewsWsdmg', 'statMewsTpb', 'statMewsScb', 'statMewsPdl',
        // Tab 8: 遠隔属性物理 WS
        'statRewsSkill', 'statRewsAtk', 'statRewsAcc', 'statRewsMaccSkill', 'statRewsMaccTotal',
        'statRewsMatk', 'statRewsMacc', 'statRewsMdmg', 'statRewsAff', 'statRewsMcrit2',
        'statRewsStp', 'statRewsSb', 'statRewsSb2', 'statRewsCrit', 'statRewsCritDmg',
        'statRewsWsdmg', 'statRewsTpb', 'statRewsScb', 'statRewsPdl', 'statRewsTs',
        // Tab 9: 魔法
        'statMgMatk', 'statMgMacc', 'statMgMeva', 'statMgMdmg',
        'statMgInt', 'statMgMnd', 'statMgChr', 'statMgMp',
    ];
    tabIds.forEach(id => {
        const elem = document.getElementById(id);
        if (elem) elem.textContent = '-';
    });
    const skillsContainer = document.getElementById('equipEffectiveSkills');
    if (skillsContainer) skillsContainer.innerHTML = '';
}
