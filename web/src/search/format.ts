// 検索結果テーブル用の表示整形。
//
// jobKanji / slotJa は結果テーブル専用の略記 (戦/モ/…、遠隔/耳/…) であり、
// constants.js の共有テーブル (戦士/レンジャー/左耳…) と表示テキストが
// 意図的に異なる (docs/tech-debt/inline-script-monolith.md)。統一するなら
// 略記フィールドをデータ側に追加してから。

const JOB_KANJI: Record<string, string> = {
    WAR: '戦', MNK: 'モ', WHM: '白', BLM: '黒', RDM: '赤', THF: 'シ',
    PLD: 'ナ', DRK: '暗', BST: '獣', BRD: '吟', RNG: '狩', SAM: '侍',
    NIN: '忍', DRG: '竜', SMN: '召', BLU: '青', COR: 'コ', PUP: 'か',
    DNC: '踊', SCH: '学', GEO: '風', RUN: '剣',
};

const SLOT_JA: Record<string, string> = {
    main: 'メイン', sub: 'サブ', range: '遠隔', ammo: '矢弾',
    head: '頭', body: '胴', hands: '両手', legs: '両脚',
    feet: '両足', neck: '首', waist: '腰', ear1: '耳',
    ear2: '耳', ring1: '指', ring2: '指', back: '背',
};

export function slotsToJa(slots?: string[]): string {
    if (!slots || slots.length === 0) return '-';
    const seen = new Set<string>();
    return slots
        .map((s) => SLOT_JA[s] || s)
        .filter((s) => {
            if (seen.has(s)) return false;
            seen.add(s);
            return true;
        })
        .join(' ');
}

export function jobsToKanji(jobs?: string[]): string {
    if (!jobs || jobs.length === 0) return '-';
    if (jobs.length === 22) return 'All Jobs';
    return jobs.map((j) => JOB_KANJI[j] || j).join('');
}

// 全角英数と ＋ － ― を半角へ (マッチング用の正規化)
const FULL_TO_HALF: Record<string, string> = {
    'Ａ': 'A', 'Ｂ': 'B', 'Ｃ': 'C', 'Ｄ': 'D', 'Ｅ': 'E', 'Ｆ': 'F', 'Ｇ': 'G',
    'Ｈ': 'H', 'Ｉ': 'I', 'Ｊ': 'J', 'Ｋ': 'K', 'Ｌ': 'L', 'Ｍ': 'M', 'Ｎ': 'N',
    'Ｏ': 'O', 'Ｐ': 'P', 'Ｑ': 'Q', 'Ｒ': 'R', 'Ｓ': 'S', 'Ｔ': 'T', 'Ｕ': 'U',
    'Ｖ': 'V', 'Ｗ': 'W', 'Ｘ': 'X', 'Ｙ': 'Y', 'Ｚ': 'Z',
    '０': '0', '１': '1', '２': '2', '３': '3', '４': '4',
    '５': '5', '６': '6', '７': '7', '８': '8', '９': '9',
    '＋': '+', '－': '-', '―': '-',
};

/**
 * 説明文中の指定ステータス値 ("防77", "DEF:77", "STR+5" 等) を
 * `<span class="stat-highlight">` で囲んだ HTML 文字列を返す。
 * 返値は HTML として描画される前提 (呼び出し側の dangerouslySetInnerHTML)。
 */
export function highlightStat(description: string, statName: string): string {
    if (!description || !statName) return description || '';

    let normalized = '';
    for (const char of description) {
        normalized += FULL_TO_HALF[char] || char;
    }

    const statUpper = statName.toUpperCase();
    const normalizedUpper = normalized.toUpperCase();
    const highlightRegex = new RegExp(`${statUpper}\\s*(?::\\s*)?[+\\-]?\\s*\\d+`, 'i');
    const match = normalizedUpper.match(highlightRegex);

    if (match && match.index !== undefined) {
        const idx = match.index;
        const matchLen = match[0].length;
        const before = description.substring(0, idx);
        const matched = description.substring(idx, idx + matchLen);
        const after = description.substring(idx + matchLen);
        return before + '<span class="stat-highlight">' + matched + '</span>' + after;
    }

    return description;
}
