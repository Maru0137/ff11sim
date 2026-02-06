/**
 * FFXI Equipment Search Module
 */

class ItemSearch {
    constructor() {
        this.items = [];
        this.loaded = false;
    }

    /**
     * Load items from JSON file
     */
    async load(url = './data/items.json') {
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`Failed to load items: ${response.status}`);
            }
            const data = await response.json();
            this.items = data.items || [];
            this.loaded = true;
            console.log(`Loaded ${this.items.length} items`);
            return this.items.length;
        } catch (error) {
            console.error('Failed to load items:', error);
            throw error;
        }
    }

    /**
     * Get available properties for filtering
     */
    getFilterableProperties() {
        return [
            { name: 'id', type: 'number', label: 'ID' },
            { name: 'en', type: 'string', label: 'Name (EN)' },
            { name: 'ja', type: 'string', label: 'Name (JA)' },
            { name: 'category', type: 'string', label: 'Category' },
            { name: 'level', type: 'number', label: 'Level' },
            { name: 'damage', type: 'number', label: 'Damage' },
            { name: 'delay', type: 'number', label: 'Delay' },
            { name: 'skill', type: 'number', label: 'Skill' },
            { name: 'jobs', type: 'array', label: 'Jobs' },
            { name: 'slots', type: 'array', label: 'Slots' },
            { name: 'races', type: 'array', label: 'Races' },
        ];
    }

    /**
     * Get available operators for a property type
     */
    getOperators(propertyType) {
        switch (propertyType) {
            case 'number':
                return [
                    { value: '=', label: '=' },
                    { value: '!=', label: '!=' },
                    { value: '>=', label: '>=' },
                    { value: '<=', label: '<=' },
                    { value: '>', label: '>' },
                    { value: '<', label: '<' },
                ];
            case 'string':
                return [
                    { value: 'contains', label: 'contains' },
                    { value: '=', label: '=' },
                    { value: '!=', label: '!=' },
                    { value: 'starts', label: 'starts with' },
                    { value: 'ends', label: 'ends with' },
                ];
            case 'array':
                return [
                    { value: 'contains', label: 'contains' },
                    { value: 'not_contains', label: 'not contains' },
                ];
            default:
                return [{ value: '=', label: '=' }];
        }
    }

    /**
     * Apply a single filter condition
     */
    applyFilter(item, filter) {
        const { property, operator, value } = filter;
        const itemValue = item[property];

        // Handle undefined/null values
        if (itemValue === undefined || itemValue === null) {
            return operator === '!=' || operator === 'not_contains';
        }

        // Array type (jobs, slots, races)
        if (Array.isArray(itemValue)) {
            const searchValue = String(value).toUpperCase();
            switch (operator) {
                case 'contains':
                    return itemValue.some(v =>
                        String(v).toUpperCase().includes(searchValue)
                    );
                case 'not_contains':
                    return !itemValue.some(v =>
                        String(v).toUpperCase().includes(searchValue)
                    );
                default:
                    return false;
            }
        }

        // Number type
        if (typeof itemValue === 'number') {
            const numValue = parseFloat(value);
            if (isNaN(numValue)) return false;

            switch (operator) {
                case '=': return itemValue === numValue;
                case '!=': return itemValue !== numValue;
                case '>=': return itemValue >= numValue;
                case '<=': return itemValue <= numValue;
                case '>': return itemValue > numValue;
                case '<': return itemValue < numValue;
                default: return false;
            }
        }

        // String type
        const strValue = String(itemValue).toLowerCase();
        const searchStr = String(value).toLowerCase();

        switch (operator) {
            case 'contains': return strValue.includes(searchStr);
            case '=': return strValue === searchStr;
            case '!=': return strValue !== searchStr;
            case 'starts': return strValue.startsWith(searchStr);
            case 'ends': return strValue.endsWith(searchStr);
            default: return false;
        }
    }

    /**
     * Extract stat value from description
     * Handles patterns like "STR+5", "ＳＴＲ＋５", "Attack+10", "命中+15"
     * @param {string} description The item description
     * @param {string} statName The stat to search for (e.g., "STR", "Attack")
     * @returns {number} The stat value, or 0 if not found
     */
    extractStatFromDescription(description, statName) {
        if (!description || !statName) return 0;

        // Normalize the stat name (convert to uppercase for comparison)
        const normalizedStat = statName.toUpperCase();

        // Full-width to half-width conversion map
        const fullToHalf = {
            'Ａ': 'A', 'Ｂ': 'B', 'Ｃ': 'C', 'Ｄ': 'D', 'Ｅ': 'E', 'Ｆ': 'F', 'Ｇ': 'G',
            'Ｈ': 'H', 'Ｉ': 'I', 'Ｊ': 'J', 'Ｋ': 'K', 'Ｌ': 'L', 'Ｍ': 'M', 'Ｎ': 'N',
            'Ｏ': 'O', 'Ｐ': 'P', 'Ｑ': 'Q', 'Ｒ': 'R', 'Ｓ': 'S', 'Ｔ': 'T', 'Ｕ': 'U',
            'Ｖ': 'V', 'Ｗ': 'W', 'Ｘ': 'X', 'Ｙ': 'Y', 'Ｚ': 'Z',
            '０': '0', '１': '1', '２': '2', '３': '3', '４': '4',
            '５': '5', '６': '6', '７': '7', '８': '8', '９': '9',
            '＋': '+', '－': '-', '―': '-'
        };

        // Normalize description (convert full-width to half-width)
        let normalized = '';
        for (const char of description) {
            normalized += fullToHalf[char] || char;
        }
        normalized = normalized.toUpperCase();

        // Pattern: STAT+/-NUMBER or STAT NUMBER (with optional spaces)
        // Match patterns like "STR+5", "STR +5", "STR5", "攻撃力+10"
        const patterns = [
            new RegExp(`${normalizedStat}\\s*[+]\\s*(\\d+)`, 'i'),
            new RegExp(`${normalizedStat}\\s*[-]\\s*(\\d+)`, 'i'),
        ];

        for (const pattern of patterns) {
            const match = normalized.match(pattern);
            if (match) {
                const value = parseInt(match[1], 10);
                // Check if it's a negative pattern
                if (pattern.source.includes('[-]')) {
                    return -value;
                }
                return value;
            }
        }

        return 0;
    }

    /**
     * Search items with filters
     * @param {Object} options Search options
     * @param {string} options.query Text search query (searches en and ja)
     * @param {string} options.slot Slot filter
     * @param {string} options.job Job filter
     * @param {Array} options.filters Array of filter objects {property, operator, value}
     * @param {string} options.sortBy Property to sort by
     * @param {string} options.descStat Stat name for description sorting
     * @param {string} options.sortOrder 'asc' or 'desc'
     * @param {number} options.limit Max results to return
     * @param {number} options.offset Offset for pagination
     */
    search(options = {}) {
        const {
            query = '',
            slot = '',
            job = '',
            filters = [],
            sortBy = 'id',
            descStat = '',
            sortOrder = 'asc',
            limit = 50,
            offset = 0
        } = options;

        let results = [...this.items];

        // Text search (name)
        if (query) {
            const q = query.toLowerCase();
            results = results.filter(item =>
                (item.en && item.en.toLowerCase().includes(q)) ||
                (item.ja && item.ja.includes(query)) ||
                (item.enl && item.enl.toLowerCase().includes(q)) ||
                (item.jal && item.jal.includes(query))
            );
        }

        // Slot filter
        if (slot) {
            results = results.filter(item =>
                item.slots && item.slots.some(s =>
                    s === slot || (slot === 'ear1' && s === 'ear2') || (slot === 'ring1' && s === 'ring2')
                )
            );
        }

        // Job filter
        if (job) {
            results = results.filter(item =>
                item.jobs && item.jobs.includes(job)
            );
        }

        // Apply custom filters
        for (const filter of filters) {
            if (filter.property && filter.operator && filter.value !== '') {
                results = results.filter(item => this.applyFilter(item, filter));
            }
        }

        // Get total before pagination
        const total = results.length;

        // Sort
        results.sort((a, b) => {
            let aVal, bVal;

            // Special handling for description stat sorting
            if (sortBy === 'desc_stat' && descStat) {
                aVal = this.extractStatFromDescription(a.description_ja || a.description_en || '', descStat);
                bVal = this.extractStatFromDescription(b.description_ja || b.description_en || '', descStat);
            } else {
                aVal = a[sortBy];
                bVal = b[sortBy];
            }

            // Handle undefined values
            if (aVal === undefined) aVal = sortOrder === 'asc' ? Infinity : -Infinity;
            if (bVal === undefined) bVal = sortOrder === 'asc' ? Infinity : -Infinity;

            // Compare
            if (typeof aVal === 'string' && typeof bVal === 'string') {
                return sortOrder === 'asc'
                    ? aVal.localeCompare(bVal)
                    : bVal.localeCompare(aVal);
            }

            if (sortOrder === 'asc') {
                return aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
            } else {
                return aVal > bVal ? -1 : aVal < bVal ? 1 : 0;
            }
        });

        // Pagination
        const paginatedResults = results.slice(offset, offset + limit);

        return {
            items: paginatedResults,
            total,
            offset,
            limit,
            hasMore: offset + limit < total
        };
    }

    /**
     * Get unique categories
     */
    getCategories() {
        const categories = new Set();
        for (const item of this.items) {
            if (item.category) {
                categories.add(item.category);
            }
        }
        return Array.from(categories).sort();
    }

    /**
     * Get unique jobs
     */
    getJobs() {
        return [
            'WAR', 'MNK', 'WHM', 'BLM', 'RDM', 'THF',
            'PLD', 'DRK', 'BST', 'BRD', 'RNG', 'SAM',
            'NIN', 'DRG', 'SMN', 'BLU', 'COR', 'PUP',
            'DNC', 'SCH', 'GEO', 'RUN'
        ];
    }

    /**
     * Get unique slots
     */
    getSlots() {
        return [
            'main', 'sub', 'range', 'ammo',
            'head', 'body', 'hands', 'legs', 'feet',
            'neck', 'waist', 'ear1', 'ear2', 'ring1', 'ring2', 'back'
        ];
    }
}

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ItemSearch;
}
