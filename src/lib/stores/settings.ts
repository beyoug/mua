import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

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
    theme: string;
    colorMode: string;
    particlesEnabled: boolean;
    startMinimized: boolean;
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
    theme: 'deep-space',
    colorMode: 'dark',
    particlesEnabled: true,
    startMinimized: false
};

export const appSettings = writable<AppConfig>(DEFAULT_CONFIG);

export async function loadAppSettings() {
    try {
        const config = await invoke<AppConfig>('get_app_config');
        appSettings.set({ ...DEFAULT_CONFIG, ...config });
    } catch (e) {
        console.error('Failed to load app settings', e);
    }
}

export async function saveAppSettings(config: AppConfig) {
    try {
        await invoke('save_app_config', { config });
        appSettings.set(config);
    } catch (e) {
        console.error('Failed to save app settings', e);
        throw e;
    }
}
