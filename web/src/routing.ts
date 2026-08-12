// hash ベースの簡易ルーティング (#/characters, #/equipsets, #/search)。
//
// ルーティングライブラリは使わない: GitHub Pages (静的配信) では History API
// のパスが 404 になるため hash 一択で、ビューは 3 つしかなく、既存の
// 「display:none 切替でビューの state を保持する」挙動 (React Router は
// unmount する) を維持したいため。
import { useSyncExternalStore } from 'react';

export type ViewId = 'characters' | 'equipsets' | 'search';

export const VIEW_IDS: readonly ViewId[] = ['characters', 'equipsets', 'search'];

/** '#/characters' 形式の hash を ViewId に解決する。不明な hash は null。 */
export function parseHash(hash: string): ViewId | null {
    const path = hash.replace(/^#\/?/, '');
    return (VIEW_IDS as readonly string[]).includes(path) ? (path as ViewId) : null;
}

export function viewHash(view: ViewId): string {
    return `#/${view}`;
}

const subscribe = (callback: () => void): (() => void) => {
    window.addEventListener('hashchange', callback);
    return () => window.removeEventListener('hashchange', callback);
};

const getHash = (): string => window.location.hash;

/** 現在の location.hash (hashchange に追従)。 */
export function useHash(): string {
    return useSyncExternalStore(subscribe, getHash);
}

export function navigate(view: ViewId): void {
    window.location.hash = viewHash(view);
}
