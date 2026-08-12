// キャラクター + 装備セットの合計ステータス計算 (旧 web/js/status-display.js の移植)。
// 旧実装は計算結果を setText(id, v) で DOM へ直接書いていたが、ここでは
// 「id → 表示値」のレコード (StatusView.values) を組み立てて返し、描画は
// StatusPanel (React) が行う。id は旧実装の DOM id と同一に保っている。
import { JOBS, JP_CATEGORY_COUNT, ALL_SKILL_KEYS } from '../constants';
import { jpDefaultRanks } from '../utils';
import { loadCharacters } from '../storage';
import {
    calculate_status_from_profile,
    calculate_default_skills,
    isWasmReady,
    isItemsLoaded,
} from '../wasm';
import { equipState } from '../equip/equip-store';
import { calculateEquipSetBonuses } from '../equip/equip-bonuses';

import { numOrDash, pctOrDash, formatStatBonus, fmtPct, formatWeaponSkill } from './format';

// 状態異常レジストのキー (表示 id `statDefRes*` の小文字 suffix と対応)
export const STATUS_RESIST_KEYS = [
    'sleep', 'paralysis', 'bind', 'silence', 'gravity', 'slow',
    'petrification', 'stun', 'poison', 'charm', 'blind', 'curse',
    'virus', 'amnesia', 'terror', 'death',
];

// 装備抽出値 + テナシティ (デス以外) の合算。
// テナシティ = RUN ジョブ特性。wiki 795.html により「全状態異常のレジスト効果アップ」と同等で
// デス耐性を除く 15 種に作用。tenacity は 0 を渡せば装備値のみ返る。
export function combineStatusResist(
    equipResists: Record<string, number>,
    tenacity: number
): Record<string, number> {
    const out: Record<string, number> = {};
    for (const st of STATUS_RESIST_KEYS) {
        const equipVal = equipResists['resist_' + st] || 0;
        out[st] = st === 'death' ? equipVal : equipVal + (tenacity || 0);
    }
    return out;
}

export interface EffectiveSkillEntry {
    key: string;
    ja: string;
    value: number;
    /** 主武器スキルなら true (バッジ表示) */
    isMain: boolean;
}

export interface StatusView {
    /** 旧 DOM id → 表示値 */
    values: Record<string, string | number>;
    /** 魔法サブタブ id → 表示可否 (メイン/サポートジョブが該当スキルを持つか) */
    magicTabVisible: Record<string, boolean>;
    effectiveSkills: EffectiveSkillEntry[];
}

/**
 * 現在の装備編集状態 (equipState) からステータス表示ビューを計算する。
 * 表示クリア (旧 clearAllEquipStats 相当) の場合は null を返す。
 */
export async function computeStatusView(): Promise<StatusView | null> {
    if (!isWasmReady() || !isItemsLoaded()) return null;

    const charName = equipState.currentEquipChar;
    const jobKey = equipState.currentEquipJob;
    const supportJob = equipState.currentEquipSupportJob || null;
    const currentEquipSlots = equipState.currentEquipSlots;
    // 共有閲覧モードでは元キャラの snapshot を使う (閲覧者は所有していない)
    const characterOverride = equipState.sharedCharacterOverride;

    if (!charName || !jobKey) return null;

    let ch;
    if (characterOverride) {
        ch = characterOverride;
    } else {
        const characters = await loadCharacters();
        ch = characters.find((c: { name: string }) => c.name === charName);
    }
    if (!ch) return null;

    const jobLevel = ch.job_levels[jobKey];
    if (!jobLevel || jobLevel.level === 0) return null;

    try {
        // JP データが未定義の場合は全振り（全 20）をデフォルトとして補完
        const jpCategories: Record<string, { ranks: number[] }> = {};
        const storedJp: Record<string, { ranks?: unknown } | undefined> =
            (ch.job_points && ch.job_points.categories) || {};
        JOBS.forEach((job: { key: string }) => {
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

        const getSlotSkillId = (slotKey: string) => {
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
            subtle_blow: equip.subtle_blow || 0,
            rapid_shot_pct: equip.rapid_shot_pct || 0,
            fast_cast_pct: equip.fast_cast_pct || 0,
            main_weapon_skill_id: mainWeaponSkillId,
            sub_weapon_skill_id: subWeaponSkillId,
            ranged_weapon_skill_id: rangedWeaponSkillId,
            skill_bonus_main: equip.skill_bonus_main || {},
            skill_bonus_sub: equip.skill_bonus_sub || {},
            skill_bonus_ranged: equip.skill_bonus_ranged || {},
            skill_bonus_global: equip.skill_bonus_global || {},
        };
        const totalStats = calculate_status_from_profile(profile, jobKey, supportJob, bonusStats);

        const V: Record<string, string | number> = {};

        // === 左パネル: 基本 9 ステ (素 / 装備 / 合計) ===
        V.equipBaseHp = baseStats.hp || 0;
        V.equipBaseMp = baseStats.mp || 0;
        V.equipBaseStr = baseStats.str_ || 0;
        V.equipBaseDex = baseStats.dex || 0;
        V.equipBaseVit = baseStats.vit || 0;
        V.equipBaseAgi = baseStats.agi || 0;
        V.equipBaseInt = baseStats.int || 0;
        V.equipBaseMnd = baseStats.mnd || 0;
        V.equipBaseChr = baseStats.chr || 0;

        V.equipEquipHp = formatStatBonus(equip.hp);
        V.equipEquipMp = formatStatBonus(equip.mp);
        V.equipEquipStr = formatStatBonus(equip.str);
        V.equipEquipDex = formatStatBonus(equip.dex);
        V.equipEquipVit = formatStatBonus(equip.vit);
        V.equipEquipAgi = formatStatBonus(equip.agi);
        V.equipEquipInt = formatStatBonus(equip.int);
        V.equipEquipMnd = formatStatBonus(equip.mnd);
        V.equipEquipChr = formatStatBonus(equip.chr);

        V.equipTotalHp = totalStats.hp || 0;
        V.equipTotalMp = totalStats.mp || 0;
        V.equipTotalStr = totalStats.str_ || 0;
        V.equipTotalDex = totalStats.dex || 0;
        V.equipTotalVit = totalStats.vit || 0;
        V.equipTotalAgi = totalStats.agi || 0;
        V.equipTotalInt = totalStats.int || 0;
        V.equipTotalMnd = totalStats.mnd || 0;
        V.equipTotalChr = totalStats.chr || 0;

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
        V.equipBaseDef = defTotal - defEquip;
        V.equipEquipDef = defEquip;
        V.equipTotalDef = defTotal;
        V.equipBaseEva = evaTotal - evaEquip;
        V.equipEquipEva = evaEquip;
        V.equipTotalEva = evaTotal;
        V.equipBaseMdef = mdefTotal - mdefEquip;
        V.equipEquipMdef = mdefEquip;
        V.equipTotalMdef = mdefTotal;
        V.equipBaseMeva = mevaBonus;
        V.equipEquipMeva = mevaEquip;
        V.equipTotalMeva = mevaTotal;
        V.equipEquipHaste = fmtPct(equip.haste_pct);
        V.equipTotalHaste = fmtPct(equip.haste_pct);
        V.equipEquipDt = fmtPct(equip.damage_taken_pct);
        V.equipTotalDt = fmtPct(equip.damage_taken_pct);
        V.equipEquipPdt = fmtPct(equip.physical_damage_taken_pct);
        V.equipTotalPdt = fmtPct(equip.physical_damage_taken_pct);
        V.equipEquipMdt = fmtPct(equip.magic_damage_taken_pct);
        V.equipTotalMdt = fmtPct(equip.magic_damage_taken_pct);

        const magicAttackTotal = totalStats.magic_attack != null ? totalStats.magic_attack : 0;
        const magicAccuracyTotal = (equip.magic_accuracy || 0) + (totalStats.magic_accuracy_bonus || 0);
        const magicEvasionTotal = (equip.magic_evasion || 0) + (totalStats.magic_evasion_bonus || 0);
        const magicDamageTotal = equip.magic_damage || 0;
        const wsDamagePct = equip.weapon_skill_damage_pct || 0;
        // 連携ボーナス総合 (装備 + ジョブ特性 + ギフト) は WASM 側で算出済み。
        const skillchainBonusTotal = totalStats.skillchain_bonus || 0;

        // ----- Tab 1: 待機/回避/防御 -----
        // オートリジェネ/オートリフレシュは装備 + ジョブ特性の合計を表示
        V.statDefRegen = numOrDash(totalStats.regen);
        V.statDefRefresh = numOrDash(totalStats.refresh);
        V.statDefRegain = numOrDash(equip.regain);
        V.statDefFastCast = pctOrDash(totalStats.fast_cast_pct);
        V.statDefQuickMagic = pctOrDash(equip.quick_magic_pct);
        // Snapshot/Rapid Shot は装備テキストでも単位無し表記が標準
        V.statDefSnapshot = numOrDash(equip.snapshot_pct);
        V.statDefRapidShot = numOrDash(totalStats.rapid_shot_pct);
        // 属性レジスト
        for (const elem of ['fire', 'ice', 'wind', 'earth', 'lightning', 'water', 'light', 'dark']) {
            const id = 'statDefRes' + elem.charAt(0).toUpperCase() + elem.slice(1);
            V[id] = numOrDash(equip['resist_' + elem]);
        }
        // 状態異常レジスト = 装備抽出 + テナシティ (デス以外 15 種)
        const statusResistTotals = combineStatusResist(equip, totalStats.tenacity || 0);
        for (const [st, total] of Object.entries(statusResistTotals)) {
            const id = 'statDefRes' + st.charAt(0).toUpperCase() + st.slice(1);
            V[id] = numOrDash(total);
        }

        // ----- Tab 2: オートアタック (近接) -----
        V.statAaMainSkill = formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value);
        V.statAaMainAtk = numOrDash(totalStats.main_attack);
        V.statAaMainAcc = numOrDash(totalStats.main_accuracy);
        V.statAaSubSkill = formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value);
        V.statAaSubAtk = totalStats.sub_attack != null ? totalStats.sub_attack : '-';
        V.statAaSubAcc = totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-';
        V.statAaStp = numOrDash(totalStats.store_tp);
        V.statAaDa = pctOrDash(totalStats.double_attack_pct);
        V.statAaTa = pctOrDash(totalStats.triple_attack_pct);
        V.statAaQa = pctOrDash(equip.quad_attack_pct);
        V.statAaCrit = pctOrDash(equip.critical_hit_rate_pct);
        V.statAaDaDmg = pctOrDash(equip.double_attack_damage_pct);
        V.statAaTaDmg = pctOrDash(equip.triple_attack_damage_pct);
        V.statAaCritDmg = pctOrDash(equip.critical_hit_damage_pct);
        V.statAaSb = numOrDash(totalStats.subtle_blow);
        V.statAaSb2 = numOrDash(equip.subtle_blow_2);

        // ----- Tab 3: 遠隔攻撃 -----
        V.statRaSkill = formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value);
        V.statRaAtk = totalStats.ranged_attack != null ? totalStats.ranged_attack : '-';
        V.statRaAcc = totalStats.ranged_accuracy != null ? totalStats.ranged_accuracy : '-';
        V.statRaStr = totalStats.str_ || '-';
        V.statRaAgi = totalStats.agi || '-';
        V.statRaStp = numOrDash(totalStats.store_tp);

        // ----- Tab 4: 近接物理 WS -----
        V.statMwsSkill = formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value);
        V.statMwsAtk = numOrDash(totalStats.main_attack);
        V.statMwsAcc = numOrDash(totalStats.main_accuracy);
        V.statMwsSubSkill = formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value);
        V.statMwsSubAtk = totalStats.sub_attack != null ? totalStats.sub_attack : '-';
        V.statMwsSubAcc = totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-';
        V.statMwsStp = numOrDash(totalStats.store_tp);
        V.statMwsSb = numOrDash(totalStats.subtle_blow);
        V.statMwsSb2 = numOrDash(equip.subtle_blow_2);
        V.statMwsDa = pctOrDash(totalStats.double_attack_pct);
        V.statMwsTa = pctOrDash(totalStats.triple_attack_pct);
        V.statMwsQa = pctOrDash(equip.quad_attack_pct);
        V.statMwsCrit = pctOrDash(equip.critical_hit_rate_pct);
        V.statMwsCritDmg = pctOrDash(equip.critical_hit_damage_pct);
        V.statMwsWsdmg = pctOrDash(wsDamagePct);
        V.statMwsTpb = numOrDash(equip.tp_bonus);
        V.statMwsScb = numOrDash(skillchainBonusTotal);
        V.statMwsPdl = pctOrDash(equip.physical_damage_limit_pct);

        // ----- Tab 5: 遠隔物理 WS -----
        V.statRwsSkill = formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value);
        V.statRwsAtk = totalStats.ranged_attack != null ? totalStats.ranged_attack : '-';
        V.statRwsAcc = totalStats.ranged_accuracy != null ? totalStats.ranged_accuracy : '-';
        V.statRwsStp = numOrDash(totalStats.store_tp);
        V.statRwsSb = numOrDash(totalStats.subtle_blow);
        V.statRwsSb2 = numOrDash(equip.subtle_blow_2);
        V.statRwsCrit = pctOrDash(equip.critical_hit_rate_pct);
        V.statRwsCritDmg = pctOrDash(equip.critical_hit_damage_pct);
        V.statRwsWsdmg = pctOrDash(wsDamagePct);
        V.statRwsTpb = numOrDash(equip.tp_bonus);
        V.statRwsScb = numOrDash(skillchainBonusTotal);
        V.statRwsPdl = pctOrDash(equip.physical_damage_limit_pct);
        V.statRwsTs = numOrDash(equip.true_shot);

        // ----- Tab 6: 属性 WS -----
        // 魔命スキルはメイン武器スロットの装備分のみ表示 (サブ/レンジは UI 上ダッシュ固定)。
        // 魔命合計 = 各行の武器スキル値 + メインの魔命スキル + 魔命総合 (装備 + ボーナス)。
        const mainMaccSkill = equip.slot_stats?.main?.magic_accuracy_skill || 0;
        const mainSkillValue = totalStats.main_weapon_skill_value || 0;
        const rangedSkillValue = totalStats.ranged_weapon_skill_value || 0;
        V.statEwsMainSkill = formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value);
        V.statEwsMainMaccSkill = numOrDash(mainMaccSkill);
        V.statEwsMainMaccTotal = numOrDash(mainSkillValue + mainMaccSkill + magicAccuracyTotal);
        V.statEwsRangedSkill = formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value);
        V.statEwsRangedMaccTotal = numOrDash(rangedSkillValue + mainMaccSkill + magicAccuracyTotal);
        V.statEwsMatk = numOrDash(magicAttackTotal);
        V.statEwsMacc = numOrDash(magicAccuracyTotal);
        V.statEwsStp = numOrDash(totalStats.store_tp);
        V.statEwsSb = numOrDash(totalStats.subtle_blow);
        V.statEwsSb2 = numOrDash(equip.subtle_blow_2);
        V.statEwsMdmg = numOrDash(magicDamageTotal);
        V.statEwsAff = numOrDash(equip.magic_affinity);
        V.statEwsMcrit2 = pctOrDash(equip.magic_critical_hit_2_pct);
        V.statEwsWsdmg = pctOrDash(wsDamagePct);
        V.statEwsTpb = numOrDash(equip.tp_bonus);
        V.statEwsScb = numOrDash(skillchainBonusTotal);

        // ----- Tab 7: 近接属性物理 WS -----
        const subMaccSkill = equip.slot_stats?.sub?.magic_accuracy_skill || 0;
        const subSkillValue = totalStats.sub_weapon_skill_value || 0;
        V.statMewsSkill = formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value);
        V.statMewsAtk = numOrDash(totalStats.main_attack);
        V.statMewsAcc = numOrDash(totalStats.main_accuracy);
        V.statMewsMaccSkill = numOrDash(mainMaccSkill);
        V.statMewsMaccTotal = numOrDash(mainSkillValue + mainMaccSkill + magicAccuracyTotal);
        V.statMewsSubSkill = formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value);
        V.statMewsSubAtk = totalStats.sub_attack != null ? totalStats.sub_attack : '-';
        V.statMewsSubAcc = totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-';
        V.statMewsSubMaccSkill = numOrDash(subMaccSkill);
        V.statMewsSubMaccTotal = numOrDash(subSkillValue + subMaccSkill + magicAccuracyTotal);
        V.statMewsMatk = numOrDash(magicAttackTotal);
        V.statMewsMacc = numOrDash(magicAccuracyTotal);
        V.statMewsMdmg = numOrDash(magicDamageTotal);
        V.statMewsAff = numOrDash(equip.magic_affinity);
        V.statMewsMcrit2 = pctOrDash(equip.magic_critical_hit_2_pct);
        V.statMewsStp = numOrDash(totalStats.store_tp);
        V.statMewsSb = numOrDash(totalStats.subtle_blow);
        V.statMewsSb2 = numOrDash(equip.subtle_blow_2);
        V.statMewsDa = pctOrDash(totalStats.double_attack_pct);
        V.statMewsTa = pctOrDash(totalStats.triple_attack_pct);
        V.statMewsQa = pctOrDash(equip.quad_attack_pct);
        V.statMewsCrit = pctOrDash(equip.critical_hit_rate_pct);
        V.statMewsCritDmg = pctOrDash(equip.critical_hit_damage_pct);
        V.statMewsWsdmg = pctOrDash(wsDamagePct);
        V.statMewsTpb = numOrDash(equip.tp_bonus);
        V.statMewsScb = numOrDash(skillchainBonusTotal);
        V.statMewsPdl = pctOrDash(equip.physical_damage_limit_pct);

        // ----- Tab 8: 遠隔属性物理 WS -----
        // レンジ枠基準: 武器スキルはレンジ、攻撃/命中は飛攻/飛命。
        V.statRewsSkill = formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value);
        V.statRewsAtk = numOrDash(totalStats.ranged_attack);
        V.statRewsAcc = numOrDash(totalStats.ranged_accuracy);
        V.statRewsMaccSkill = numOrDash(mainMaccSkill);
        V.statRewsMaccTotal = numOrDash(rangedSkillValue + mainMaccSkill + magicAccuracyTotal);
        V.statRewsMatk = numOrDash(magicAttackTotal);
        V.statRewsMacc = numOrDash(magicAccuracyTotal);
        V.statRewsMdmg = numOrDash(magicDamageTotal);
        V.statRewsAff = numOrDash(equip.magic_affinity);
        V.statRewsMcrit2 = pctOrDash(equip.magic_critical_hit_2_pct);
        V.statRewsStp = numOrDash(totalStats.store_tp);
        V.statRewsSb = numOrDash(totalStats.subtle_blow);
        V.statRewsSb2 = numOrDash(equip.subtle_blow_2);
        V.statRewsCrit = pctOrDash(equip.critical_hit_rate_pct);
        V.statRewsCritDmg = pctOrDash(equip.critical_hit_damage_pct);
        V.statRewsWsdmg = pctOrDash(wsDamagePct);
        V.statRewsTpb = numOrDash(equip.tp_bonus);
        V.statRewsScb = numOrDash(skillchainBonusTotal);
        V.statRewsPdl = pctOrDash(equip.physical_damage_limit_pct);
        V.statRewsTs = numOrDash(equip.true_shot);

        // ----- Tab 9-19: 魔法 (11 種別) -----
        // 各種別ごとに: 該当スキル + 魔攻/魔命/魔回避/魔法ダメ + INT/MND/CHR/MP
        // 呪歌 (歌唱/弦楽器/管楽器), 風水 (風水/風水鈴) は複数スキルを持つ。
        const effSkillsForMagic = totalStats.effective_skills || {};
        const magicTabs: { prefix: string; skills: (string | [string, string])[] }[] = [
            { prefix: 'Divine',     skills: ['Divine'] },
            { prefix: 'Healing',    skills: ['Healing'] },
            { prefix: 'Enhancing',  skills: ['Enhancing'] },
            { prefix: 'Enfeebling', skills: ['Enfeebling'] },
            { prefix: 'Elemental',  skills: ['Elemental'] },
            { prefix: 'Dark',       skills: ['Dark'] },
            { prefix: 'Summoning',  skills: ['Summoning'] },
            { prefix: 'Ninjutsu',   skills: ['Ninjutsu'] },
            // 呪歌 = 歌唱 + 弦楽器 + 管楽器 (3 スキル別表示)
            { prefix: 'Song',       skills: [
                ['SongSingingSkill', 'Singing'],
                ['SongStringSkill',  'StringInstrument'],
                ['SongWindSkill',    'WindInstrument'],
            ]},
            { prefix: 'Blue',       skills: ['BlueMagic'] },
            // 風水 = 風水 + 風水鈴 (2 スキル別表示)
            { prefix: 'Geomancy',   skills: [
                ['GeomancySkill',         'Geomancy'],
                ['GeomancyHandbellSkill', 'Handbell'],
            ]},
        ];
        // タブの可視性: メイン or サポートジョブが該当スキルを習得 (effective_skill > 0) しているもののみ表示
        const skillKeysOf = (skills: (string | [string, string])[]) =>
            skills.map((s) => (typeof s === 'string' ? s : s[1]));
        const isTabAvailable = (skills: (string | [string, string])[]) =>
            skillKeysOf(skills).some((k) => (effSkillsForMagic[k] || 0) > 0);

        const magicTabVisible: Record<string, boolean> = {};
        magicTabs.forEach(({ prefix, skills }) => {
            // スキル値の表示 (単一スキル → "<prefix>Skill"、複数 → 各 ID 指定)
            if (skills.length === 1 && typeof skills[0] === 'string') {
                V[`statMg${prefix}Skill`] = numOrDash(effSkillsForMagic[skills[0]]);
            } else {
                (skills as [string, string][]).forEach(([idSuffix, key]) => {
                    V[`statMg${prefix}${idSuffix.replace(prefix, '')}`] =
                        numOrDash(effSkillsForMagic[key]);
                });
            }
            V[`statMg${prefix}Matk`] = numOrDash(magicAttackTotal);
            V[`statMg${prefix}Macc`] = numOrDash(magicAccuracyTotal);
            V[`statMg${prefix}Meva`] = numOrDash(magicEvasionTotal);
            V[`statMg${prefix}Mdmg`] = numOrDash(magicDamageTotal);
            V[`statMg${prefix}Int`] = totalStats.int || '-';
            V[`statMg${prefix}Mnd`] = totalStats.mnd || '-';
            V[`statMg${prefix}Chr`] = totalStats.chr || '-';
            V[`statMg${prefix}Mp`] = totalStats.mp || '-';

            magicTabVisible[`subtab-magic-${prefix.toLowerCase()}`] = isTabAvailable(skills);
        });

        // 有効スキル値（値が 0 のスキルは非表示）
        const effSkills = totalStats.effective_skills || {};
        const effectiveSkills: EffectiveSkillEntry[] = [];
        (ALL_SKILL_KEYS as [string, string][]).forEach(([k, ja]) => {
            const v = effSkills[k] || 0;
            if (v <= 0) return;
            effectiveSkills.push({
                key: k,
                ja,
                value: v,
                isMain: k === totalStats.main_weapon_skill,
            });
        });

        return { values: V, magicTabVisible, effectiveSkills };
    } catch (e) {
        console.error('Error calculating equipment edit status:', e);
        return null;
    }
}
