// タブ切り替え (メインタブ / ステータスサブタブ)。
// 状態は持たず、data-tab / data-subtab 属性で対象コンテンツを切り替えるだけ。

export function initTabs() {
    // --- Tab switching ---
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById(btn.dataset.tab).classList.add('active');
        });
    });

    // --- Status sub-tab switching ---
    document.querySelectorAll('.status-subtab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.status-subtab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.status-subtab-content').forEach(c => c.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById(btn.dataset.subtab).classList.add('active');
        });
    });
}
