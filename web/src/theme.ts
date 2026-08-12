// Mantine テーマ (docs/adr/0013)。既存のダーク配色 (index.css) を
// Mantine の CSS 変数側にも反映し、Mantine コンポーネントと手書き CSS が
// 同じ見た目になるようにする。
import { createTheme, type CSSVariablesResolver } from '@mantine/core';

// 既存アクセント #f0c040 を shade 5 に置いた 10 段階
const gold = [
    '#fdf7e3',
    '#f9ecc0',
    '#f6e19c',
    '#f4d878',
    '#f2cc5a',
    '#f0c040',
    '#d9a92e',
    '#b98f22',
    '#997417',
    '#7a5b0e',
] as const;

export const theme = createTheme({
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    colors: { gold },
    primaryColor: 'gold',
    primaryShade: 5,
});

// 既存 index.css の基調色に合わせる (アプリはダーク固定)
export const cssVariablesResolver: CSSVariablesResolver = () => ({
    variables: {},
    light: {},
    dark: {
        '--mantine-color-body': '#1a1a2e',
        '--mantine-color-text': '#eee',
        '--mantine-color-default-border': '#444',
    },
});
