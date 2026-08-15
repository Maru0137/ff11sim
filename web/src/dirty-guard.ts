// 未保存の編集を失う操作を 1 箇所でガードする (docs/adr/0020)。
//
// 編集フォームを持つ画面は registerDirtyEditor() で DirtyEditor を登録し、
// 編集内容が失われる操作 (装備セットのタブ切替・ビュー移動・ログアウト等) は
// guard() を通す。未保存の変更があれば確認ダイアログ (UnsavedChangesModal) を
// 出し、「保存して続行 / 破棄して続行 / キャンセル」を選ばせる。
//
// 保存ボタンの活性制御 (未編集なら無効) は各画面が isDirty() を直接読んで行う。
import { createStore } from './store-utils';

export interface DirtyEditor {
    /** ダイアログ本文に出す対象の呼び名 (例: 装備セット「WS用」) */
    label: () => string;
    isDirty: () => boolean;
    /** 保存を試みる。成功なら true、失敗ならユーザーに見せる理由を返す */
    save: () => Promise<true | string>;
    /** 編集内容を捨てて保存済みの状態へ戻す (「破棄して続行」で呼ばれる) */
    discard: () => void;
}

const editors = new Map<string, DirtyEditor>();

/** 登録し、解除用の関数を返す (React からは useEffect のクリーンアップに使う) */
export function registerDirtyEditor(id: string, editor: DirtyEditor): () => void {
    editors.set(id, editor);
    return () => {
        if (editors.get(id) === editor) editors.delete(id);
    };
}

/**
 * 未保存の変更を持つエディタを返す (無ければ null)。
 * id を渡した場合はそのエディタだけを見る。
 */
export function findDirtyEditor(id?: string): DirtyEditor | null {
    if (id !== undefined) {
        const editor = editors.get(id);
        return editor && editor.isDirty() ? editor : null;
    }
    for (const editor of editors.values()) {
        if (editor.isDirty()) return editor;
    }
    return null;
}

export function isAnyDirty(): boolean {
    return findDirtyEditor() !== null;
}

export interface GuardPrompt {
    /** ダイアログに出す操作名 (例: メニュー「装備検索」へ移動) */
    action: string;
    editor: DirtyEditor;
    proceed: () => void;
    onCancel?: () => void;
    /** 保存に失敗した理由 (ダイアログに留まって表示する) */
    error: string | null;
    saving: boolean;
}

/** null = ダイアログ非表示 */
export const guardStore = createStore<GuardPrompt | null>(null);

export interface GuardOptions {
    /** 対象エディタを限定する (省略時は登録順で最初の dirty なエディタ) */
    editorId?: string;
    /** キャンセル時の後始末。controlled な select を元の値へ描き戻す等 */
    onCancel?: () => void;
}

/** 編集内容が失われる操作を実行する。未保存なら確認ダイアログを挟む。 */
export function guard(action: string, proceed: () => void, options: GuardOptions = {}): void {
    const editor = findDirtyEditor(options.editorId);
    if (!editor) {
        proceed();
        return;
    }
    guardStore.set({
        action,
        editor,
        proceed,
        onCancel: options.onCancel,
        error: null,
        saving: false,
    });
}

export async function guardSaveAndProceed(): Promise<void> {
    const prompt = guardStore.get();
    if (!prompt || prompt.saving) return;
    guardStore.set({ ...prompt, saving: true, error: null });

    let result: true | string;
    try {
        result = await prompt.editor.save();
    } catch (e) {
        result = e instanceof Error ? e.message : String(e);
    }

    if (result !== true) {
        // 名前未入力・名前重複などで保存できないときは移動せずダイアログに留まる
        const current = guardStore.get();
        if (current) guardStore.set({ ...current, saving: false, error: result });
        return;
    }
    guardStore.set(null);
    prompt.proceed();
}

export function guardDiscardAndProceed(): void {
    const prompt = guardStore.get();
    if (!prompt) return;
    guardStore.set(null);
    // 破棄せずに proceed すると、移動先から戻ったときに古い編集が dirty のまま
    // 残り、無関係な操作でダイアログが出続ける
    prompt.editor.discard();
    prompt.proceed();
}

export function guardCancel(): void {
    const prompt = guardStore.get();
    if (!prompt) return;
    guardStore.set(null);
    prompt.onCancel?.();
}
