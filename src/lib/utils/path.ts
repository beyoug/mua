import { homeDir } from '@tauri-apps/api/path';

let cachedHomeDir: string | null = null;

/**
 * 将路径中的用户主目录替换为 "~"
 * @param path - 完整路径
 * @returns 缩短后的路径
 */
export async function compactPath(path: string): Promise<string> {
    if (!path) return '';

    try {
        if (!cachedHomeDir) {
            cachedHomeDir = await homeDir();
        }

        if (cachedHomeDir && path.startsWith(cachedHomeDir)) {
            return path.replace(cachedHomeDir, '~');
        }
    } catch (e) {
        console.error('Failed to compact path:', e);
    }

    return path;
}
