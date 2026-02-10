/**
 * theme.ts - 主题管理 Store
 * 支持：
 * - 三套主题色（深空、电光蓝、赛博紫）
 * - 三种颜色模式（深色、浅色、自动）
 * - 持久化到 localStorage
 */
import { derived, get } from 'svelte/store';
import { appSettings, saveAppSettings } from './settings';

// ============ 主题色 ============
export type ThemeId = 'electric-blue' | 'cyber-purple' | 'deep-space';

export interface Theme {
	id: ThemeId;
	name: string;
	primary: string;
	secondary: string;
	glow: string;
}

export const themes: Record<ThemeId, Theme> = {
	'deep-space': {
		id: 'deep-space',
		name: '深空',
		primary: '#ffffff',
		secondary: '#94a3b8',
		glow: 'rgba(255, 255, 255, 0.25)'
	},
	'electric-blue': {
		id: 'electric-blue',
		name: '电光蓝',
		primary: '#4A9EFF',
		secondary: '#70B4FF',
		glow: 'rgba(74, 158, 255, 0.4)'
	},
	'cyber-purple': {
		id: 'cyber-purple',
		name: '赛博紫',
		primary: '#c084fc',
		secondary: '#a855f7',
		glow: 'rgba(139, 92, 246, 0.4)'
	}
};

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
async function updateConfigKey<K extends keyof import('./settings').AppConfig>(key: K, value: import('./settings').AppConfig[K]) {
	const current = get(appSettings);
	if (current[key] === value) return;

	await saveAppSettings({
		...current,
		[key]: value
	});
}

// ============ Theme Store (Derived from AppSettings) ============

export const currentTheme = {
	subscribe: derived(appSettings, $s => $s.theme as ThemeId).subscribe,
	set: (val: ThemeId) => updateConfigKey('theme', val)
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

// ============ 系统偏好检测 ============
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export const systemPrefersDark = writable(true);

if (browser) {
	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	systemPrefersDark.set(mediaQuery.matches);
	mediaQuery.addEventListener('change', (e) => {
		systemPrefersDark.set(e.matches);
	});
}

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
