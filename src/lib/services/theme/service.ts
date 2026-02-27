/**
 * theme.ts - 主题管理 Store
 * 支持：
 * - 两套主题色（电光蓝、深空默认）
 * - 三种颜色模式（深色、浅色、自动）
 * - 持久化到 localStorage
 */
import { derived, get, readable } from 'svelte/store';
import { browser } from '$app/environment';
import { appSettings, updateAppSettings } from '$lib/services/settings';
import type { AppConfig } from '$lib/services/settings';

import {
    COLOR_MODES,
    THEME_IDS,
    coerceColorMode,
    coerceThemeId,
    type ThemeId,
    type Theme,
    type ColorMode
} from '$lib/types/theme';

export const themes: Record<ThemeId, Theme> = {
	'default': {
		id: 'default',
		name: '默认',
		primary: '#4A9EFF',
		secondary: '#70B4FF',
		glow: 'rgba(74, 158, 255, 0.4)'
	},
	'cyberpunk': {
		id: 'cyberpunk',
		name: '赛博朋克',
		primary: '#22d3ee',
		secondary: '#d946ef',
		glow: 'rgba(34, 211, 238, 0.6)'
	}
};

// ============ 颜色模式 ============
export const colorModes: ReadonlyArray<{ id: ColorMode; name: string }> = [
	{ id: 'auto', name: '跟随系统' },
	{ id: 'light', name: '浅色' },
	{ id: 'dark', name: '深色' }
];

const themeClassNames = THEME_IDS.map((id) => `theme-${id}` as const);
const requiredThemeVars = [
    '--accent-primary',
    '--accent-secondary',
    '--accent-hover',
    '--accent-active',
    '--accent-glow',
    '--accent-subtle',
    '--accent-dim',
    '--glass-bg',
    '--dialog-overlay-bg'
] as const;

// ============ 获取状态的辅助逻辑 ============

/**
 * 通用的设置保存包装
 */
async function updateConfigKey<K extends keyof AppConfig>(
	key: K,
	value: AppConfig[K]
) {
	const current = get(appSettings);
	if (current[key] === value) return;

	await updateAppSettings({
		[key]: value
	} as Pick<AppConfig, K>);
}

// ============ Theme Store (Derived from AppSettings) ============

export const currentTheme = {
	subscribe: derived(appSettings, $s => coerceThemeId($s.theme)).subscribe,
	set: (val: ThemeId) => updateConfigKey('theme', val)
};

export const colorMode = {
	subscribe: derived(appSettings, $s => coerceColorMode($s.colorMode)).subscribe,
	set: (val: ColorMode) => updateConfigKey('colorMode', val)
};

export const particlesEnabled = {
	subscribe: derived(appSettings, $s => $s.particlesEnabled).subscribe,
	set: (val: boolean) => updateConfigKey('particlesEnabled', val),
	toggle: () => {
		const current = get(appSettings).particlesEnabled;
		updateConfigKey('particlesEnabled', !current);
	}
};

export const systemPrefersDark = readable(true, (set) => {
	if (!browser) {
		return;
	}

	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	set(mediaQuery.matches);

	const onChange = (event: MediaQueryListEvent) => {
		set(event.matches);
	};

	mediaQuery.addEventListener('change', onChange);
	return () => {
		mediaQuery.removeEventListener('change', onChange);
	};
});

// 实际应用的颜色模式（考虑 auto）
export const effectiveColorMode = derived(
	[appSettings, systemPrefersDark],
	([$appSettings, $systemPrefersDark]) => {
		const mode = coerceColorMode($appSettings.colorMode);
		if (mode === 'auto') {
			return $systemPrefersDark ? 'dark' : 'light';
		}
		return mode;
	}
);

export function resolveThemeState(settings: Pick<AppConfig, 'theme' | 'colorMode'>, systemDark: boolean): {
    themeId: ThemeId;
    mode: Exclude<ColorMode, 'auto'>;
} {
    const themeId = coerceThemeId(settings.theme);
    const colorMode = coerceColorMode(settings.colorMode);
    const mode = colorMode === 'auto' ? (systemDark ? 'dark' : 'light') : colorMode;
    return { themeId, mode };
}

export function applyThemeToDocument(state: {
    themeId: ThemeId;
    mode: Exclude<ColorMode, 'auto'>;
}): void {
    if (!browser) return;

    const el = document.documentElement;
    el.classList.remove(...themeClassNames, 'light', 'dark');
    el.classList.add(`theme-${state.themeId}`, state.mode);
    el.style.colorScheme = state.mode;

    if (import.meta.env.DEV) {
        const style = getComputedStyle(el);
        const missing = requiredThemeVars.filter((token) => style.getPropertyValue(token).trim() === '');
        if (missing.length > 0) {
            console.warn(`[Theme] Missing tokens for ${state.themeId}/${state.mode}:`, missing.join(', '));
        }
    }
}
