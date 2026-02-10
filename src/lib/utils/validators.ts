/**
 * validators.ts - URL 和配置验证工具
 */

import type { DownloadConfig } from '$lib/types/download';

/**
 * 验证结果接口
 */
export interface ValidationResult {
    valid: boolean;
    error?: string;
}

/**
 * 验证 URL 是否有效
 * @param urlString - URL 字符串
 * @returns 是否为有效的下载 URL
 */
export function isValidDownloadUrl(urlString: string): boolean {
    if (!urlString || urlString.trim() === '') return false;

    try {
        const url = new URL(urlString);
        // 支持的协议：http, https, ftp, ftps, magnet
        const validProtocols = ['http:', 'https:', 'ftp:', 'ftps:', 'magnet:'];
        return validProtocols.includes(url.protocol);
    } catch {
        return false;
    }
}

/**
 * 验证单个 URL 并返回错误消息
 * @param urlText - URL 文本
 * @returns 验证结果（空字符串表示无错误）
 */
export function validateUrl(urlText: string): string {
    const trimmed = urlText.trim();

    if (!trimmed) {
        return '请输入下载链接';
    }

    if (!isValidDownloadUrl(trimmed)) {
        return '无效的URL格式，请使用 http/https/ftp/magnet 协议';
    }

    return '';
}

/**
 * 验证下载配置
 * @param config - 下载配置对象
 * @returns 验证结果
 */
export function validateDownloadConfig(config: DownloadConfig): ValidationResult {
    // 验证 URLs
    if (!config.urls || config.urls.length === 0) {
        return { valid: false, error: '至少需要一个下载链接' };
    }

    for (const url of config.urls) {
        if (!isValidDownloadUrl(url)) {
            return { valid: false, error: `无效的 URL: ${url}` };
        }
    }

    // 验证保存路径
    if (!config.savePath || config.savePath.trim() === '') {
        return { valid: false, error: '请指定保存路径' };
    }

    return { valid: true };
}
