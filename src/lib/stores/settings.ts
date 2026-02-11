import { get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { createLogger } from '$lib/utils/logger';

const logger = createLogger('SettingsStore');

export interface AppConfig {
    rpcPort: number;
    closeToTray: boolean;
    autoResume: boolean;
    rpcSecret?: string;
    aria2SaveSessionInterval?: number;
    useCustomAria2: boolean;
    autoStart: boolean;
    maxConcurrentDownloads: number;
    uaHistory: string[];
    defaultSavePath: string;
    globalMaxDownloadLimit: string;
    globalMaxUploadLimit: string;
    theme: string;
    colorMode: string;
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

const DEFAULT_CONFIG: AppConfig = {
    rpcPort: 6800,
    closeToTray: true,
    autoResume: false,
    useCustomAria2: false,
    autoStart: false,
    maxConcurrentDownloads: 3,
    uaHistory: [],
    defaultSavePath: '',
    globalMaxDownloadLimit: '',
    globalMaxUploadLimit: '',
    theme: 'default',
    colorMode: 'dark',
    particlesEnabled: true,
    startMinimized: false,
    btTrackers: '',
    enableDht: true,
    enablePeerExchange: true,
    enableSeeding: true,
    seedRatio: 1.0,
    dhtListenPort: '6881',
    listenPort: '6881'
};

export const appSettings = writable<AppConfig>(DEFAULT_CONFIG);

export type AppSettingsPatch = Partial<AppConfig>;

function hasConfigChange(current: AppConfig, next: AppConfig): boolean {
    const keys = new Set<keyof AppConfig>([
        ...Object.keys(current) as (keyof AppConfig)[],
        ...Object.keys(next) as (keyof AppConfig)[]
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
        const config = await invoke<AppConfig>('get_app_config');
        appSettings.set({ ...DEFAULT_CONFIG, ...config });
    } catch (e) {
        logger.error('Failed to load app settings', { error: e });
    }
}

export async function saveAppSettings(config: AppConfig) {
    try {
        await invoke('save_app_config', { config });
        appSettings.set(config);
    } catch (e) {
        logger.error('Failed to save app settings', { error: e });
        throw e;
    }
}

export async function updateAppSettings(patch: AppSettingsPatch): Promise<void> {
    const current = get(appSettings);
    const next = { ...current, ...patch };

    if (!hasConfigChange(current, next)) {
        return;
    }

    await saveAppSettings(next);
}
