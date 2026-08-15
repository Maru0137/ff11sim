// 未保存ガード (docs/adr/0020) の分岐を検証する。
// 「保存できないときに移動してしまう」「破棄したのに dirty が残る」といった
// 事故は UI からは気づきにくいので、ここで押さえる。
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
    guard,
    guardStore,
    guardCancel,
    guardDiscardAndProceed,
    guardSaveAndProceed,
    registerDirtyEditor,
    findDirtyEditor,
    isAnyDirty,
    type DirtyEditor,
} from './dirty-guard';

interface FakeEditor extends DirtyEditor {
    dirty: boolean;
}

function fakeEditor(overrides: Partial<DirtyEditor> & { dirty?: boolean } = {}): FakeEditor {
    const editor: FakeEditor = {
        dirty: overrides.dirty ?? true,
        label: overrides.label ?? (() => '装備セット「WS用」'),
        isDirty: () => editor.dirty,
        save: overrides.save ?? (async () => true),
        discard: overrides.discard ?? (() => { editor.dirty = false; }),
    };
    return editor;
}

let unregister: (() => void)[] = [];

function register(id: string, editor: DirtyEditor) {
    unregister.push(registerDirtyEditor(id, editor));
}

beforeEach(() => {
    unregister.forEach((fn) => fn());
    unregister = [];
    guardStore.set(null);
});

describe('guard', () => {
    it('未編集なら確認せずそのまま実行する', () => {
        const editor = fakeEditor({ dirty: false });
        register('a', editor);
        const proceed = vi.fn();

        guard('タブ切替', proceed);

        expect(proceed).toHaveBeenCalledTimes(1);
        expect(guardStore.get()).toBeNull();
    });

    it('未保存ならダイアログを開き、操作は保留する', () => {
        register('a', fakeEditor());
        const proceed = vi.fn();

        guard('タブ切替', proceed);

        expect(proceed).not.toHaveBeenCalled();
        expect(guardStore.get()?.action).toBe('タブ切替');
        expect(guardStore.get()?.error).toBeNull();
    });

    it('editorId を指定すると他のエディタの未保存は無視する', () => {
        register('a', fakeEditor({ dirty: true }));
        register('b', fakeEditor({ dirty: false }));
        const proceed = vi.fn();

        guard('タブ切替', proceed, { editorId: 'b' });

        expect(proceed).toHaveBeenCalledTimes(1);
    });

    it('登録解除したエディタは未保存の判定から外れる', () => {
        const off = registerDirtyEditor('a', fakeEditor());
        expect(isAnyDirty()).toBe(true);
        off();
        expect(isAnyDirty()).toBe(false);
        expect(findDirtyEditor()).toBeNull();
    });
});

describe('保存して続行', () => {
    it('保存に成功したら操作を実行してダイアログを閉じる', async () => {
        const save = vi.fn(async (): Promise<true | string> => true);
        register('a', fakeEditor({ save }));
        const proceed = vi.fn();
        guard('タブ切替', proceed);

        await guardSaveAndProceed();

        expect(save).toHaveBeenCalledTimes(1);
        expect(proceed).toHaveBeenCalledTimes(1);
        expect(guardStore.get()).toBeNull();
    });

    it('保存できないときは移動せず、理由を出したままダイアログに留まる', async () => {
        register('a', fakeEditor({ save: async () => '装備セット名を入力してください。' }));
        const proceed = vi.fn();
        guard('タブ切替', proceed);

        await guardSaveAndProceed();

        expect(proceed).not.toHaveBeenCalled();
        expect(guardStore.get()?.error).toBe('装備セット名を入力してください。');
        expect(guardStore.get()?.saving).toBe(false);
    });

    it('保存が例外を投げても移動しない', async () => {
        register('a', fakeEditor({
            save: async () => {
                throw new Error('ネットワークエラー');
            },
        }));
        const proceed = vi.fn();
        guard('タブ切替', proceed);

        await guardSaveAndProceed();

        expect(proceed).not.toHaveBeenCalled();
        expect(guardStore.get()?.error).toBe('ネットワークエラー');
    });
});

describe('破棄して続行 / キャンセル', () => {
    it('破棄では discard を呼んでから操作を実行する (dirty を残さない)', () => {
        const editor = fakeEditor();
        register('a', editor);
        const proceed = vi.fn();
        guard('タブ切替', proceed);

        guardDiscardAndProceed();

        expect(editor.dirty).toBe(false);
        expect(proceed).toHaveBeenCalledTimes(1);
        expect(guardStore.get()).toBeNull();
    });

    it('キャンセルでは操作を実行せず onCancel だけ呼ぶ', () => {
        const editor = fakeEditor();
        register('a', editor);
        const proceed = vi.fn();
        const onCancel = vi.fn();
        guard('キャラクターの変更', proceed, { onCancel });

        guardCancel();

        expect(proceed).not.toHaveBeenCalled();
        expect(onCancel).toHaveBeenCalledTimes(1);
        expect(editor.dirty).toBe(true);
        expect(guardStore.get()).toBeNull();
    });
});
