/**
 * 应用常量配置
 */

export const APP_NAME = 'Mua';
export const APP_VERSION = '0.0.1';

/**
 * aria2c 默认配置
 */
export const DEFAULT_ARIA2_CONFIG = {
    host: 'localhost',
    port: 6800,
    secret: '',
    secure: false
} as const;

/**
 * 下载状态
 */
export const DOWNLOAD_STATUS = {
    ACTIVE: 'active',
    WAITING: 'waiting',
    PAUSED: 'paused',
    ERROR: 'error',
    COMPLETE: 'complete',
    REMOVED: 'removed'
} as const;

/**
 * 文件大小单位
 */
export const SIZE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;
