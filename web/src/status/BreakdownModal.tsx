// ステータス / プロパティセットの内訳モーダル (docs/adr/0016)。
// ADR 0014 の方針どおり Mantine は Modal のみ使い、中身は手書き CSS
// (styles/breakdown.css) に乗せる。開閉は StatusPanel のローカル state。
// 内訳の計算 (WASM 呼び出し含む) はモーダルを開いた時に遅延実行する。
import { useEffect, useState } from 'react';
import { Modal } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import { JOBS, RACE_NAMES } from '../constants';
import { equipState } from '../equip/equip-store';
import { calculateEquipSetBonuses } from '../equip/equip-bonuses';
import { calculateUserPropertyValuesPerSlot } from '../propsets/user-item-values';
import { propsetsStore } from '../propsets/propsets-store';
import { loadCharacters } from '../storage';
import { calculate_status_breakdown } from '../wasm';
import { buildStatusProfile, buildBonusStats, type StatusView } from './compute';
import {
    STATUS_BREAKDOWN_COLUMNS,
    buildPropsetColumns,
    buildBreakdownModel,
    type BreakdownCell,
    type BreakdownModel,
} from './breakdown';
import '../../styles/breakdown.css';

export interface BreakdownModalProps {
    mode: 'status' | 'propset';
    title: string;
    /** propset モード: 列にする項目 id (カタログ id / 'user:<term>') */
    itemIds?: string[];
    view: StatusView;
    onClose: () => void;
}

async function computeBreakdownModel(
    mode: 'status' | 'propset',
    itemIds: string[],
    view: StatusView
): Promise<BreakdownModel | null> {
    const charName = equipState.currentEquipChar;
    const jobKey = equipState.currentEquipJob;
    const supportJob = equipState.currentEquipSupportJob || null;
    const slots = equipState.currentEquipSlots;
    if (!charName || !jobKey) return null;

    let ch;
    if (equipState.sharedCharacterOverride) {
        ch = equipState.sharedCharacterOverride;
    } else {
        const characters = await loadCharacters();
        ch = characters.find((c: { name: string }) => c.name === charName);
    }
    if (!ch) return null;

    const profile = buildStatusProfile(ch);
    const equip = calculateEquipSetBonuses({ slots });
    const bonusStats = buildBonusStats(equip, slots);
    const breakdown = calculate_status_breakdown(profile, jobKey, supportJob, bonusStats);

    const userItems = propsetsStore.get().userItems;
    const perSlotUserValues =
        mode === 'propset' ? calculateUserPropertyValuesPerSlot(slots, userItems) : {};

    // キャラ由来行の補足ラベル (ジョブ名 + 有効レベル)。
    // サポートジョブのセレクト値は小文字キーなので、JOBS / job_levels の
    // 参照は大文字小文字を無視して正規化する (WASM 側の str_to_job と同じ扱い)
    const findJob = (key: string) => JOBS.find((j) => j.key.toLowerCase() === key.toLowerCase());
    const jobName = (key: string) => findJob(key)?.name || key;
    const jobLevelOf = (key: string) => ch.job_levels[findJob(key)?.key ?? key];
    const mainLv: number = jobLevelOf(jobKey)?.level || 0;
    const masterLv: number = jobLevelOf(jobKey)?.master_lv || 0;
    // サポート有効レベルは WASM が計算済みの値 (character_profile::to_chara) を使う
    const subEffectiveLv: number = breakdown.support_effective_lv ?? 0;
    const charItemLabels: Partial<Record<string, string>> = {
        base: 'Lv・ステータス・スキル由来',
        race: RACE_NAMES[ch.race] || ch.race,
        main_job: `${jobName(jobKey)}${mainLv} (ジョブ+特性)`,
        support_job: supportJob ? `${jobName(supportJob)}${subEffectiveLv} (ジョブ+特性)` : '',
        master_level: masterLv > 0 ? `ML${masterLv}` : '',
    };

    const columns = mode === 'status' ? STATUS_BREAKDOWN_COLUMNS : buildPropsetColumns(itemIds);
    return buildBreakdownModel({
        columns,
        slots,
        perSlotStats: equip.per_slot_stats,
        perSlotUserValues,
        charRows: breakdown.rows || {},
        totals: mode === 'status' ? view.values : view.propertyValues,
        charItemLabels,
        perSlotSkillBonuses: equip.per_slot_skill_bonuses,
        weaponSkillKinds: view.weaponSkillKinds,
    });
}

function cellClass(v: BreakdownCell): string {
    if (v === null) return 'breakdown-cell-zero';
    if (typeof v === 'number' && v < 0) return 'breakdown-cell-negative';
    return 'breakdown-cell-value';
}

function cellText(v: BreakdownCell): string {
    return v === null ? '-' : String(v);
}

export function BreakdownModal({ mode, title, itemIds, view, onClose }: BreakdownModalProps) {
    // getInitialValueInEffect: false — 初回レンダーから fullScreen を確定させる
    const isMobile = useMediaQuery('(max-width: 768px)', false, {
        getInitialValueInEffect: false,
    });
    const [model, setModel] = useState<BreakdownModel | null>(null);
    const [failed, setFailed] = useState(false);
    const [hideEmpty, setHideEmpty] = useState(false);

    useEffect(() => {
        let cancelled = false;
        (async () => {
            try {
                const m = await computeBreakdownModel(mode, itemIds || [], view);
                if (!cancelled) {
                    setModel(m);
                    setFailed(m === null);
                }
            } catch (e) {
                console.error('Error computing breakdown:', e);
                if (!cancelled) setFailed(true);
            }
        })();
        return () => {
            cancelled = true;
        };
    }, [mode, itemIds, view]);

    const rows = model
        ? hideEmpty
            ? model.rows.filter((r) => r.cells.some((c) => c !== null))
            : model.rows
        : [];

    return (
        <Modal
            opened
            onClose={onClose}
            title={title}
            size="90%"
            zIndex={900}
            fullScreen={isMobile}
            // スクロールロック中もピンチズーム/パンを許可 (iOS Safari 対策)
            removeScrollProps={{ allowPinchZoom: true }}
        >
            {failed ? (
                <div className="breakdown-empty">内訳を計算できませんでした</div>
            ) : model === null ? (
                <div className="breakdown-empty">計算中...</div>
            ) : model.columns.length === 0 ? (
                <div className="breakdown-empty">内訳を表示できる項目がありません</div>
            ) : (
                <>
                    <div className="breakdown-controls">
                        <label className="breakdown-hide-empty">
                            <input
                                type="checkbox"
                                checked={hideEmpty}
                                onChange={(e) => setHideEmpty(e.target.checked)}
                            />
                            値のない行を隠す
                        </label>
                    </div>
                    <div className="breakdown-table-wrap">
                        <table className="breakdown-table">
                            <thead>
                                <tr>
                                    <th className="breakdown-rowhead">部位</th>
                                    <th className="breakdown-itemname">装備</th>
                                    {model.columns.map((c) => (
                                        <th key={c.key}>{c.label}</th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {rows.map((r) => (
                                    <tr key={r.key} className={r.kind === 'char' ? 'breakdown-row-char' : undefined}>
                                        <td className="breakdown-rowhead">{r.label}</td>
                                        <td className="breakdown-itemname">{r.itemLabel}</td>
                                        {r.cells.map((c, i) => (
                                            <td key={model.columns[i].key} className={cellClass(c)}>
                                                {cellText(c)}
                                            </td>
                                        ))}
                                    </tr>
                                ))}
                                <tr className="breakdown-row-total">
                                    <td className="breakdown-rowhead">合計</td>
                                    <td className="breakdown-itemname"></td>
                                    {model.totals.map((c, i) => (
                                        <td key={model.columns[i].key} className={cellClass(c)}>
                                            {cellText(c)}
                                        </td>
                                    ))}
                                </tr>
                            </tbody>
                        </table>
                    </div>
                    <div className="breakdown-notes">
                        <div>
                            ※ ジョブ特性はメイン / サポートのうちランクの高い側の行に表示
                            (実際の計算と同じ採用規則)。
                        </div>
                        {mode === 'status' && (
                            <div>
                                ※ 種族・メイン・サポの基本ステータスは小数値を持ち、合算後に切り捨て。
                                「基礎」行はレベル・ステータス・スキル由来の項
                                (防御 = VIT×1.5 + Lv + α、魔防 = 100 など)。
                            </div>
                        )}
                        {mode === 'propset' && (
                            <div>
                                ※ 「基礎」行はレベル・ステータス・スキル由来の項 (攻撃 = STR +
                                武器スキル + 8 など)。スキル値列の基礎は素のキャップまでの値で、
                                メリポ/ML 行はキャップ引き上げによる増分 (キャラ値が届いていない分は
                                表示されない)。実効魔命など内訳を分解できない合成値項目は列に含まれません。
                            </div>
                        )}
                    </div>
                </>
            )}
        </Modal>
    );
}
