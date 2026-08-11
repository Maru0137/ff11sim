// メインタブ切り替え。
// 状態は持たず、data-tab 属性で対象コンテンツを切り替えるだけ。
// ステータスサブタブは React 側 (web/src/status/StatusPanel.tsx) が持つ。

export function initTabs() {
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById(btn.dataset.tab).classList.add('active');
        });
    });
}
