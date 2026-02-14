import { derived, get, readable } from 'svelte/store';
import { browser } from '$app/environment';
import { appSettings, updateAppSettings } from '$lib/services/settings';
import type { AppConfig } from '$lib/services/settings';

// ============ 主题色 ============
export type ThemeId = 'cyberpunk' | 'default';

export interface Theme {
	id: ThemeId;
	name: string;
	primary: string;
	secondary: string;
	glow: string;
}

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

function normalizeThemeId(value: unknown): ThemeId {
	if (value === 'cyberpunk' || value === 'default') return value;
	return 'default';
}

// ============ 颜色模式 ============
export type ColorMode = 'dark' | 'light' | 'auto';

export const colorModes: { id: ColorMode; name: string }[] = [
	{ id: 'auto', name: '跟随系统' },
	{ id: 'light', name: '浅色' },
	{ id: 'dark', name: '深色' }
];

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
	subscribe: derived(appSettings, ($s) => normalizeThemeId($s.theme)).subscribe,
	set: (val: ThemeId) => updateConfigKey('theme', normalizeThemeId(val))
};

export const colorMode = {
	subscribe: derived(appSettings, $s => $s.colorMode as ColorMode).subscribe,
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
		const mode = $appSettings.colorMode as ColorMode;
		if (mode === 'auto') {
			return $systemPrefersDark ? 'dark' : 'light';
		}
		return mode;
	}
);
