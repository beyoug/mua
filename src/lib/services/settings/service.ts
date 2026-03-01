import { downloadDir } from "@tauri-apps/api/path";
import { get, writable } from "svelte/store";
import { getAppConfig, saveAppConfig } from "$lib/api/settings";
import { createLogger } from "$lib/utils/logger";
import type { AppConfig, AppSettingsPatch } from "./types";

const logger = createLogger("SettingsService");
const APPEARANCE_CACHE_KEY = "mua:appearance";

const DEFAULT_CONFIG: AppConfig = {
	rpcPort: 6800,
	closeToTray: true,
	autoResume: false,
	useCustomAria2: false,
	autoStart: false,
	maxConcurrentDownloads: 3,
	uaHistory: [],
	defaultSavePath: "",
	globalMaxDownloadLimit: "",
	globalMaxUploadLimit: "",
	theme: "default",
	colorMode: "dark",
	particlesEnabled: true,
	startMinimized: false,
	btTrackers: "",
	enableDht: true,
	enablePeerExchange: true,
	enableSeeding: true,
	seedRatio: 1.0,
	dhtListenPort: "6881",
	listenPort: "6881",
};

function readCachedAppearance(): Partial<
	Pick<AppConfig, "theme" | "colorMode">
> {
	if (typeof window === "undefined") return {};

	try {
		const raw = window.localStorage.getItem(APPEARANCE_CACHE_KEY);
		if (!raw) return {};

		const parsed = JSON.parse(raw) as { theme?: unknown; colorMode?: unknown };
		const appearance: Partial<Pick<AppConfig, "theme" | "colorMode">> = {};

		if (parsed.theme === "default") {
			appearance.theme = "default";
		}

		if (
			parsed.colorMode === "dark" ||
			parsed.colorMode === "light" ||
			parsed.colorMode === "auto"
		) {
			appearance.colorMode = parsed.colorMode;
		}

		return appearance;
	} catch {
		return {};
	}
}

function cacheAppearance(config: Pick<AppConfig, "theme" | "colorMode">): void {
	if (typeof window === "undefined") return;

	try {
		window.localStorage.setItem(
			APPEARANCE_CACHE_KEY,
			JSON.stringify({
				theme: config.theme,
				colorMode: config.colorMode,
			}),
		);
	} catch {}
}

const INITIAL_CONFIG: AppConfig = {
	...DEFAULT_CONFIG,
	...readCachedAppearance(),
};

export const appSettings = writable<AppConfig>(INITIAL_CONFIG);

function hasConfigChange(current: AppConfig, next: AppConfig): boolean {
	const keys = new Set<keyof AppConfig>([
		...(Object.keys(current) as (keyof AppConfig)[]),
		...(Object.keys(next) as (keyof AppConfig)[]),
	]);

	for (const key of keys) {
		if (!Object.is(current[key], next[key])) {
			return true;
		}
	}

	return false;
}

export async function loadAppSettings() {
	try {
		const config = await getAppConfig<AppConfig>();

		// 自动迁移逻辑：将旧的端口范围格式转换为单端口
		if (config.dhtListenPort === "6881-6999") config.dhtListenPort = "6881";
		if (config.listenPort === "6881-6999") config.listenPort = "6881";

		if (!config.defaultSavePath) {
			try {
				const systemDownloadDir = await downloadDir();
				if (systemDownloadDir) {
					config.defaultSavePath = systemDownloadDir;
				}
			} catch (err) {
				logger.warn("Failed to get system download dir", { error: err });
			}
		}

		const next = { ...DEFAULT_CONFIG, ...config };
		appSettings.set(next);
		cacheAppearance(next);
	} catch (e) {
		logger.error("Failed to load app settings", { error: e });
	}
}

export async function saveAppSettings(config: AppConfig) {
	try {
		await saveAppConfig(config);
		appSettings.set(config);
		cacheAppearance(config);
	} catch (e) {
		logger.error("Failed to save app settings", { error: e });
		throw e;
	}
}

export async function updateAppSettings(
	patch: AppSettingsPatch,
): Promise<void> {
	const current = get(appSettings);
	const next = { ...current, ...patch };

	if (!hasConfigChange(current, next)) {
		return;
	}

	await saveAppSettings(next);
}
