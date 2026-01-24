import { writable } from 'svelte/store';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { getAria2ConfigPath, readAria2Config, importAria2Config as importAria2ConfigCmd } from '$lib/api/cmd';

export const aria2Config = writable<string>('');
export const configPath = writable<string>('');
export const isImporting = writable<boolean>(false);

export async function loadAria2Config() {
    try {
        const path = await getAria2ConfigPath();
        configPath.set(path);
        const config = await readAria2Config();
        aria2Config.set(config);
    } catch (e) {
        console.error('Failed to load aria2 config:', e);
    }
}

export async function importAria2Config() {
    isImporting.set(true);
    try {
        const selected = await openDialog({
            filters: [{
                name: 'Config',
                extensions: ['conf', 'txt']
            }]
        });

        if (selected) {
            const path = typeof selected === 'string' ? selected : (selected as any).path;

            await importAria2ConfigCmd(path);
            await loadAria2Config(); // Reload to show preview
            alert('配置导入成功！请重启应用以生效。');
        }
    } catch (e) {
        console.error('Failed to import config:', e);
        alert('导入失败: ' + e);
    } finally {
        isImporting.set(false);
    }
}
