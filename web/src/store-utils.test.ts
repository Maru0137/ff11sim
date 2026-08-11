import { describe, it, expect, vi } from 'vitest';
import { createStore } from './store-utils';

describe('createStore', () => {
    it('set した値を get で取得できる', () => {
        const store = createStore(1);
        expect(store.get()).toBe(1);
        store.set(2);
        expect(store.get()).toBe(2);
    });

    it('set で購読者へ通知され、unsubscribe 後は通知されない', () => {
        const store = createStore(0);
        const listener = vi.fn();
        const unsubscribe = store.subscribe(listener);

        store.set(1);
        expect(listener).toHaveBeenCalledTimes(1);

        unsubscribe();
        store.set(2);
        expect(listener).toHaveBeenCalledTimes(1);
    });
});
