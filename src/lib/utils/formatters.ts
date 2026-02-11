import type { SpeedInfo } from '$lib/types/download';

/**
 * 格式化字节为人类可读的速度（用于前端聚合统计）
 * @param bytesPerSecond - 每秒字节数
 * @returns SpeedInfo 对象，如 { value: "12.50", unit: "MB/s" }
 */
export function formatSpeed(bytesPerSecond: number): SpeedInfo {
    if (bytesPerSecond === 0) return { value: '0', unit: 'B/s' };
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));
    const value = (bytesPerSecond / Math.pow(k, i)).toFixed(2);
    const unit = i < sizes.length ? `${sizes[i]}/s` : 'B/s';
    return { value, unit };
}

/**
 * 格式化日期为添加时间字符串
 * @param date - 日期对象，默认为当前时间
 * @returns 格式: "2024-05-20 14:30"
 */
export function formatAddedAt(date: Date = new Date()): string {
    return new Intl.DateTimeFormat('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false
    }).format(date).replace(/\//g, '-');
}

/**
 * 从 URL 提取文件名
 * @param url - URL 字符串
 * @returns 提取的文件名，失败时返回 'download'
 */
export function extractFilenameFromUrl(url: string): string {
    try {
        const urlObj = new URL(url);
        const pathname = urlObj.pathname;
        const segments = pathname.split('/').filter(Boolean);
        const filename = segments.pop();

        if (!filename) return 'download';

        // 处理查询参数
        const cleanFilename = filename.split('?')[0];

        return decodeURIComponent(cleanFilename);
    } catch {
        // URL 解析失败，尝试简单分割
        const parts = url.split('/').filter(Boolean);
        const lastPart = parts.pop();
        if (lastPart && !lastPart.startsWith('http')) {
            return decodeURIComponent(lastPart.split('?')[0]);
        }
        return 'download';
    }
}

/**
 * 格式化字节为人类可读的大小
 * @param bytes - 字节数
 * @param decimals - 小数位数
 * @returns 格式化后的字符串，如 "1.50 MB"
 */
export function formatBytes(bytes: number, decimals = 2): string {
    if (!+bytes) return '0 Bytes';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}
