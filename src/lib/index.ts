/**
 * lib 模块统一导出
 */

// Utils
export { cn } from './utils';
export { lockScroll, unlockScroll, createScrollLockEffect } from './utils/scroll-lock';
export { parseSpeedToBytes, formatSpeed, formatAddedAt, extractFilenameFromUrl } from './utils/formatters';
export {
    DownloadStateGroups,
    isActiveTask,
    isCompletedTask,
    isDownloading,
    isRemovableTask,
    canResume,
    getStateScore
} from './utils/downloadStates';

// Config
export { queryClient } from './config/query';

// Types
export type { DownloadTask, DownloadConfig, DownloadState } from './types/download';
