import { writable } from 'svelte/store';
import {
    getAria2ConfigPath,
    readAria2Config,
    importAria2Config as importAria2ConfigCmd
} from '$lib/api/aria2';
import { createLogger } from '$lib/utils/logger';
import { pickSingleFile } from '$lib/utils/dialog';
import { showErrorFeedback, showSuccessFeedback } from '$lib/services/feedback';

const logger = createLogger('Aria2ConfigService');

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
        logger.error('Failed to load aria2 config', { error: e });
    }
}

export async function importAria2Config() {
    isImporting.set(true);
    try {
        const path = await pickSingleFile('选择 aria2 配置文件', [{
            name: 'Config',
            extensions: ['conf', 'txt']
        }]);
        if (path) {

            await importAria2ConfigCmd(path);
            await loadAria2Config(); // Reload to show preview
            await showSuccessFeedback('导入成功', '配置导入成功！请重启应用以生效。');
        }
    } catch (e) {
        logger.error('Failed to import aria2 config', { error: e });
        await showErrorFeedback('导入失败', e);
    } finally {
        isImporting.set(false);
    }
}
