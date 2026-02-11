import { writable } from 'svelte/store';
import { open as openDialog, message } from '@tauri-apps/plugin-dialog';
import { getAria2ConfigPath, readAria2Config, importAria2Config as importAria2ConfigCmd } from '$lib/api/cmd';
import { createLogger } from '$lib/utils/logger';

const logger = createLogger('Aria2ConfigStore');

export const aria2Config = writable<string>('');
export const configPath = writable<string>('');
export const isImporting = writable<boolean>(false);

type DialogSelection = string | { path?: string } | Array<string | { path?: string }> | null;

function resolveDialogPath(selection: DialogSelection): string | null {
    if (!selection) return null;
    if (typeof selection === 'string') return selection;
    if (Array.isArray(selection)) {
        const first = selection[0];
        if (!first) return null;
        return typeof first === 'string' ? first : first.path ?? null;
    }
    return selection.path ?? null;
}

export async function loadAria2Config() {
    try {
        const path = await getAria2ConfigPath();
        configPath.set(path);
        const config = await readAria2Config();
        aria2Config.set(config);
    } catch (e) {
        logger.error('Failed to load aria2 config', { error: e });
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

        const path = resolveDialogPath(selected as DialogSelection);
        if (path) {

            await importAria2ConfigCmd(path);
            await loadAria2Config(); // Reload to show preview
            await message('配置导入成功！请重启应用以生效。', { title: '导入成功', kind: 'info' });
        }
    } catch (e) {
        logger.error('Failed to import aria2 config', { error: e });
        await message('导入失败: ' + e, { title: '导入失败', kind: 'error' });
    } finally {
        isImporting.set(false);
    }
}
