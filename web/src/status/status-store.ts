// ステータス表示ビューのストア (旧 web/js/equip-status.js の置き換え)。
// 装備編集状態を変更した側 (レガシー JS / React 両方) が
// updateEquipEditStatus() を呼ぶと再計算され、StatusPanel が再描画される。
import { createStore } from '../store-utils';
import { computeStatusView } from './compute';
import type { StatusView } from './compute';
import { notifyEquipState } from '../equip/equip-store';

export const statusStore = createStore<StatusView | null>(null);

export async function updateEquipEditStatus() {
    // equipState の全変更点はこの関数を呼ぶ規約なので、equipState を読む
    // React island (装備スロット UI) への変更通知もここで行う。
    // 計算 (async) を待たず同期で通知し、入力の再描画を遅らせない。
    notifyEquipState();
    statusStore.set(await computeStatusView());
}
