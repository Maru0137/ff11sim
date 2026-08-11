// React 島とレガシー JS 層の橋渡しに使う最小の外部ストア。
// レガシー側は set/get を直接呼び、React 側は useSyncExternalStore で購読する
// (docs/roadmap/react-migration.md の「既存ストアへの接続」規約)。
export interface Store<T> {
    get: () => T;
    set: (next: T) => void;
    subscribe: (callback: () => void) => () => void;
}

export function createStore<T>(initial: T): Store<T> {
    let state = initial;
    const listeners = new Set<() => void>();
    return {
        get: () => state,
        set(next: T) {
            state = next;
            listeners.forEach((l) => l());
        },
        subscribe(callback: () => void) {
            listeners.add(callback);
            return () => {
                listeners.delete(callback);
            };
        },
    };
}
