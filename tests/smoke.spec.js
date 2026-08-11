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

  // ステータスサブタブが切り替わること (タブ切り替えロジックの回帰検出)
  await page.click('button[data-subtab="subtab-melee-auto"]');
  await expect(page.locator('#subtab-melee-auto')).toHaveClass(/active/);

  expect(errors).toEqual([]);
});

// 装備セットとキャラクターを localStorage に投入して装備セットタブを開く共通手順。
// slots に入れる装備は各テストの検証対象に合わせて呼び出し側が選ぶ。
async function seedAndOpenEquipTab(page, slots) {
  await page.goto('/');
  await page.evaluate(({ ch, slots }) => {
    localStorage.setItem('ff11sim_characters', JSON.stringify([ch]));
    localStorage.setItem('ff11sim_equipsets', JSON.stringify(
      slots ? [{ name: 'smoke-set', character: ch.name, job: 'War', slots }] : []
    ));
  }, { ch: makeCharacter(), slots });
  await page.reload();
  await page.waitForTimeout(3000);
  await page.click('button[data-tab="tab-equipsets"]');
  await page.selectOption('#equipSelectChar', 'smoke');
  await page.selectOption('#equipSelectJob', 'War');
  await page.waitForTimeout(2000);
}

test('オーグメント選択でテキストとステータスが更新される', async ({ page }) => {
  const errors = collectErrors(page);
  // 23755 は web/data/augments.json に Default パス (rank 15-30) を持つ。
  // オーグメント選択肢の構築 → 選択 → テキスト表示 → ステータス集計の
  // 経路 (augments.json 由来のデータが WASM の抽出まで流れる) を検証する。
  await seedAndOpenEquipTab(page, { main: { item_id: 23755 } });

  const augPath = page.locator('.equip-slot-aug-path[data-slot="main"]');
  await expect(augPath).toBeEnabled();
  // "-- オーグメント --" 以外の選択肢が構築されていること
  expect(await augPath.locator('option').count()).toBeGreaterThan(1);

  await augPath.selectOption('0-15');
  await page.waitForTimeout(500);
  const augText = await page.locator('.equip-slot-aug-text[data-slot="main"]').innerText();
  expect(augText).toContain('飛攻');

  // カスタム説明の入力が装備ステータス集計へ反映されること
  const strBefore = await page.locator('#equipEquipStr').innerText();
  await page.fill('.equip-slot-custom-desc[data-slot="main"]', 'STR+5');
  await page.waitForTimeout(500);
  const strAfter = await page.locator('#equipEquipStr').innerText();
  expect(strAfter).not.toBe(strBefore);

  expect(errors).toEqual([]);
});

test('スロット検索で装備を選び、装備セットを保存できる', async ({ page }) => {
  const errors = collectErrors(page);
  // セット 0 件で開始し、検索 UI → アイテム選択 → 保存 → 再読込後の残存を検証。
  // 既存テストは localStorage へ直接投入するため、この対話経路は通らない。
  await seedAndOpenEquipTab(page, null);

  // セットが無いので "+" タブから新規フォームを開く
  await page.click('.equipset-tab-add');
  await expect(page.locator('#equipEditSection')).toBeVisible();

  // 検索欄にフォーカスすると WAR + メインスロットで絞った候補が出る
  const searchInput = page.locator('.equip-slot-search[data-slot="main"]');
  await searchInput.focus();
  await page.waitForTimeout(1000);
  const items = page.locator('.equip-slot-dropdown-item');
  expect(await items.count()).toBeGreaterThan(0);

  await items.first().click();
  await page.waitForTimeout(500);
  expect((await searchInput.inputValue()).length).toBeGreaterThan(0);

  await page.fill('#equipSetName', 'ui-set');
  await page.click('#btnSaveEquipSet');
  await page.waitForTimeout(1000);
  await expect(page.locator('.equipset-tab', { hasText: 'ui-set' })).toBeVisible();

  // 再読込しても保存したセットが残っていること
  await page.reload();
  await page.waitForTimeout(3000);
  await page.click('button[data-tab="tab-equipsets"]');
  await page.selectOption('#equipSelectChar', 'smoke');
  await page.selectOption('#equipSelectJob', 'War');
  await page.waitForTimeout(2000);
  await expect(page.locator('.equipset-tab', { hasText: 'ui-set' })).toBeVisible();
  expect((await searchInput.inputValue()).length).toBeGreaterThan(0);

  expect(errors).toEqual([]);
});

test('キャラクターを UI から作成できる', async ({ page }) => {
  const errors = collectErrors(page);
  // 既存テストは localStorage へ直接投入するため、フォーム経由の作成
  // (デフォルト値の構築・スキルデフォルト計算・保存) はここで検証する。
  await page.goto('/');
  await page.waitForTimeout(3000);

  await page.click('#btnNewChar');
  await expect(page.locator('#charEditSection')).toBeVisible();
  await page.fill('#charName', 'ui-char');
  await page.click('#btnSaveChar');
  await page.waitForTimeout(1000);

  await expect(page.locator('#charList')).toContainText('ui-char');
  // 装備セットタブのキャラクターセレクタにも反映されること
  await expect(page.locator('#equipSelectChar option[value="ui-char"]')).toHaveCount(1);

  expect(errors).toEqual([]);
});

test('共有 URL (?share=) で装備セットが閲覧できる', async ({ page }) => {
  const errors = collectErrors(page);
  // Supabase REST をスタブし、共有閲覧モードの分岐 (通常初期化のスキップ・
  // キャラクター snapshot からのステータス再現) をネットワーク非依存で検証する。
  const row = {
    name: 'shared-smoke',
    character_name: 'smoke',
    job: 'War',
    data: {
      slots: { main: { item_id: 21071, skill: 11 } },
      support_job: null,
      character: makeCharacter(),
    },
    created_at: '2026-01-01T00:00:00Z',
  };
  await page.route('**/rest/v1/shared_equipsets*', (route) => {
    // supabase-js の maybeSingle は Accept ヘッダでレスポンス形式が変わる
    const accept = route.request().headers()['accept'] || '';
    const body = accept.includes('vnd.pgrst.object')
      ? JSON.stringify(row)
      : JSON.stringify([row]);
    route.fulfill({ status: 200, contentType: 'application/json', body });
  });

  await page.goto('/?share=00000000-0000-0000-0000-000000000000');
  await page.waitForTimeout(3000);

  await expect(page.locator('body')).toHaveClass(/share-mode/);
  await expect(page.locator('#sharedHeader')).toBeVisible();
  await expect(page.locator('#sharedSetName')).toHaveText('shared-smoke');

  // snapshot からステータスが再現されること (characterOverride 経路)
  const hpText = await page.locator('#equipBaseHp').innerText();
  const hp = parseInt(hpText.replace(/[^0-9-]/g, ''), 10);
  expect(hp).toBeGreaterThan(0);

  expect(errors).toEqual([]);
});

test('ログインボタンが表示され、クリックしてもエラーが出ない', async ({ page }) => {
  const errors = collectErrors(page);
  // OAuth リダイレクト先をスタブし、実際の認証へ飛ばさずに配線だけ検証する
  await page.route('**/auth/v1/**', (route) => {
    route.fulfill({ status: 200, contentType: 'text/html', body: '<html></html>' });
  });

  await page.goto('/');
  await page.waitForTimeout(3000);

  const loginBtn = page.locator('#auth-ui .auth-btn-google');
  await expect(loginBtn).toBeVisible();
  await loginBtn.click();
  await page.waitForTimeout(1000);

  expect(errors).toEqual([]);
});
