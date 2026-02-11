import type { DownloadConfig } from '$lib/types/download';

export interface AdvancedSettingsState {
    selectedUaValue: string;
    customUserAgent: string;
    referer: string;
    headers: string;
    proxy: string;
    maxDownloadLimitValue: string;
    maxDownloadLimitUnit: string;
}

export function normalizeUrls(input: string): string {
    return input
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line)
        .join('\n');
}

export function splitUrlLines(input: string): string[] {
    return input
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line);
}

export function validateInputUrls(
    input: string,
    isValidDownloadUrl: (url: string) => boolean
): string {
    if (!input.trim()) return '';

    const lines = splitUrlLines(input);
    if (lines.length === 0) return '';

    for (let i = 0; i < lines.length; i++) {
        if (!isValidDownloadUrl(lines[i])) {
            return lines.length > 1 ? `第 ${i + 1} 行链接无效` : '无效的链接格式';
        }
    }

    return '';
}

export function hasMixedLinks(input: string, isMagnetUrl: (url: string) => boolean): boolean {
    const lines = splitUrlLines(input);
    if (lines.length < 2) return false;

    const hasMagnet = lines.some((line) => isMagnetUrl(line));
    const hasNormal = lines.some((line) => !isMagnetUrl(line));
    return hasMagnet && hasNormal;
}

export function buildDownloadConfigs(params: {
    urls: string;
    filename: string;
    savePath: string;
    canUseAdvanced: boolean;
    advanced: AdvancedSettingsState;
    effectiveUserAgent: string;
}): DownloadConfig[] {
    const { urls, filename, savePath, canUseAdvanced, advanced, effectiveUserAgent } = params;
    const lines = splitUrlLines(urls);
    if (lines.length === 0) return [];

    const limitValue = canUseAdvanced ? String(advanced.maxDownloadLimitValue || '').trim() : '';
    const maxDownloadLimit = limitValue ? `${limitValue}${advanced.maxDownloadLimitUnit}` : '';

    const userAgent = canUseAdvanced ? effectiveUserAgent : '';
    const referer = canUseAdvanced ? advanced.referer : '';
    const headers = canUseAdvanced ? advanced.headers : '';
    const proxy = canUseAdvanced ? advanced.proxy : '';

    const isMulti = lines.length > 1;
    return lines.map((url) => ({
        urls: [url],
        savePath,
        filename: isMulti ? '' : filename,
        userAgent,
        referer,
        headers,
        proxy,
        maxDownloadLimit
    }));
}
