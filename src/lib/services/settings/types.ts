import type { ColorMode, ThemeId } from "$lib/types/theme";

export interface AppConfig {
	rpcPort: number;
	closeToTray: boolean;
	autoResume: boolean;
	rpcSecret?: string;
	customAria2Hash?: string;
	customAria2Trusted?: boolean;
	aria2SaveSessionInterval?: number;
	useCustomAria2: boolean;
	autoStart: boolean;
	maxConcurrentDownloads: number;
	uaHistory: string[];
	defaultSavePath: string;
	globalMaxDownloadLimit: string;
	globalMaxUploadLimit: string;
	theme: ThemeId;
	colorMode: ColorMode;
	particlesEnabled: boolean;
	startMinimized: boolean;
	btTrackers: string;
	enableDht: boolean;
	enablePeerExchange: boolean;
	enableSeeding: boolean;
	seedRatio: number;
	dhtListenPort: string;
	listenPort: string;
}

export type AppSettingsPatch = Partial<AppConfig>;
