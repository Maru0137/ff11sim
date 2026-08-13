// 左サイドバーレイアウト (docs/adr/0013)。
// - デスクトップ: 240px ⇔ 56px アイコンレール (折りたたみ状態は localStorage)
// - モバイル (< sm): ハンバーガーで開閉するオーバーレイ表示
// 共有閲覧モード (?share=) ではこのレイアウト自体を使わない (App.tsx で分岐)。
import type { ReactNode } from 'react';
import { AppShell, Burger, NavLink, Overlay, Tooltip, UnstyledButton } from '@mantine/core';
import { useDisclosure, useLocalStorage } from '@mantine/hooks';
import { navigate, type ViewId } from './routing';
import { AuthWidget } from './auth-ui';

export interface ViewMeta {
    view: ViewId;
    label: string;
    description: string;
    icon: ReactNode;
}

// AppShell の navbar/overlay より手前、既存モーダル (.modal-backdrop, z=1000) より奥
const SHELL_Z_INDEX = 200;

interface AppLayoutProps {
    views: ViewMeta[];
    activeView: ViewId;
    children: ReactNode;
}

export function AppLayout({ views, activeView, children }: AppLayoutProps) {
    const [mobileOpened, { toggle: toggleMobile, close: closeMobile }] = useDisclosure(false);
    const [collapsed, setCollapsed] = useLocalStorage({
        key: 'ff11sim_nav_collapsed',
        defaultValue: false,
    });
    const active = views.find((v) => v.view === activeView) ?? views[0];

    return (
        <AppShell
            navbar={{
                // 56px アイコンレールは width の切替で実現する
                // (AppShell の collapsed.desktop は幅 0 の完全非表示専用)
                width: collapsed ? 56 : 240,
                breakpoint: 'sm',
                collapsed: { mobile: !mobileOpened, desktop: false },
            }}
            // モバイルでは .view-content 側の padding と二重になるので詰める
            padding={{ base: 'xs', sm: 'lg' }}
            transitionDuration={200}
            zIndex={SHELL_Z_INDEX}
        >
            <AppShell.Navbar className="sidebar" data-collapsed={collapsed || undefined}>
                <div className="sidebar-brand">
                    <div className="sidebar-brand-mark">XI</div>
                    {!collapsed && (
                        <div className="sidebar-brand-name">
                            FF11
                            <br />
                            ステータスシミュレータ
                        </div>
                    )}
                </div>

                <nav className="sidebar-nav">
                    {views.map((v) => (
                        <Tooltip
                            key={v.view}
                            label={v.label}
                            position="right"
                            disabled={!collapsed}
                        >
                            <NavLink
                                component="button"
                                className="sidebar-navlink"
                                data-nav={v.view}
                                active={v.view === activeView}
                                label={collapsed ? undefined : v.label}
                                leftSection={v.icon}
                                onClick={() => {
                                    navigate(v.view);
                                    closeMobile();
                                }}
                            />
                        </Tooltip>
                    ))}
                </nav>

                <div className="sidebar-footer">
                    <div id="auth-ui" className="auth-ui sidebar-auth">
                        <AuthWidget />
                    </div>
                    <UnstyledButton
                        className="sidebar-collapse-btn"
                        visibleFrom="sm"
                        onClick={() => setCollapsed(!collapsed)}
                        title={collapsed ? 'サイドバーを展開' : 'サイドバーを折りたたむ'}
                    >
                        <IconChevronLeft />
                        {!collapsed && <span>折りたたむ</span>}
                    </UnstyledButton>
                </div>
            </AppShell.Navbar>

            {mobileOpened && (
                <Overlay
                    hiddenFrom="sm"
                    fixed
                    backgroundOpacity={0.55}
                    zIndex={SHELL_Z_INDEX - 1}
                    onClick={closeMobile}
                />
            )}

            <AppShell.Main>
                <div className="content-wrap">
                    <header className="content-header">
                        <Burger
                            hiddenFrom="sm"
                            opened={mobileOpened}
                            onClick={toggleMobile}
                            size="sm"
                            aria-label="メニュー"
                        />
                        <div>
                            <h2 className="content-title">{active.label}</h2>
                            <div className="content-desc">{active.description}</div>
                        </div>
                    </header>
                    {children}
                </div>
            </AppShell.Main>
        </AppShell>
    );
}

// アイコンは 3 つだけなのでライブラリは入れず inline SVG (stroke: currentColor)
function svgProps(size = 20) {
    return {
        width: size,
        height: size,
        viewBox: '0 0 24 24',
        fill: 'none',
        stroke: 'currentColor',
        strokeWidth: 1.8,
        strokeLinecap: 'round',
        strokeLinejoin: 'round',
    } as const;
}

export function IconCharacter() {
    return (
        <svg {...svgProps()}>
            <circle cx="12" cy="8" r="4" />
            <path d="M4 21c0-4 3.6-7 8-7s8 3 8 7" />
        </svg>
    );
}

export function IconEquipSet() {
    return (
        <svg {...svgProps()}>
            <path d="M6 3h12l2 4-8 14L4 7z" />
            <path d="M4 7h16" />
            <path d="M12 21 8.5 7" />
            <path d="m12 21 3.5-14" />
        </svg>
    );
}

export function IconSearch() {
    return (
        <svg {...svgProps()}>
            <circle cx="11" cy="11" r="7" />
            <path d="m21 21-4.3-4.3" />
        </svg>
    );
}

function IconChevronLeft() {
    return (
        <svg {...svgProps(16)}>
            <path d="m15 6-6 6 6 6" />
        </svg>
    );
}
