export const THEME_IDS = ["default"] as const;
export type ThemeId = (typeof THEME_IDS)[number];

export const COLOR_MODES = ["dark", "light", "auto"] as const;
export type ColorMode = (typeof COLOR_MODES)[number];

export interface Theme {
	id: ThemeId;
	name: string;
	primary: string;
	secondary: string;
	glow: string;
}

export function isThemeId(value: string): value is ThemeId {
	return THEME_IDS.includes(value as ThemeId);
}

export function coerceThemeId(
	value: string | null | undefined,
	fallback: ThemeId = "default",
): ThemeId {
	if (!value) return fallback;
	return isThemeId(value) ? value : fallback;
}

export function isColorMode(value: string): value is ColorMode {
	return COLOR_MODES.includes(value as ColorMode);
}

export function coerceColorMode(
	value: string | null | undefined,
	fallback: ColorMode = "dark",
): ColorMode {
	if (!value) return fallback;
	return isColorMode(value) ? value : fallback;
}
