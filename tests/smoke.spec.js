// スモークテスト: ページが根本的に壊れていないことを確認する。
//
// 値の正しさは検証しない。それは Rust 側のユニットテスト (cargo test) の責務。
// ここで見るのは、Rust のテストでは検出できない層:
//   - JS の参照漏れ (削除したモジュールの識別子を使い続けている等)
//   - WASM の初期化失敗・アセットの 404
//   - JS と WASM のシグネチャ不一致
//
// 背景: 装備ロジックの Rust 移植時、削除した equip-stats.js の WEAPON_SKILL_KEYS
// を index.html が参照したままになり、本番でステータスが全て 0 になった。
// Rust のテスト 177 件は全て通っていた。この層のテストが無いと再発する。
import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';

// テスト用キャラクターは UI 操作ではなく localStorage へ直接投入する。
// UI フォームは要素数が多く操作が不安定なうえ、ここで検証したいのは
// 「保存済みキャラクターの表示経路」であって入力フォームではない。
const jobsData = JSON.parse(readFileSync('data/jobs.json', 'utf8')).data;
const skillsData = JSON.parse(readFileSync('data/skills.json', 'utf8')).data;

function makeCharacter() {
  const job_levels = {};
  for (const j of jobsData) {
    job_levels[j.key] = { level: j.key === 'War' ? 99 : 0, master_lv: 0 };
  }
  const merit_points = {
    hp: 0, mp: 0, str_: 0, dex: 0, vit: 0, agi: 0, int: 0, mnd: 0, chr: 0,
  };
  const job_points = { categories: {} };
  for (const j of jobsData) {
    job_points.categories[j.key] = { ranks: Array(10).fill(0) };
  }
  const values = {};
  for (const s of skillsData) values[s.key] = 0;
  return {
    name: 'smoke', race: 'Hum',
    job_levels, merit_points, job_points,
    skills: { values },
  };
}

// コンソールエラーと pageerror を収集する。どのテストでも最終的に空であること。
function collectErrors(page) {
  const errors = [];
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text());
  });
  page.on('pageerror', (e) => errors.push(`pageerror: ${e.message}`));
  return errors;
}

test('トップページがエラーなしで読み込める', async ({ page }) => {
  const errors = collectErrors(page);
  await page.goto('/');
  // WASM の初期化と初期描画を待つ
  await page.waitForTimeout(3000);
  expect(errors).toEqual([]);
});

test('検索ページがエラーなしで読み込め、検索が動く', async ({ page }) => {
  const errors = collectErrors(page);
  await page.goto('/search.html');
  await page.waitForTimeout(3000);
  // WASM 埋め込みデータでの検索が 1 件以上返ること (表示名は日本語)
  await page.fill('#searchQuery', 'Excalibur');
  await page.click('#searchBtn');
  await page.waitForTimeout(1500);
  const results = await page.locator('#resultsContainer').innerText();
  expect(results).toContain('エクスカリバー');
  expect(errors).toEqual([]);
});

test('保存済みキャラクターのステータスが 0 でなく表示される', async ({ page }) => {
  const errors = collectErrors(page);
  // localStorage はオリジンごとなので、一度ページを開いてから投入して再読み込みする。
  // 装備セットが 0 件だと編集フォームごと隠れてステータスが出ないため、
  // セットを 1 つ入れておく (job はセレクタの value = Job キーに合わせる)。
  //
  // スロットは空にしない。装備の抽出・合算・スキルボーナス集計は装備があって
  // 初めて実行され、空だとその経路のバグ (WEAPON_SKILL_KEYS 事故もこれ) を
  // 素通しする。武器スキルボーナス持ちの装備を意図的に選ぶ。
  // 21071 = C. Palug Hammer ("Club skill +N" を含む)。
  await page.goto('/');
  await page.evaluate((ch) => {
    localStorage.setItem('ff11sim_characters', JSON.stringify([ch]));
    localStorage.setItem('ff11sim_equipsets', JSON.stringify([
      {
        name: 'smoke-set', character: ch.name, job: 'War',
        slots: { main: { item_id: 21071, skill: 11 } },
      },
    ]));
  }, makeCharacter());
  await page.reload();
  await page.waitForTimeout(3000);

  // 装備セットタブでキャラクターとジョブを選ぶと、最初のセットが自動選択され
  // ステータス表示が走る
  await page.click('button[data-tab="tab-equipsets"]');
  await page.selectOption('#equipSelectChar', 'smoke');
  await page.selectOption('#equipSelectJob', 'War');
  await page.waitForTimeout(2000);

  // ベース HP が表示され、0 でないこと。
  // WEAPON_SKILL_KEYS 事故では例外で表示処理が中断し、ここが 0 のままだった。
  const hpText = await page.locator('#equipBaseHp').innerText();
  const hp = parseInt(hpText.replace(/[^0-9-]/g, ''), 10);
  expect(Number.isFinite(hp)).toBe(true);
  expect(hp).toBeGreaterThan(0);

  expect(errors).toEqual([]);
});
