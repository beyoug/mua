import { get, writable } from 'svelte/store';
import { downloadDir } from '@tauri-apps/api/path';
import { createLogger } from '$lib/utils/logger';
import { getAppConfig, saveAppConfig } from '$lib/api/settings';
import type { AppConfig, AppSettingsPatch } from './types';

const logger = createLogger('SettingsService');

const DEFAULT_CONFIG: AppConfig = {
    rpcPort: 6800,
    closeToTray: true,
    autoResume: false,
    useCustomAria2: false,
    autoStart: false,
    maxConcurrentDownloads: 3,
    uaHistory: [],
    defaultSavePath: '...', // 将在 loadAppSettings 时从后端动态获取
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
        const config = await getAppConfig<AppConfig>();

        // 自动迁移逻辑：将旧的端口范围格式转换为单端口
        if (config.dhtListenPort === '6881-6999') config.dhtListenPort = '6881';
        if (config.listenPort === '6881-6999') config.listenPort = '6881';

        // 如果默认下载路径未设置或为占位符，尝试获取系统默认下载目录
        if (!config.defaultSavePath || config.defaultSavePath === '...') {
            try {
                const systemDownloadDir = await downloadDir();
                if (systemDownloadDir) {
                    config.defaultSavePath = systemDownloadDir;
                    // 可选：立即保存回后端，或者等待用户更改设置时保存
                    // await saveAppSettings(config);
                }
            } catch (err) {
                logger.warn('Failed to get system download dir', { error: err });
            }
        }

        appSettings.set({ ...DEFAULT_CONFIG, ...config });
    } catch (e) {
        logger.error('Failed to load app settings', { error: e });
    }
}

export async function saveAppSettings(config: AppConfig) {
    try {
        await saveAppConfig(config);
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
