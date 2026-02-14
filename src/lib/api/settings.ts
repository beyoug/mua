import { invoke } from '@tauri-apps/api/core';

export async function getAppConfig<T>(): Promise<T> {
    return invoke<T>('get_app_config');
}

export async function saveAppConfig<T>(config: T): Promise<void> {
    await invoke('save_app_config', { config });
}
