/**
 * lib 模块统一导出
 */

// Utils
export { lockScroll, unlockScroll, createScrollLockEffect } from './utils/scroll-lock';
export { formatSpeed, formatAddedAt, extractFilenameFromUrl, formatBytes, formatDuration } from './utils/formatters';
export { isValidDownloadUrl, isMagnetUrl, validateUrl, validateDownloadConfig } from './utils/validators';
export { clickOutside } from './utils/click-outside';
export {
    DownloadStateGroups,
    isActiveTask,
    isCompletedTask,
    isRemovableTask,
    isPausedTask,
    isDownloadingTask,
    isWaitingTask,
    isErrorTask,
    isCancelledTask,
    isResumableTask,
    isTerminalTask
} from './utils/downloadStates';

// Stores
export {
    activeTasks,
    completeTasks,
    allTasks,
    downloadStats,
    addDownloadTasks,
    pauseTask,
    resumeTask,
    cancelTask,
    removeTask,
    removeTasks,
    cancelTasks,
    pauseAll,
    resumeAll,
    hasDownloadingTasks,
    hasPausedTasks
} from './stores/downloadStore';

// Types
export type { DownloadTask, DownloadConfig, DownloadState, DownloadStats } from './types/download';
