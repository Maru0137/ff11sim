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
import { BUILTIN_PROPERTY_ITEMS, type PropertyValueContext } from '../propsets/catalog';
import { propsetsStore } from '../propsets/propsets-store';
import { calculateUserPropertyValues } from '../propsets/user-item-values';

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
    effectiveSkills: EffectiveSkillEntry[];
    /** プロパティ項目 id (カタログ / 'user:<term>') → 表示値 (docs/adr/0015) */
    propertyValues: Record<string, string | number>;
    /** レンジスロットが WS を撃てる武器 (弓術/射撃) か。WS 系テンプレートのレンジ行表示に使う */
    rangedWsWeapon: boolean;
    /** レンジスロットの楽器種別。呪歌テンプレートの楽器スキル列切替に使う */
    songInstrument: 'wind' | 'string' | null;
    /** レンジスロットに風水鈴を装備しているか。風水テンプレートの風水鈴スキル列表示に使う */
    geoHandbell: boolean;
    /** 武器スロット → 装備中武器のスキルキー。内訳の武器スキル列の装備分解決に使う */
    weaponSkillKinds: { main: string | null; sub: string | null; ranged: string | null };
}

/**
 * キャラレコードから WASM に渡す CharacterProfile を組み立てる。
 * JP 未定義時は全振り (全 20)、skills 未定義時はデフォルト (全ジョブ最大) を補完。
 * 内訳モーダル (calculate_status_breakdown) も同一入力を使うため export する。
 */
export function buildStatusProfile(ch: {
    name: string;
    race: string;
    job_levels: unknown;
    merit_points?: unknown;
    job_points?: { categories?: Record<string, { ranks?: unknown }> };
    skills?: { values?: unknown };
}) {
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

    return {
        name: ch.name,
        race: ch.race,
        job_levels: ch.job_levels,
        merit_points: ch.merit_points,
        job_points: { categories: jpCategories },
        skills: charSkills,
    };
}

/**
 * 装備合算値 (calculateEquipSetBonuses の結果) から WASM に渡す BonusStats を
 * 組み立てる。内訳モーダルも同一入力を使うため export する。
 */
export function buildBonusStats(
    equip: any,
    slots: Record<string, { skill?: number | null } | null | undefined> | undefined
) {
    const getSlotSkillId = (slotKey: string) => {
        const s = slots && slots[slotKey];
        return s && s.skill != null ? s.skill : null;
    };

    return {
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
        main_weapon_skill_id: getSlotSkillId('main'),
        sub_weapon_skill_id: getSlotSkillId('sub'),
        ranged_weapon_skill_id: getSlotSkillId('range'),
        skill_bonus_main: equip.skill_bonus_main || {},
        skill_bonus_sub: equip.skill_bonus_sub || {},
        skill_bonus_ranged: equip.skill_bonus_ranged || {},
        skill_bonus_global: equip.skill_bonus_global || {},
    };
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
        const profile = buildStatusProfile(ch);

        const baseStats = calculate_status_from_profile(profile, jobKey, supportJob, null);

        const tempEquipSet = { name: '_temp', slots: currentEquipSlots };
        const equip = calculateEquipSetBonuses(tempEquipSet);

        const bonusStats = buildBonusStats(equip, currentEquipSlots);
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

        // 防御系ステータス。「素」は WASM が解析計算した値 (def_base 等) を使う。
        // 素 + 装備プロパティ == 総合 の恒等式は Rust 側テストで担保
        const mevaEquip = equip.magic_evasion || 0;
        const mevaBonus = totalStats.magic_evasion_bonus || 0;
        const mevaTotal = mevaEquip + mevaBonus;
        V.equipBaseDef = totalStats.def_base || 0;
        V.equipEquipDef = equip.def || 0;
        V.equipTotalDef = totalStats.def || 0;
        V.equipBaseEva = totalStats.evasion_base || 0;
        V.equipEquipEva = equip.evasion || 0;
        V.equipTotalEva = totalStats.evasion || 0;
        V.equipBaseMdef = totalStats.mdef_base || 0;
        V.equipEquipMdef = equip.magic_def_bonus || 0;
        V.equipTotalMdef = totalStats.mdef || 0;
        V.equipBaseMeva = mevaBonus;
        V.equipEquipMeva = mevaEquip;
        V.equipTotalMeva = mevaTotal;
        V.equipEquipHaste = fmtPct(equip.haste_pct);
        V.equipTotalHaste = fmtPct(equip.haste_pct);
        // 被ダメ系は「被ダメージ-」表記で符号を反転して表示する
        // (軽減 -30% → 30%。増加装備なら負値になり悪化が分かる)
        const flipDt = (v: number | null | undefined) => (v ? -v : v);
        V.equipEquipDt = fmtPct(flipDt(equip.damage_taken_pct));
        V.equipTotalDt = fmtPct(flipDt(equip.damage_taken_pct));
        V.equipEquipPdt = fmtPct(flipDt(equip.physical_damage_taken_pct));
        V.equipTotalPdt = fmtPct(flipDt(equip.physical_damage_taken_pct));
        V.equipEquipMdt = fmtPct(flipDt(equip.magic_damage_taken_pct));
        V.equipTotalMdt = fmtPct(flipDt(equip.magic_damage_taken_pct));

        const magicAttackTotal = totalStats.magic_attack != null ? totalStats.magic_attack : 0;
        const magicAccuracyTotal = (equip.magic_accuracy || 0) + (totalStats.magic_accuracy_bonus || 0);
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
        V.statAaPdl = pctOrDash(equip.physical_damage_limit_pct);
        V.statAaSb = numOrDash(totalStats.subtle_blow);
        V.statAaSb2 = numOrDash(equip.subtle_blow_2);

        // ----- Tab 3: 遠隔攻撃 -----
        // 装備抽出 + キャラ側総合の合成値。カタログ項目も同じ値を参照するため
        // derived として渡す (式の二重定義を避ける)
        const trueShotTotal = (equip.true_shot || 0) + (totalStats.trueshot || 0);
        const recycleTotal = (equip.recycle || 0) + (totalStats.recycle || 0);
        const doubleShotTotal = (equip.double_shot_pct || 0) + (totalStats.double_shot_pct || 0);
        const tripleShotTotal = (equip.triple_shot_pct || 0) + (totalStats.triple_shot_pct || 0);
        const conserveMpTotal = (equip.conserve_mp || 0) + (totalStats.conserve_mp || 0);
        V.statRaSkill = formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value);
        V.statRaAtk = totalStats.ranged_attack != null ? totalStats.ranged_attack : '-';
        V.statRaAcc = totalStats.ranged_accuracy != null ? totalStats.ranged_accuracy : '-';
        V.statRaStp = numOrDash(totalStats.store_tp);
        V.statRaSb = numOrDash(totalStats.subtle_blow);
        V.statRaSb2 = numOrDash(equip.subtle_blow_2);
        V.statRaDoubleShot = pctOrDash(doubleShotTotal);
        V.statRaTripleShot = pctOrDash(tripleShotTotal);
        V.statRaDoubleShotDmg = pctOrDash(equip.double_shot_damage_pct);
        V.statRaTripleShotDmg = pctOrDash(equip.triple_shot_damage_pct);
        V.statRaCrit = pctOrDash(equip.critical_hit_rate_pct);
        V.statRaCritDmg = pctOrDash(equip.critical_hit_damage_pct);
        V.statRaPdl = pctOrDash(equip.physical_damage_limit_pct);
        V.statRaTs = numOrDash(trueShotTotal);
        V.statRaRecycle = numOrDash(recycleTotal);

        // ----- Tab 4: 近接物理 WS -----
        // 魔命スキルは武器スロット (メイン/サブ) の装備分のみ表示。レンジ枠は
        // メイン分を表示 (属性 WS タブと同じ規則)。
        const mainMaccSkill = equip.slot_stats?.main?.magic_accuracy_skill || 0;
        const subMaccSkill = equip.slot_stats?.sub?.magic_accuracy_skill || 0;
        const mainSkillValue = totalStats.main_weapon_skill_value || 0;
        const rangedSkillValue = totalStats.ranged_weapon_skill_value || 0;
        // レンジの実効魔命は WS を撃てる弓術/射撃の装備時のみ算出。
        // 楽器 (弦/管) や投てきではスキル値が WS の魔命に寄与しないため '-' 表示
        const rangedIsWsWeapon =
            totalStats.ranged_weapon_skill === 'Archery' ||
            totalStats.ranged_weapon_skill === 'Marksmanship';
        // 実効魔命 = 魔命+ (装備 + ボーナス) + メインの魔命スキル + 当該スキル値。
        // キーは 'main' | 'ranged' | 魔法種別 (魔法分は Tab 9-19 のループで追加)。
        // メイン/レンジ行のみ表示 (サブ行は UI 上ダッシュ固定)
        const maccTotals: Record<string, number> = {
            main: mainSkillValue + mainMaccSkill + magicAccuracyTotal,
            ranged: rangedIsWsWeapon
                ? rangedSkillValue + mainMaccSkill + magicAccuracyTotal
                : 0,
        };
        V.statMwsSkill = formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value);
        V.statMwsAtk = numOrDash(totalStats.main_attack);
        V.statMwsAcc = numOrDash(totalStats.main_accuracy);
        V.statMwsMaccSkill = numOrDash(mainMaccSkill);
        V.statMwsMaccTotal = numOrDash(maccTotals.main);
        V.statMwsSubSkill = formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value);
        V.statMwsSubAtk = totalStats.sub_attack != null ? totalStats.sub_attack : '-';
        V.statMwsSubAcc = totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-';
        V.statMwsSubMaccSkill = numOrDash(subMaccSkill);
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
        V.statRwsMaccSkill = numOrDash(mainMaccSkill);
        V.statRwsMaccTotal = numOrDash(maccTotals.ranged);
        V.statRwsStp = numOrDash(totalStats.store_tp);
        V.statRwsSb = numOrDash(totalStats.subtle_blow);
        V.statRwsSb2 = numOrDash(equip.subtle_blow_2);
        V.statRwsCrit = pctOrDash(equip.critical_hit_rate_pct);
        V.statRwsCritDmg = pctOrDash(equip.critical_hit_damage_pct);
        V.statRwsWsdmg = pctOrDash(wsDamagePct);
        V.statRwsTpb = numOrDash(equip.tp_bonus);
        V.statRwsScb = numOrDash(skillchainBonusTotal);
        V.statRwsPdl = pctOrDash(equip.physical_damage_limit_pct);
        V.statRwsTs = numOrDash(trueShotTotal);

        // ----- Tab 6: 属性 WS -----
        V.statEwsMainSkill = formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value);
        V.statEwsMainMaccSkill = numOrDash(mainMaccSkill);
        V.statEwsMainMaccTotal = numOrDash(maccTotals.main);
        V.statEwsRangedSkill = formatWeaponSkill(totalStats.ranged_weapon_skill, totalStats.ranged_weapon_skill_value);
        V.statEwsRangedMaccTotal = numOrDash(maccTotals.ranged);
        V.statEwsMatk = numOrDash(magicAttackTotal);
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
        V.statMewsSkill = formatWeaponSkill(totalStats.main_weapon_skill, totalStats.main_weapon_skill_value);
        V.statMewsAtk = numOrDash(totalStats.main_attack);
        V.statMewsAcc = numOrDash(totalStats.main_accuracy);
        V.statMewsMaccSkill = numOrDash(mainMaccSkill);
        V.statMewsMaccTotal = numOrDash(maccTotals.main);
        V.statMewsSubSkill = formatWeaponSkill(totalStats.sub_weapon_skill, totalStats.sub_weapon_skill_value);
        V.statMewsSubAtk = totalStats.sub_attack != null ? totalStats.sub_attack : '-';
        V.statMewsSubAcc = totalStats.sub_accuracy != null ? totalStats.sub_accuracy : '-';
        V.statMewsSubMaccSkill = numOrDash(subMaccSkill);
        V.statMewsMatk = numOrDash(magicAttackTotal);
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
        V.statRewsMaccTotal = numOrDash(maccTotals.ranged);
        V.statRewsMatk = numOrDash(magicAttackTotal);
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
        V.statRewsTs = numOrDash(trueShotTotal);

        // ----- Tab 9-19: 魔法 (11 種別) -----
        // 各種別ごとに: 該当スキル + 魔攻/魔命/魔回避/魔法ダメ + INT/MND/CHR/MP
        // 呪歌 (歌唱/弦楽器/管楽器), 風水 (風水/風水鈴) は複数スキルを持つ。
        const effSkillsForMagic = totalStats.effective_skills || {};
        // レンジスロットの楽器/風水鈴種別 (呪歌・風水の魔命合計とスキル列切替に使う)
        const songInstrument: 'wind' | 'string' | null =
            totalStats.ranged_weapon_skill === 'WindInstrument' ? 'wind'
            : totalStats.ranged_weapon_skill === 'StringInstrument' ? 'string'
            : null;
        const geoHandbell = totalStats.ranged_weapon_skill === 'Handbell';
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
            // 実効魔命 = 魔命+ + メインの魔命スキル + 当該魔法スキル値。
            // 種別キー (subtab の kind = prefix 小文字) で maccTotals に追加する。
            // 呪歌: 歌唱 + 管楽器 (管楽器装備時のみ加算。弦楽器はスキル表示のみで加算しない)
            // 風水: 風水魔法 + 風水鈴 (レンジに風水鈴装備時のみ加算)
            let maccMagicSkill: number;
            if (prefix === 'Song') {
                maccMagicSkill = (effSkillsForMagic.Singing || 0)
                    + (songInstrument === 'wind' ? effSkillsForMagic.WindInstrument || 0 : 0);
            } else if (prefix === 'Geomancy') {
                maccMagicSkill = (effSkillsForMagic.Geomancy || 0)
                    + (geoHandbell ? effSkillsForMagic.Handbell || 0 : 0);
            } else {
                maccMagicSkill = effSkillsForMagic[skills[0] as string] || 0;
            }
            const maccTotal = magicAccuracyTotal + mainMaccSkill + maccMagicSkill;
            maccTotals[prefix.toLowerCase()] = maccTotal;
            V[`statMg${prefix}MaccSkill`] = numOrDash(mainMaccSkill);
            V[`statMg${prefix}MaccTotal`] = numOrDash(maccTotal);
            V[`statMg${prefix}Matk`] = numOrDash(magicAttackTotal);
            V[`statMg${prefix}Mdmg`] = numOrDash(magicDamageTotal);
            V[`statMg${prefix}ConserveMp`] = numOrDash(conserveMpTotal);
            V[`statMg${prefix}FastCast`] = pctOrDash(totalStats.fast_cast_pct);
            V[`statMg${prefix}QuickMagic`] = pctOrDash(equip.quick_magic_pct);
            // MB 系は MB の成立する種別のみ表示 (神聖/精霊/暗黒/忍術/青)。
            // MB.ボーナス = ジョブ特性 + ギフト、MBダメージ/II = 装備抽出
            if (['Divine', 'Elemental', 'Dark', 'Ninjutsu', 'Blue'].includes(prefix)) {
                V[`statMg${prefix}MbBonus`] = numOrDash(totalStats.magic_burst_damage);
                V[`statMg${prefix}MbDmg`] = numOrDash(equip.magic_burst_damage);
                V[`statMg${prefix}MbDmg2`] = numOrDash(equip.magic_burst_damage_2);
            }
            V[`statMg${prefix}Int`] = totalStats.int || '-';
            V[`statMg${prefix}Mnd`] = totalStats.mnd || '-';
            V[`statMg${prefix}Chr`] = totalStats.chr || '-';
            V[`statMg${prefix}Mp`] = totalStats.mp || '-';
        });

        // 再詠唱間隔 (装備抽出のみ。データに存在する 4 種: 精霊/青は %、歌/忍術は秒)
        V.statMgElementalRecast = pctOrDash(equip.elemental_recast_delay_pct);
        V.statMgBlueRecast = pctOrDash(equip.blue_recast_delay_pct);
        V.statMgSongRecast = numOrDash(equip.song_recast_delay);
        V.statMgNinjutsuRecast = numOrDash(equip.ninjutsu_recast_delay);

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

        // プロパティ項目値 (docs/adr/0015)。カスタムセットの表示用に
        // 全カタログ項目 + 全ユーザー定義項目を毎回計算する (選択に依存しない)。
        const propertyValues: Record<string, string | number> = {};
        const propertyCtx: PropertyValueContext = {
            equip,
            totalStats,
            derived: {
                magicAttackTotal,
                magicAccuracyTotal,
                magicDamageTotal,
                wsDamagePct,
                skillchainBonusTotal,
                statusResists: statusResistTotals,
                mainMaccSkill,
                subMaccSkill,
                maccTotals,
                trueShotTotal,
                recycleTotal,
                doubleShotTotal,
                tripleShotTotal,
                conserveMpTotal,
            },
        };
        for (const item of BUILTIN_PROPERTY_ITEMS) {
            try {
                propertyValues[item.id] = item.resolve(propertyCtx);
            } catch {
                propertyValues[item.id] = '-';
            }
        }
        const userItemTotals = calculateUserPropertyValues(
            currentEquipSlots,
            propsetsStore.get().userItems
        );
        for (const [id, raw] of Object.entries(userItemTotals)) {
            propertyValues[id] = numOrDash(raw);
        }

        return {
            values: V,
            effectiveSkills,
            propertyValues,
            rangedWsWeapon: rangedIsWsWeapon,
            songInstrument,
            geoHandbell,
            weaponSkillKinds: {
                main: totalStats.main_weapon_skill ?? null,
                sub: totalStats.sub_weapon_skill ?? null,
                ranged: totalStats.ranged_weapon_skill ?? null,
            },
        };
    } catch (e) {
        console.error('Error calculating equipment edit status:', e);
        return null;
    }
}
