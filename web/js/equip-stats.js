/**
 * FFXI Equipment Stat Extraction Module
 *
 * Parses equipment stat bonuses from description_en strings.
 */

/**
 * Extract all stats from an item's description_en field.
 * @param {string} descriptionEn - The English description text
 * @returns {Object} Object with stat keys and numeric values (only non-zero stats included)
 */
function extractAllStats(descriptionEn) {
    if (!descriptionEn) return {};

    // Normalize literal \n to actual newlines
    let text = descriptionEn.replace(/\\n/g, '\n');

    // Unity Ranking ボーナスは最大値を採用してプレフィックスを外す
    // e.g. "Unity Ranking: Attack+10～15" → " Attack+15"
    text = text.replace(
        /Unity\s+Ranking:\s*([A-Za-z][\w\s]*?)\s*[+-]\s*\d+\s*[～~]\s*(\d+)/g,
        ' $1+$2'
    );

    // Strip "Pet: ..." segments (apply only to summoned pets, not the wearer).
    // Continue until the next ":"-prefixed section or end of string.
    text = text.replace(/Pet:[^:]*/g, '');

    // 省略表記を正式名称に展開（AF3+3 などで使われる "Rng. Atk." 形式に対応）。
    // 順序が重要: 複合形を単純形より先に展開する。
    // "Mag. Atk. Bonus" → "Magic Atk. Bonus" (既存 regex が拾える形を保持)
    text = text.replace(/Mag\. ?Atk\. Bonus/g, 'Magic Atk. Bonus');
    text = text.replace(/M\. ?Def\. ?B\./g, 'Magic Def. Bonus');
    // 複合: "Rng. Acc." / "Rng. Atk." / "Mag. Acc." / "Mag. Eva." / "Mag. Dmg." / "Mag. Def."
    text = text.replace(/Rng\. ?Acc\./g, 'Ranged Accuracy');
    text = text.replace(/Rng\. ?Atk\./g, 'Ranged Attack');
    text = text.replace(/Mag\. ?Acc\./g, 'Magic Accuracy');
    text = text.replace(/Mag\. ?Eva\./g, 'Magic Evasion');
    text = text.replace(/Mag\. ?Dmg\./g, 'Magic Damage');
    text = text.replace(/Mag\. ?Def\./g, 'Magic Defense');
    // 単純: "Acc.+N" / "Atk.+N" / "Eva.+N" — 直前が単語/ドットでなく数値が続く場合のみ
    text = text.replace(/(?<![A-Za-z.])Acc\.(?=\s*[+-]?\s*\d)/g, 'Accuracy');
    text = text.replace(/(?<![A-Za-z.])Atk\.(?=\s*[+-]?\s*\d)/g, 'Attack');
    text = text.replace(/(?<![A-Za-z.])Eva\.(?=\s*[+-]?\s*\d)/g, 'Evasion');

    // Expand slash-separated stats: "STR/VIT+10" → "STR+10 VIT+10"
    text = text.replace(/([A-Z]{2,3}(?:\/[A-Z]{2,3})+)\s*([+-]\s*\d+%?)/g, (_, stats, val) => {
        return stats.split('/').map(s => s + val).join(' ');
    });

    const result = {};

    // Helper: match a signed stat pattern like "STR+15" or "Attack +22"
    function matchSigned(pattern) {
        const re = new RegExp(pattern, 'i');
        const m = text.match(re);
        if (!m) return 0;
        const sign = m[1] === '-' ? -1 : 1;
        return sign * parseInt(m[2], 10);
    }

    // Helper: match a colon-format pattern like "DEF:77" or "DMG:+165"
    function matchColon(pattern) {
        const re = new RegExp(pattern, 'i');
        const m = text.match(re);
        if (!m) return 0;
        return parseInt(m[1], 10);
    }

    // Helper: match a value where the sign is optional (e.g. "Snapshot"5 = +5)
    // Default sign is "+" when omitted.
    function matchOptionalSign(pattern) {
        const re = new RegExp(pattern, 'i');
        const m = text.match(re);
        if (!m) return 0;
        const sign = m[1] === '-' ? -1 : 1;
        return sign * parseInt(m[2], 10);
    }

    // Helper: set value only if non-zero
    function set(key, value) {
        if (value !== 0) result[key] = value;
    }

    // === Basic 9 stats (flat) ===
    // Use negative lookahead (?!%) to avoid matching percentage patterns
    set('hp', matchSigned('(?<![A-Za-z])HP\\s*([+-])\\s*(\\d+)(?!\\d*%)'));
    set('mp', matchSigned('(?<![A-Za-z])MP\\s*([+-])\\s*(\\d+)(?!\\d*%)'));
    set('str', matchSigned('(?<![A-Za-z])STR\\s*(?=[+-])([+-])\\s*(\\d+)(?!\\d*%)'));
    set('dex', matchSigned('(?<![A-Za-z])DEX\\s*(?=[+-])([+-])\\s*(\\d+)(?!\\d*%)'));
    set('vit', matchSigned('(?<![A-Za-z])VIT\\s*(?=[+-])([+-])\\s*(\\d+)(?!\\d*%)'));
    set('agi', matchSigned('(?<![A-Za-z])AGI\\s*(?=[+-])([+-])\\s*(\\d+)(?!\\d*%)'));
    set('int', matchSigned('(?<![A-Za-z])INT\\s*(?=[+-])([+-])\\s*(\\d+)(?!\\d*%)'));
    set('mnd', matchSigned('(?<![A-Za-z])MND\\s*(?=[+-])([+-])\\s*(\\d+)(?!\\d*%)'));
    set('chr', matchSigned('(?<![A-Za-z])CHR\\s*(?=[+-])([+-])\\s*(\\d+)(?!\\d*%)'));

    // === Basic stats (percentage) ===
    set('hp_pct', matchSigned('(?<![A-Za-z])HP\\s*([+-])\\s*(\\d+)%'));
    set('mp_pct', matchSigned('(?<![A-Za-z])MP\\s*([+-])\\s*(\\d+)%'));

    // === Defense (colon format) ===
    set('def', matchColon('DEF:(\\d+)'));

    // === Combat stats (specific before general to avoid false matches) ===
    set('ranged_attack', matchSigned('Ranged Attack\\s*([+-])\\s*(\\d+)(?!%)'));
    set('ranged_accuracy', matchSigned('Ranged Accuracy\\s*([+-])\\s*(\\d+)'));
    // 属性別 Magic Atk. Bonus (例: "Dark Elemental \"Magic Atk. Bonus\"+28") は通常の魔攻に積まれない。
    // 直前に "Elemental " (Dark/Light/Fire/Ice/Wind/Earth/Lightning/Water) を伴うものを除外する。
    set('magic_attack', matchSigned('(?<!Elemental )"Magic Atk\\.? Bonus"\\s*([+-])\\s*(\\d+)'));
    set('magic_accuracy', matchSigned('Magic Accuracy\\s*([+-])\\s*(\\d+)'));
    // 魔命スキル: 装備の "Magic Accuracy skill +X" は通常の魔命と別枠。
    set('magic_accuracy_skill', matchSigned('Magic Accuracy skill\\s*([+-])\\s*(\\d+)'));
    set('magic_evasion', matchSigned('Magic Evasion\\s*([+-])\\s*(\\d+)'));
    set('magic_damage', matchSigned('Magic [Dd]amage\\s*([+-])\\s*(\\d+)'));

    // Plain Attack/Accuracy/Evasion — use negative lookbehind to exclude Ranged/Magic variants
    set('attack', matchSigned('(?<!Ranged )(?<![A-Za-z])Attack\\s*([+-])\\s*(\\d+)(?!%)'));
    set('accuracy', matchSigned('(?<!Ranged )(?<!Magic )(?<![Ss]kill )(?<![A-Za-z])Accuracy\\s*([+-])\\s*(\\d+)'));
    set('evasion', matchSigned('(?<!Magic )(?<![A-Za-z])Evasion\\s*([+-])\\s*(\\d+)'));

    // Attack percentage (e.g. "Attack+3%")
    set('attack_pct', matchSigned('(?<!Ranged )(?<![A-Za-z])Attack\\s*([+-])\\s*(\\d+)%'));

    // === Rate / special stats ===
    set('haste_pct', matchSigned('Haste\\s*([+-])\\s*(\\d+)%'));
    set('store_tp', matchSigned('"?Store TP"?\\s*([+-])\\s*(\\d+)'));
    // Double/Triple Attack damage は Double/Triple Attack より先に判定（部分一致回避）
    set('double_attack_damage_pct', matchSigned('Double Attack damage\\s*([+-])\\s*(\\d+)%?'));
    set('double_attack_pct', matchSigned('"Double Attack"\\s*([+-])\\s*(\\d+)%'));
    set('triple_attack_damage_pct', matchSigned('Triple Attack damage\\s*([+-])\\s*(\\d+)%?'));
    set('triple_attack_pct', matchSigned('"Triple Attack"\\s*([+-])\\s*(\\d+)%'));
    set('quad_attack_pct', matchSigned('"Quadruple Attack"\\s*([+-])\\s*(\\d+)%'));
    set('critical_hit_damage_pct', matchSigned('Critical hit damage\\s*([+-])\\s*(\\d+)%'));
    set('critical_hit_rate_pct', matchSigned('Critical hit rate\\s*([+-])\\s*(\\d+)%'));
    // 属性ゴルゲット/ベルト (フォシャ含む) や Rune Algol の WSダメは "Latent effect:" 以下にあり、
    // 同連携属性 WS 等の発動条件付きのため常時加算しない。"Latent effect:" 以降を除外して判定する。
    {
        const nonLatent = text.replace(/Latent effect:[\s\S]*/i, '');
        const m = nonLatent.match(/Weapon skill damage\s*([+-])\s*(\d+)%/i);
        set('weapon_skill_damage_pct', m ? (m[1] === '-' ? -1 : 1) * parseInt(m[2], 10) : 0);
    }
    // モクシャII を先に判定（モクシャの部分一致回避）
    set('subtle_blow_2', matchSigned('"Subtle Blow II"\\s*([+-])\\s*(\\d+)'));
    set('subtle_blow', matchSigned('"Subtle Blow"\\s*([+-])\\s*(\\d+)'));

    // TPボーナス: "TP Bonus +X" / "\"TP Bonus\" +X"。
    // 直前が "Avatar:"/"Wyvern:"/"Automaton:"/"All Jumps " 等の修飾語の場合はペット/特定アビ専用なので除外。
    set('tp_bonus', matchSigned('(?<!Avatar:\\s)(?<!Wyvern:\\s)(?<!Automaton:\\s)(?<!All Jumps )"?TP Bonus"?\\s*([+-])\\s*(\\d+)'));
    // 連携ボーナス: "Skillchain bonus +X" / "\"Skillchain Bonus\" +X"
    set('skillchain_bonus', matchSigned('"?Skillchain [Bb]onus"?\\s*([+-])\\s*(\\d+)'));
    // 物理ダメージ上限: "Physical damage limit+X%"
    set('physical_damage_limit_pct', matchSigned('Physical damage limit\\s*([+-])\\s*(\\d+)%'));
    // トゥルーショット: "True Shot"+X (RNG専用、適正距離における命中・ダメージ補正)
    set('true_shot', matchSigned('"True Shot"\\s*([+-])\\s*(\\d+)'));
    // 魔法クリティカルヒットII: "Magic Crit. Hit Rate II +X%" / "Magic critical hit rate II +X%"
    set('magic_critical_hit_2_pct', matchSigned('Magic [Cc]rit(?:ical|\\.) [Hh]it [Rr]ate II\\s*([+-])\\s*(\\d+)%'));
    // アフィニティ (魔法属性アフィニティ): エンゲージウェポンや一部装備で見られる "Affinity+X" 形式
    // 現状 items.json では装備記述に出現しないため、将来の装備追加に備えた parser のみ用意。
    set('magic_affinity', matchSigned('(?<![A-Za-z])Affinity\\s*([+-])\\s*(\\d+)'));

    // === Damage taken stats ===
    set('damage_taken_pct', matchSigned('(?<!Physical )(?<!Magic )Damage taken\\s*([+-])\\s*(\\d+)%'));
    set('physical_damage_taken_pct', matchSigned('Physical damage taken\\s*([+-])\\s*(\\d+)%'));
    set('magic_damage_taken_pct', matchSigned('Magic damage taken\\s*([+-])\\s*(\\d+)%'));
    set('magic_def_bonus', matchSigned('"Magic Def\\.? Bonus"\\s*([+-])\\s*(\\d+)'));

    // === HP/MP 自動回復・自動 TP ===
    set('regen', matchSigned('"?Regen"?\\s*([+-])\\s*(\\d+)'));
    set('refresh', matchSigned('"?Refresh"?\\s*([+-])\\s*(\\d+)'));
    set('regain', matchSigned('"?Regain"?\\s*([+-])\\s*(\\d+)'));

    // === 詠唱・ジョブアビ短縮系 ===
    // 表記揺れ:
    //   - Fast Cast / Quick Magic は基本 "+N%" だが "+N" (% なし) のケースもあり
    //   - Snapshot / Rapid Shot は "+N" 表記が主流 (% なし)、稀に符号も無い ("Snapshot"5)
    // → 符号と % を両方任意にする
    set('fast_cast_pct', matchOptionalSign('"?Fast Cast"?\\s*([+-]?)\\s*(\\d+)%?'));
    set('quick_magic_pct', matchOptionalSign('"?Quick Magic"?\\s*([+-]?)\\s*(\\d+)%?'));
    set('snapshot_pct', matchOptionalSign('"?Snapshot"?\\s*([+-]?)\\s*(\\d+)%?'));
    set('rapid_shot_pct', matchOptionalSign('"?Rapid Shot"?\\s*([+-]?)\\s*(\\d+)%?'));

    // === 属性レジスト (Fire/Ice/Wind/Earth/Lightning/Water/Light/Dark) ===
    // 注意: items.json には "Fire Resistance +N" 形式の装備は現状存在しない。
    // augments / custom_description で記述された場合のために regex は残す。
    for (const elem of ['Fire', 'Ice', 'Wind', 'Earth', 'Lightning', 'Water', 'Light', 'Dark']) {
        set(`resist_${elem.toLowerCase()}`, matchSigned(
            `(?:${elem} Resistance|Resist ${elem})\\s*([+-])\\s*(\\d+)`
        ));
    }

    // === 状態異常レジスト ===
    // 表記揺れ: "Resist Sleep+5" / "Terror resistance +30" / "Status ailment resistance +N"
    const statusResistMap = [
        ['sleep', 'Sleep'],
        ['paralysis', 'Paralysis'],
        ['bind', 'Bind'],
        ['silence', 'Silence'],
        ['gravity', 'Gravity'],
        ['slow', 'Slow'],
        ['petrification', 'Petrification'],
        ['stun', 'Stun'],
        ['poison', 'Poison'],
        ['charm', 'Charm'],
        ['blind', 'Blind'],
        ['curse', 'Curse'],
        ['virus', 'Virus'],
        ['amnesia', 'Amnesia'],
        ['terror', 'Terror'],
        ['death', 'Death'],
    ];
    for (const [key, en] of statusResistMap) {
        set(`resist_${key}`, matchSigned(
            `(?:Resist ${en}|${en} [Rr]esistance)\\s*([+-])\\s*(\\d+)`
        ));
    }

    // === Weapon stats (colon format) ===
    set('dmg', matchColon('DMG:[+]?(\\d+)'));
    set('delay', matchColon('Delay:[+]?(\\d+)'));

    // === ALLBP: applies to all 7 base parameters (STR~CHR) ===
    const allbp = matchSigned('ALL\\s*BP\\s*([+-])\\s*(\\d+)');
    if (allbp !== 0) {
        for (const key of ['str', 'dex', 'vit', 'agi', 'int', 'mnd', 'chr']) {
            result[key] = (result[key] || 0) + allbp;
        }
    }

    return result;
}

/**
 * Extract skill bonuses (e.g., "Sword skill +10", "Healing magic skill +5")
 * from an item's description_en field.
 * @param {string} descriptionEn
 * @returns {Object} skill_key -> value (only non-zero entries)
 *
 * Returned keys match WASM SkillKind serialization:
 *   HandToHand, Dagger, Sword, GreatSword, Axe, GreatAxe, Scythe, Polearm,
 *   Katana, GreatKatana, Club, Staff, Archery, Marksmanship, Throwing,
 *   Guarding, Evasion, Shield, Parrying,
 *   Divine, Healing, Enhancing, Enfeebling, Elemental, Dark, Summoning,
 *   Ninjutsu, Singing, StringInstrument, WindInstrument, BlueMagic, Geomancy, Handbell
 */
function extractSkillBonuses(descriptionEn) {
    if (!descriptionEn) return {};
    let text = descriptionEn.replace(/\\n/g, '\n');

    // Expand "A/B magic skill +X" into two entries:
    // "A magic skill +X B magic skill +X".
    // Applies when there's a trailing modifier (magic/instrument) or "skill".
    text = text.replace(
        /([A-Za-z][\w-]*(?:\/[A-Za-z][\w-]*)+)(\s+(?:magic|instrument))?(\s+skill)?\s*([+-]\s*\d+)/gi,
        (m, names, mod, skillWord, val) => {
            if (!mod && !skillWord) return m; // don't expand STR/VIT+10 style here
            const modifier = mod || '';
            const sw = skillWord || '';
            return names
                .split('/')
                .map((n) => `${n.trim()}${modifier}${sw} ${val}`)
                .join(' ');
        }
    );

    const result = {};
    const add = (key, value) => {
        if (value) result[key] = (result[key] || 0) + value;
    };
    const matchAllRegex = (re, key) => {
        for (const m of text.matchAll(re)) {
            const sign = m[1] === '-' ? -1 : 1;
            add(key, sign * parseInt(m[2], 10));
        }
    };

    // Weapon skills — require "skill" keyword to avoid collisions with
    // combat stats like "Evasion+5" or ambiguous patterns.
    matchAllRegex(/(?<![A-Za-z])Hand-to-Hand\s+skill\s*([+-])\s*(\d+)/gi, 'HandToHand');
    matchAllRegex(/(?<![A-Za-z])Dagger\s+skill\s*([+-])\s*(\d+)/gi, 'Dagger');
    matchAllRegex(/(?<![A-Za-z])Great\s+Sword\s+skill\s*([+-])\s*(\d+)/gi, 'GreatSword');
    matchAllRegex(/(?<!Great\s)(?<![A-Za-z])Sword\s+skill\s*([+-])\s*(\d+)/gi, 'Sword');
    matchAllRegex(/(?<![A-Za-z])Great\s+Axe\s+skill\s*([+-])\s*(\d+)/gi, 'GreatAxe');
    matchAllRegex(/(?<!Great\s)(?<![A-Za-z])Axe\s+skill\s*([+-])\s*(\d+)/gi, 'Axe');
    matchAllRegex(/(?<![A-Za-z])Scythe\s+skill\s*([+-])\s*(\d+)/gi, 'Scythe');
    matchAllRegex(/(?<![A-Za-z])Polearm\s+skill\s*([+-])\s*(\d+)/gi, 'Polearm');
    matchAllRegex(/(?<![A-Za-z])Great\s+Katana\s+skill\s*([+-])\s*(\d+)/gi, 'GreatKatana');
    matchAllRegex(/(?<!Great\s)(?<![A-Za-z])Katana\s+skill\s*([+-])\s*(\d+)/gi, 'Katana');
    matchAllRegex(/(?<![A-Za-z])Club\s+skill\s*([+-])\s*(\d+)/gi, 'Club');
    matchAllRegex(/(?<![A-Za-z])Staff\s+skill\s*([+-])\s*(\d+)/gi, 'Staff');
    matchAllRegex(/(?<![A-Za-z])Archery\s+skill\s*([+-])\s*(\d+)/gi, 'Archery');
    matchAllRegex(/(?<![A-Za-z])Marksmanship\s+skill\s*([+-])\s*(\d+)/gi, 'Marksmanship');
    matchAllRegex(/(?<![A-Za-z])Throwing\s+skill\s*([+-])\s*(\d+)/gi, 'Throwing');

    // Defensive skills — "skill" keyword required to avoid matching combat stats
    matchAllRegex(/(?<![A-Za-z])Guarding\s+skill\s*([+-])\s*(\d+)/gi, 'Guarding');
    matchAllRegex(/(?<![A-Za-z])Evasion\s+skill\s*([+-])\s*(\d+)/gi, 'Evasion');
    matchAllRegex(/(?<![A-Za-z])Shield\s+skill\s*([+-])\s*(\d+)/gi, 'Shield');
    matchAllRegex(/(?<![A-Za-z])Parrying\s+skill\s*([+-])\s*(\d+)/gi, 'Parrying');

    // Magic skills — "skill" required for disambiguation, except Geomancy which
    // is sometimes written as just "Geomancy +X" in FFXI item descriptions
    matchAllRegex(/(?<![A-Za-z])Divine\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'Divine');
    matchAllRegex(/(?<![A-Za-z])Healing\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'Healing');
    matchAllRegex(/(?<![A-Za-z])Enhancing\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'Enhancing');
    matchAllRegex(/(?<![A-Za-z])Enfeebling\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'Enfeebling');
    matchAllRegex(/(?<![A-Za-z])Elemental\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'Elemental');
    matchAllRegex(/(?<![A-Za-z])Dark\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'Dark');
    matchAllRegex(/(?<![A-Za-z])Summoning\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'Summoning');
    matchAllRegex(/(?<![A-Za-z])Ninjutsu\s+skill\s*([+-])\s*(\d+)/gi, 'Ninjutsu');
    matchAllRegex(/(?<![A-Za-z])Singing\s+skill\s*([+-])\s*(\d+)/gi, 'Singing');
    matchAllRegex(/(?<![A-Za-z])String\s+instrument\s+skill\s*([+-])\s*(\d+)/gi, 'StringInstrument');
    matchAllRegex(/(?<![A-Za-z])Wind\s+instrument\s+skill\s*([+-])\s*(\d+)/gi, 'WindInstrument');
    matchAllRegex(/(?<![A-Za-z])Blue\s+magic\s+skill\s*([+-])\s*(\d+)/gi, 'BlueMagic');
    matchAllRegex(/(?<![A-Za-z])Geomancy(?:\s+skill)?\s*([+-])\s*(\d+)/gi, 'Geomancy');
    matchAllRegex(/(?<![A-Za-z])Handbell(?:\s+skill)?\s*([+-])\s*(\d+)/gi, 'Handbell');

    return result;
}

// 武器スキル(SkillKind)のキー一覧 — 15種
const WEAPON_SKILL_KEYS = new Set([
    'HandToHand', 'Dagger', 'Sword', 'GreatSword', 'Axe', 'GreatAxe',
    'Scythe', 'Polearm', 'Katana', 'GreatKatana', 'Club', 'Staff',
    'Archery', 'Marksmanship', 'Throwing',
]);

/**
 * Create an empty stats object with all keys set to 0.
 */
function getEmptyStats() {
    return {
        hp: 0, mp: 0, str: 0, dex: 0, vit: 0, agi: 0, int: 0, mnd: 0, chr: 0,
        hp_pct: 0, mp_pct: 0,
        def: 0, attack: 0, accuracy: 0, evasion: 0,
        attack_pct: 0,
        ranged_attack: 0, ranged_accuracy: 0,
        magic_attack: 0, magic_accuracy: 0, magic_accuracy_skill: 0, magic_evasion: 0, magic_damage: 0,
        haste_pct: 0, store_tp: 0,
        double_attack_pct: 0, triple_attack_pct: 0, quad_attack_pct: 0,
        double_attack_damage_pct: 0, triple_attack_damage_pct: 0,
        critical_hit_rate_pct: 0, critical_hit_damage_pct: 0,
        weapon_skill_damage_pct: 0,
        subtle_blow: 0, subtle_blow_2: 0,
        tp_bonus: 0, skillchain_bonus: 0, physical_damage_limit_pct: 0,
        true_shot: 0,
        magic_critical_hit_2_pct: 0, magic_affinity: 0,
        damage_taken_pct: 0, physical_damage_taken_pct: 0, magic_damage_taken_pct: 0,
        magic_def_bonus: 0,
        dmg: 0, delay: 0,
        // 待機/回避/防御 タブ表示用
        regen: 0, refresh: 0, regain: 0,
        fast_cast_pct: 0, quick_magic_pct: 0, snapshot_pct: 0, rapid_shot_pct: 0,
        resist_fire: 0, resist_ice: 0, resist_wind: 0, resist_earth: 0,
        resist_lightning: 0, resist_water: 0, resist_light: 0, resist_dark: 0,
        resist_sleep: 0, resist_paralysis: 0, resist_bind: 0, resist_silence: 0,
        resist_gravity: 0, resist_slow: 0, resist_petrification: 0, resist_stun: 0,
        resist_poison: 0, resist_charm: 0, resist_blind: 0, resist_curse: 0,
        resist_virus: 0, resist_amnesia: 0, resist_terror: 0, resist_death: 0,
    };
}

/**
 * Sum stats from multiple items.
 * @param {Array<Object>} statsArray - Array of stat objects from extractAllStats
 * @returns {Object} Combined stats
 */
function sumStats(statsArray) {
    const totals = getEmptyStats();
    for (const stats of statsArray) {
        for (const key of Object.keys(totals)) {
            totals[key] += (stats[key] || 0);
        }
    }
    return totals;
}

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { extractAllStats, extractSkillBonuses, getEmptyStats, sumStats, WEAPON_SKILL_KEYS };
}
