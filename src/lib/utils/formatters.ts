/**
 * 速度解析和格式化工具
 */

/**
 * 解析速度字符串，支持多种单位
 * @param speedStr - 例如 "12.5 MB/s", "850 KB/s"
 * @returns 字节/秒
 */
export function parseSpeedToBytes(speedStr: string): number {
    if (!speedStr) return 0;

    const match = speedStr.match(/^([\d.]+)\s*(B|KB|MB|GB)\/s$/i);
    if (!match) return 0;

    const [, value, unit] = match;
    const num = parseFloat(value);

    const multipliers: Record<string, number> = {
        'B': 1,
        'KB': 1024,
        'MB': 1024 * 1024,
        'GB': 1024 * 1024 * 1024
    };

    return num * (multipliers[unit.toUpperCase()] || 0);
}

/**
 * 格式化字节为人类可读的速度
 * @param bytesPerSecond - 每秒字节数
 * @returns 格式化的速度字符串，如 "12.5 MB/s"
 */
export function formatSpeed(bytesPerSecond: number): string {
    if (bytesPerSecond === 0) return '0 B/s';
    if (bytesPerSecond < 1024) return `${bytesPerSecond} B/s`;
    if (bytesPerSecond < 1024 * 1024) {
        return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
    }
    if (bytesPerSecond < 1024 * 1024 * 1024) {
        return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
    }
    return `${(bytesPerSecond / (1024 * 1024 * 1024)).toFixed(2)} GB/s`;
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
