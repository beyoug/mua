/**
 * theme.ts - 主题管理 Store
 * 支持：
 * - 三套主题色（深空、电光蓝、赛博紫）
 * - 三种颜色模式（深色、浅色、自动）
 * - 持久化到 localStorage
 */
import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

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
		primary: '#3B82F6',
		secondary: '#60A5FA',
		glow: 'rgba(59, 130, 246, 0.4)'
	},
	'cyber-purple': {
		id: 'cyber-purple',
		name: '赛博紫',
		primary: '#8B5CF6',
		secondary: '#A855F7',
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

// ============ Storage Keys ============
const THEME_KEY = 'mua-theme';
const MODE_KEY = 'mua-color-mode';
const PARTICLES_KEY = 'mua-particles-enabled';

const DEFAULT_THEME: ThemeId = 'deep-space';
const DEFAULT_MODE: ColorMode = 'dark';
const DEFAULT_PARTICLES: boolean = true;

// ============ 初始化函数 ============
function getInitialTheme(): ThemeId {
	if (!browser) return DEFAULT_THEME;
	const stored = localStorage.getItem(THEME_KEY);
	if (stored && stored in themes) {
		return stored as ThemeId;
	}
	return DEFAULT_THEME;
}

function getInitialColorMode(): ColorMode {
	if (!browser) return DEFAULT_MODE;
	const stored = localStorage.getItem(MODE_KEY);
	if (stored && ['dark', 'light', 'auto'].includes(stored)) {
		return stored as ColorMode;
	}
	return DEFAULT_MODE;
}

function getInitialParticlesEnabled(): boolean {
	if (!browser) return DEFAULT_PARTICLES;
	const stored = localStorage.getItem(PARTICLES_KEY);
	if (stored !== null) {
		return stored === 'true';
	}
	return DEFAULT_PARTICLES;
}

// ============ Theme Store ============
function createThemeStore() {
	const { subscribe, set } = writable<ThemeId>(getInitialTheme());

	return {
		subscribe,
		set: (themeId: ThemeId) => {
			if (browser) {
				localStorage.setItem(THEME_KEY, themeId);
			}
			set(themeId);
		}
	};
}

// ============ Color Mode Store ============
function createColorModeStore() {
	const { subscribe, set } = writable<ColorMode>(getInitialColorMode());

	return {
		subscribe,
		set: (mode: ColorMode) => {
			if (browser) {
				localStorage.setItem(MODE_KEY, mode);
			}
			set(mode);
		}
	};
}

// ============ Particles Enabled Store ============
function createParticlesEnabledStore() {
	const { subscribe, set, update } = writable<boolean>(getInitialParticlesEnabled());

	return {
		subscribe,
		set: (enabled: boolean) => {
			if (browser) {
				localStorage.setItem(PARTICLES_KEY, String(enabled));
			}
			set(enabled);
		},
		toggle: () => {
			update((current) => {
				const newValue = !current;
				if (browser) {
					localStorage.setItem(PARTICLES_KEY, String(newValue));
				}
				return newValue;
			});
		}
	};
}

// ============ 系统偏好检测 ============
export const systemPrefersDark = writable(true);

if (browser) {
	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	systemPrefersDark.set(mediaQuery.matches);
	mediaQuery.addEventListener('change', (e) => {
		systemPrefersDark.set(e.matches);
	});
}

// ============ Exports ============
export const currentTheme = createThemeStore();
export const colorMode = createColorModeStore();
export const particlesEnabled = createParticlesEnabledStore();

// 实际应用的颜色模式（考虑 auto）
export const effectiveColorMode = derived(
	[colorMode, systemPrefersDark],
	([$colorMode, $systemPrefersDark]) => {
		if ($colorMode === 'auto') {
			return $systemPrefersDark ? 'dark' : 'light';
		}
		return $colorMode;
	}
);
