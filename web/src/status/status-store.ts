// ステータス表示ビューのストア (旧 web/js/equip-status.js の置き換え)。
// 装備編集状態を変更した側 (レガシー JS / React 両方) が
// updateEquipEditStatus() を呼ぶと再計算され、StatusPanel が再描画される。
import { createStore } from '../store-utils';
import { computeStatusView } from './compute';
import type { StatusView } from './compute';

export const statusStore = createStore<StatusView | null>(null);

export async function updateEquipEditStatus() {
    statusStore.set(await computeStatusView());
}
