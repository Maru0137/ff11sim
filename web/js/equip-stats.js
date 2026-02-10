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
    const text = descriptionEn.replace(/\\n/g, '\n');

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
    set('hp', matchSigned('(?<![A-Za-z])HP\\s*([+-])\\s*(\\d+)(?!%)'));
    set('mp', matchSigned('(?<![A-Za-z])MP\\s*([+-])\\s*(\\d+)(?!%)'));
    set('str', matchSigned('(?<![A-Za-z])STR\\s*([+-])\\s*(\\d+)'));
    set('dex', matchSigned('(?<![A-Za-z])DEX\\s*([+-])\\s*(\\d+)'));
    set('vit', matchSigned('(?<![A-Za-z])VIT\\s*([+-])\\s*(\\d+)'));
    set('agi', matchSigned('(?<![A-Za-z])AGI\\s*([+-])\\s*(\\d+)'));
    set('int', matchSigned('(?<![A-Za-z])INT\\s*([+-])\\s*(\\d+)'));
    set('mnd', matchSigned('(?<![A-Za-z])MND\\s*([+-])\\s*(\\d+)'));
    set('chr', matchSigned('(?<![A-Za-z])CHR\\s*([+-])\\s*(\\d+)'));

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

    // === Weapon stats (colon format) ===
    set('dmg', matchColon('DMG:[+]?(\\d+)'));
    set('delay', matchColon('Delay:[+]?(\\d+)'));

    return result;
}

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
    module.exports = { extractAllStats, getEmptyStats, sumStats };
}
