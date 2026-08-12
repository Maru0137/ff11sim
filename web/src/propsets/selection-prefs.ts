// 装備セットごとの「最後に選択したプロパティセット」記憶 (docs/adr/0015)。
// ローカル専用の UI プリファレンス (docs/adr/0005 の UI プリファレンス層):
// storage ファサードを通さず localStorage に直接保存し、Supabase には同期しない。
// 装備セットレコードに載せると選択のたびに全行 upsert が走り、
// shared_equipsets の共有ペイロードにも漏れるため。
//
// キーは装備セットの複合識別子 (equipset-repo.ts の setKey と同形式)。
// 値は選択 id ('template:subtab-*' またはカスタムセットの UUID)。

const SELECTION_KEY = 'ff11sim_propset_selection';

function readAll(): Record<string, string> {
    try {
        const data = localStorage.getItem(SELECTION_KEY);
        const parsed = data ? JSON.parse(data) : {};
        return parsed && typeof parsed === 'object' ? parsed : {};
    } catch {
        return {};
    }
}

function writeAll(map: Record<string, string>) {
    try {
        localStorage.setItem(SELECTION_KEY, JSON.stringify(map));
    } catch {
        // UI プリファレンスなので保存失敗は無視 (消失許容)
    }
}

export const equipSetPrefKey = (character: string, job: string, name: string) =>
    `${character}|${job}|${name}`;

export function getSelectedPropsetId(equipSetKey: string): string | null {
    return readAll()[equipSetKey] ?? null;
}

export function setSelectedPropsetId(equipSetKey: string, selectionId: string) {
    const map = readAll();
    map[equipSetKey] = selectionId;
    writeAll(map);
}

export function removeSelectedPropsetId(equipSetKey: string) {
    const map = readAll();
    if (!(equipSetKey in map)) return;
    delete map[equipSetKey];
    writeAll(map);
}

/** 装備セットのリネームに追随してキーを移し替える */
export function migrateSelectionKey(oldKey: string, newKey: string) {
    if (oldKey === newKey) return;
    const map = readAll();
    if (!(oldKey in map)) return;
    map[newKey] = map[oldKey];
    delete map[oldKey];
    writeAll(map);
}
