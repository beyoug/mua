/**
 * formatters.ts - 格式化工具
 * 
 * 注意：大部分格式化由后端 Rust 统一处理，前端仅保留必要的聚合计算函数
 */

/**
 * 格式化字节为人类可读的速度（用于前端聚合统计）
 * @param bytesPerSecond - 每秒字节数
 * @returns 格式化的速度字符串，如 "12.50|MB/s"
 */
export function formatSpeed(bytesPerSecond: number): string {
    if (bytesPerSecond === 0) return '0.00|B/s';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));
    const val = (bytesPerSecond / Math.pow(k, i)).toFixed(2);
    const unit = i < sizes.length ? `${sizes[i]}/s` : 'B/s';
    return `${val}|${unit}`;
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
