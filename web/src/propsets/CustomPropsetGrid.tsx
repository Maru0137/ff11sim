// カスタムプロパティセットの表示グリッド (docs/adr/0015)。
// テンプレート (SubtabContents) の表形式と違い、項目のフラットな
// ラベル/値セルを並べるだけ。値は StatusView.propertyValues から引く。
import type { StatusView } from '../status/compute';
import type { PropertySet, UserPropertyItem } from './types';
import { BUILTIN_ITEM_BY_ID } from './catalog';
import '../../styles/propsets.css';

export function CustomPropsetGrid({
    set,
    userItems,
    view,
}: {
    set: PropertySet;
    userItems: UserPropertyItem[];
    view: StatusView | null;
}) {
    if (set.items.length === 0) {
        return (
            <div className="propset-grid-empty">
                項目がありません。「プロパティセット管理」から編集してください。
            </div>
        );
    }
    return (
        <div className="propset-grid">
            {set.items.map((id) => {
                const builtin = BUILTIN_ITEM_BY_ID.get(id);
                const user = builtin ? undefined : userItems.find((u) => u.id === id);
                // 同期で来た stale id (削除済みユーザー項目など) は薄色 + '-'
                const known = !!builtin || !!user;
                return (
                    <div
                        key={id}
                        className={'propset-cell' + (known ? '' : ' propset-cell-unknown')}
                        title={user ? 'ユーザー定義項目 (装備説明文から抽出)' : undefined}
                    >
                        <span className={'propset-cell-label' + (user ? ' user' : '')}>
                            {builtin?.label ?? user?.term ?? id}
                        </span>
                        <span className="propset-cell-value">
                            {view?.propertyValues[id] ?? '-'}
                        </span>
                    </div>
                );
            })}
        </div>
    );
}
