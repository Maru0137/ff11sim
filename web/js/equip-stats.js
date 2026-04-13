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
    set('magic_attack', matchSigned('"Magic Atk\\.? Bonus"\\s*([+-])\\s*(\\d+)'));
    set('magic_accuracy', matchSigned('Magic Accuracy\\s*([+-])\\s*(\\d+)'));
    set('magic_evasion', matchSigned('Magic Evasion\\s*([+-])\\s*(\\d+)'));
    set('magic_damage', matchSigned('Magic [Dd]amage\\s*([+-])\\s*(\\d+)'));

    // Plain Attack/Accuracy/Evasion — use negative lookbehind to exclude Ranged/Magic variants
    set('attack', matchSigned('(?<!Ranged )(?<![A-Za-z])Attack\\s*([+-])\\s*(\\d+)(?!%)'));
    set('accuracy', matchSigned('(?<!Ranged )(?<!Magic )(?<![A-Za-z])Accuracy\\s*([+-])\\s*(\\d+)'));
    set('evasion', matchSigned('(?<!Magic )(?<![A-Za-z])Evasion\\s*([+-])\\s*(\\d+)'));

    // Attack percentage (e.g. "Attack+3%")
    set('attack_pct', matchSigned('(?<!Ranged )(?<![A-Za-z])Attack\\s*([+-])\\s*(\\d+)%'));

    // === Rate / special stats ===
    set('haste_pct', matchSigned('Haste\\s*([+-])\\s*(\\d+)%'));
    set('store_tp', matchSigned('"?Store TP"?\\s*([+-])\\s*(\\d+)'));
    set('double_attack_pct', matchSigned('"Double Attack"\\s*([+-])\\s*(\\d+)%'));
    set('triple_attack_pct', matchSigned('"Triple Attack"\\s*([+-])\\s*(\\d+)%'));
    set('critical_hit_rate_pct', matchSigned('Critical hit rate\\s*([+-])\\s*(\\d+)%'));
    set('weapon_skill_damage_pct', matchSigned('Weapon skill damage\\s*([+-])\\s*(\\d+)%'));

    // === Damage taken stats ===
    set('damage_taken_pct', matchSigned('(?<!Physical )(?<!Magic )Damage taken\\s*([+-])\\s*(\\d+)%'));
    set('physical_damage_taken_pct', matchSigned('Physical damage taken\\s*([+-])\\s*(\\d+)%'));
    set('magic_damage_taken_pct', matchSigned('Magic damage taken\\s*([+-])\\s*(\\d+)%'));
    set('magic_def_bonus', matchSigned('"Magic Def\\.? Bonus"\\s*([+-])\\s*(\\d+)'));

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
        magic_attack: 0, magic_accuracy: 0, magic_evasion: 0, magic_damage: 0,
        haste_pct: 0, store_tp: 0,
        double_attack_pct: 0, triple_attack_pct: 0,
        critical_hit_rate_pct: 0, weapon_skill_damage_pct: 0,
        damage_taken_pct: 0, physical_damage_taken_pct: 0, magic_damage_taken_pct: 0,
        magic_def_bonus: 0,
        dmg: 0, delay: 0,
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
